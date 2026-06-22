pub mod ast;
pub mod lexer;

pub use ast::{Diagram, DiagramType, NodeShape, Statement, Subgraph};
pub use lexer::{Lexer, Token, TokenType};

use std::fmt;

/// Represents a parse error with location information.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Line number where the error occurred (0-indexed).
    pub line: usize,
    /// Column number where the error occurred.
    pub column: usize,
    /// Human-readable error description.
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}:{}: {}",
            self.line + 1,
            self.column,
            self.message
        )
    }
}

/// A recursive descent parser for Mermaid diagram syntax.
///
/// Turns a sequence of tokens from the [`Lexer`] into a
/// [`Diagram`] AST.
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    _source: String,
    subgraph_count: usize,
}

impl Parser {
    /// Creates a new Parser from Mermaid source code.
    ///
    /// The source code is first tokenized by the internal Lexer.
    pub fn new(code: &str) -> Self {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
        Parser {
            tokens,
            current: 0,
            _source: code.to_string(),
            subgraph_count: 0,
        }
    }

    /// Parse the tokenized Mermaid code into a Diagram AST.
    ///
    /// Returns the root [`Diagram`] node on success, or a
    /// [`ParseError`] if the input is invalid.
    pub fn parse(&mut self) -> Result<Diagram, ParseError> {
        if self.is_at_end() {
            return Err(ParseError {
                line: 0,
                column: 0,
                message: "Empty input".to_string(),
            });
        }

        // 检查第一个关键字（graph 或 flowchart）
        let diagram_type = match self.peek()?.token_type {
            TokenType::Keyword(ref k) if k == "graph" => {
                let _ = self.advance();
                DiagramType::Flowchart
            }
            TokenType::Keyword(ref k) if k == "flowchart" => {
                let _ = self.advance();
                DiagramType::Flowchart
            }
            _ => {
                return Err(self.error("Expected 'graph' or 'flowchart'"));
            }
        };

        // 解析方向（TD, LR, etc.）
        let direction = if let Ok(token) = self.peek() {
            if let TokenType::Keyword(k) = &token.token_type {
                if k == "TD" || k == "LR" || k == "RL" || k == "BT" {
                    let direction = k.clone();
                    let _ = self.advance();
                    Some(direction)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let mut statements = Vec::new();
        let mut subgraphs = Vec::new();

        // 解析语句，直到到达 end 或文件末尾
        while !self.is_at_end() {
            // 检查是否是 subgraph
            if let Ok(token) = self.peek() {
                if let TokenType::Keyword(ref k) = &token.token_type {
                    if k == "subgraph" {
                        let _ = self.advance(); // 跳过 subgraph
                        let sg = self.parse_subgraph()?;
                        subgraphs.push(sg);
                        continue;
                    }
                }
            }
            // 尝试解析一个或多个语句
            match self.parse_statements() {
                Ok(stmts) => {
                    for stmt in stmts {
                        statements.push(stmt);
                    }
                }
                Err(_e) => {
                    // 跳过错误的 token，继续解析
                    if !self.is_at_end() {
                        let _ = self.advance();
                    }
                }
            }
        }

        Ok(Diagram {
            diagram_type,
            statements,
            direction,
            subgraphs,
        })
    }

    fn parse_statements(&mut self) -> Result<Vec<Statement>, ParseError> {
        if self.is_at_end() {
            return Ok(vec![]);
        }

        let token = self.peek()?;
        match &token.token_type {
            TokenType::Keyword(k) if k == "end" => {
                let _ = self.advance();
                Ok(vec![])
            }
            TokenType::Identifier(_) => self.parse_node_or_edge_statements(),
            TokenType::Semicolon => {
                let _ = self.advance();
                Ok(vec![])
            }
            _ => {
                let _ = self.advance();
                Ok(vec![])
            }
        }
    }

    fn parse_node_or_edge_statements(&mut self) -> Result<Vec<Statement>, ParseError> {
        let from_id = self.parse_node_id()?;

        // 检查形状和标签
        let (from_label, from_shape) = self.parse_optional_shape_and_label()?;

        let mut stmts = Vec::new();

        // 如果源节点有标签或非默认形状，添加 NodeDef
        if from_label.is_some() || from_shape != NodeShape::Rect {
            stmts.push(Statement::NodeDef {
                id: from_id.clone(),
                label: from_label,
                shape: from_shape,
            });
        }

        // 检查是否是边定义（有箭头）
        if let Ok(arrow_token) = self.peek() {
            if matches!(arrow_token.token_type, TokenType::Arrow) {
                let _ = self.advance(); // 跳过箭头

                // 解析可选的边标签：-->|label|
                let edge_label = self.parse_edge_label()?;

                let to_id = self.parse_node_id()?;
                // 解析目标节点的形状和标签
                let (to_label, to_shape) = self.parse_optional_shape_and_label()?;
                // 如果目标节点有标签或非默认形状，添加 NodeDef
                if to_label.is_some() || to_shape != NodeShape::Rect {
                    stmts.push(Statement::NodeDef {
                        id: to_id.clone(),
                        label: to_label,
                        shape: to_shape,
                    });
                }
                // 添加 EdgeDef
                stmts.push(Statement::EdgeDef {
                    from: from_id,
                    to: to_id,
                    label: edge_label,
                });
                return Ok(stmts);
            }
        }

        // 否则是节点定义（如果没有 NodeDef 已添加）
        if stmts.is_empty() {
            stmts.push(Statement::NodeDef {
                id: from_id,
                label: None,
                shape: NodeShape::Rect,
            });
        }

        Ok(stmts)
    }

    /// Parse optional shape syntax and label after a node ID.
    /// Returns (label, shape). Shape defaults to Rect if no shape syntax found.
    fn parse_optional_shape_and_label(
        &mut self,
    ) -> Result<(Option<String>, NodeShape), ParseError> {
        let token = match self.peek() {
            Ok(t) => t.clone(),
            Err(_) => return Ok((None, NodeShape::Rect)),
        };

        match &token.token_type {
            // [ — 检查是 [label], [[label]], 还是 [(label)]
            TokenType::LeftBracket => {
                let _ = self.advance();
                if let Ok(next) = self.peek() {
                    match &next.token_type {
                        // [[ — 子程序
                        TokenType::LeftBracket => {
                            let _ = self.advance();
                            let label = self.read_label_text()?;
                            self.expect_right_bracket()?;
                            self.expect_right_bracket()?;
                            return Ok((Some(label), NodeShape::Subroutine));
                        }
                        // [( — 圆柱
                        TokenType::LeftParen => {
                            let _ = self.advance();
                            let label = self.read_label_text()?;
                            self.expect_right_paren()?;
                            self.expect_right_bracket()?;
                            return Ok((Some(label), NodeShape::Cylinder));
                        }
                        _ => {}
                    }
                }
                // [label] — 普通矩形
                let label = self.read_label_text()?;
                self.expect_right_bracket()?;
                Ok((Some(label), NodeShape::Rect))
            }
            // ( — 检查是 (label), ((label)), 或 ([label])
            TokenType::LeftParen => {
                let _ = self.advance();
                if let Ok(next) = self.peek() {
                    match &next.token_type {
                        // (( — 双圆
                        TokenType::LeftParen => {
                            let _ = self.advance();
                            let label = self.read_label_text()?;
                            self.expect_right_paren()?;
                            self.expect_right_paren()?;
                            return Ok((Some(label), NodeShape::DoubleCircle));
                        }
                        // ([ — 大圆角
                        TokenType::LeftBracket => {
                            let _ = self.advance();
                            let label = self.read_label_text()?;
                            self.expect_right_bracket()?;
                            self.expect_right_paren()?;
                            return Ok((Some(label), NodeShape::Rounded));
                        }
                        _ => {}
                    }
                }
                // (label) — 圆角矩形
                let label = self.read_label_text()?;
                self.expect_right_paren()?;
                Ok((Some(label), NodeShape::Circle))
            }
            // {label} — 菱形
            TokenType::LeftBrace => {
                let _ = self.advance();
                let label = self.read_label_text()?;
                self.expect_right_brace()?;
                Ok((Some(label), NodeShape::Diamond))
            }
            // >label] — 旗帜
            TokenType::GreaterThan => {
                let _ = self.advance();
                let label = self.read_label_text()?;
                self.expect_right_bracket()?;
                Ok((Some(label), NodeShape::Flag))
            }
            _ => Ok((None, NodeShape::Rect)),
        }
    }

    /// Read label text from the next token.
    fn read_label_text(&mut self) -> Result<String, ParseError> {
        let token = self.advance()?;
        match &token.token_type {
            TokenType::Identifier(s) => Ok(s.clone()),
            TokenType::String(s) => Ok(s.clone()),
            TokenType::Keyword(k) => Ok(k.clone()),
            _ => Err(self.error("Expected label text")),
        }
    }

    /// Expect and consume a `]` token.
    fn expect_right_bracket(&mut self) -> Result<(), ParseError> {
        let token = self.peek()?;
        if matches!(token.token_type, TokenType::RightBracket) {
            let _ = self.advance();
            Ok(())
        } else {
            Err(self.error("Expected ']'"))
        }
    }

    /// Expect and consume a `)` token.
    fn expect_right_paren(&mut self) -> Result<(), ParseError> {
        let token = self.peek()?;
        if matches!(token.token_type, TokenType::RightParen) {
            let _ = self.advance();
            Ok(())
        } else {
            Err(self.error("Expected ')'"))
        }
    }

    /// Expect and consume a `}` token.
    fn expect_right_brace(&mut self) -> Result<(), ParseError> {
        let token = self.peek()?;
        if matches!(token.token_type, TokenType::RightBrace) {
            let _ = self.advance();
            Ok(())
        } else {
            Err(self.error("Expected '}'"))
        }
    }

    fn parse_node_id(&mut self) -> Result<String, ParseError> {
        let token = self.advance()?;
        match &token.token_type {
            TokenType::Identifier(id) => Ok(id.clone()),
            _ => Err(self.error("Expected identifier")),
        }
    }

    /// Parse optional edge label in the format `|label|`.
    /// Returns `None` if no pipe-delimited label follows.
    fn parse_edge_label(&mut self) -> Result<Option<String>, ParseError> {
        if let Ok(token) = self.peek() {
            if matches!(token.token_type, TokenType::Pipe) {
                let _ = self.advance(); // 跳过左边的 |
                let label = self.read_edge_label_text()?;
                // 期望右边的 |
                if let Ok(next) = self.peek() {
                    if matches!(next.token_type, TokenType::Pipe) {
                        let _ = self.advance();
                        return Ok(Some(label));
                    }
                }
                return Err(self.error("Expected closing '|' for edge label"));
            }
        }
        Ok(None)
    }

    /// Read the text content of an edge label (between `|...|`).
    fn read_edge_label_text(&mut self) -> Result<String, ParseError> {
        // 累积文本直到遇到 | 或行尾
        let mut text = String::new();
        while !self.is_at_end() {
            let token = self.peek()?.clone();
            match &token.token_type {
                TokenType::Pipe => break,
                TokenType::Keyword(k) => text.push_str(k),
                TokenType::Identifier(id) => text.push_str(id),
                TokenType::String(s) => text.push_str(s),
                _ => break,
            }
            let _ = self.advance();
        }
        if text.is_empty() {
            return Err(self.error("Empty edge label"));
        }
        Ok(text)
    }

    /// Parse a subgraph block: `subgraph [title]\n...end`
    fn parse_subgraph(&mut self) -> Result<Subgraph, ParseError> {
        // 解析 subgraph 的 ID 或标题
        let id = if let Ok(token) = self.peek() {
            match &token.token_type {
                TokenType::Identifier(id) => {
                    let id = id.clone();
                    let _ = self.advance();
                    id
                }
                _ => format!("sg_{}", self.subgraph_counter()),
            }
        } else {
            format!("sg_{}", self.subgraph_counter())
        };

        let mut sub_statements = Vec::new();
        let title = None;

        // 解析内部的语句
        while !self.is_at_end() {
            if let Ok(token) = self.peek() {
                if let TokenType::Keyword(ref k) = &token.token_type {
                    if k == "end" {
                        let _ = self.advance(); // 跳过 end
                        return Ok(Subgraph {
                            id,
                            title,
                            statements: sub_statements,
                        });
                    }
                }
            }
            let stmts = self.parse_statements()?;
            for stmt in stmts {
                sub_statements.push(stmt);
            }
        }

        // 没有 end，隐式结束
        Ok(Subgraph {
            id,
            title,
            statements: sub_statements,
        })
    }

    /// Counter for auto-generating subgraph IDs
    fn subgraph_counter(&mut self) -> usize {
        let count = self.subgraph_count;
        self.subgraph_count += 1;
        count
    }

    fn peek(&self) -> Result<&Token, ParseError> {
        if self.is_at_end() {
            Err(ParseError {
                line: 0,
                column: 0,
                message: "Unexpected end of input".to_string(),
            })
        } else {
            Ok(&self.tokens[self.current])
        }
    }

    fn advance(&mut self) -> Result<Token, ParseError> {
        if self.is_at_end() {
            Err(ParseError {
                line: 0,
                column: 0,
                message: "Unexpected end of input".to_string(),
            })
        } else {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            Ok(token)
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn error(&self, message: &str) -> ParseError {
        let (line, column) = if self.current < self.tokens.len() {
            let token = &self.tokens[self.current];
            (token.line, token.column)
        } else {
            (0, 0)
        };

        ParseError {
            line,
            column,
            message: message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_flowchart() {
        let code = "graph TD\nA[Start]-->B[End]";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_empty_input() {
        let code = "";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_flowchart_keyword() {
        let code = "flowchart TD\nA-->B";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_node_with_label() {
        let code = "graph TD\nA[Start]";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 1);
        match &diagram.statements[0] {
            Statement::NodeDef { id, label, .. } => {
                assert_eq!(id, "A");
                assert_eq!(label.as_deref(), Some("Start"));
            }
            _ => panic!("Expected NodeDef"),
        }
    }

    #[test]
    fn test_parse_edge_with_labels() {
        let code = "graph TD\nA[Start]-->B[End]";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        // Should have: NodeDef(A, Start) + NodeDef(B, End) + EdgeDef(A->B)
        assert_eq!(diagram.statements.len(), 3);
        match &diagram.statements[0] {
            Statement::NodeDef { id, label, .. } => {
                assert_eq!(id, "A");
                assert_eq!(label.as_deref(), Some("Start"));
            }
            _ => panic!("Expected NodeDef for A"),
        }
        match &diagram.statements[1] {
            Statement::NodeDef { id, label, .. } => {
                assert_eq!(id, "B");
                assert_eq!(label.as_deref(), Some("End"));
            }
            _ => panic!("Expected NodeDef for B"),
        }
        match &diagram.statements[2] {
            Statement::EdgeDef { from, to, .. } => {
                assert_eq!(from, "A");
                assert_eq!(to, "B");
            }
            _ => panic!("Expected EdgeDef"),
        }
    }

    #[test]
    fn test_parse_edge_without_labels() {
        let code = "graph TD\nA-->B";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 1);
        match &diagram.statements[0] {
            Statement::EdgeDef { from, to, label } => {
                assert_eq!(from, "A");
                assert_eq!(to, "B");
                assert!(label.is_none());
            }
            _ => panic!("Expected EdgeDef"),
        }
    }

    // --- 节点形状测试 ---

    #[test]
    fn test_parse_rounded_node() {
        let code = "graph TD\nA(Rounded)";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::NodeDef { id, label, shape } => {
                assert_eq!(id, "A");
                assert_eq!(label.as_deref(), Some("Rounded"));
                assert_eq!(*shape, NodeShape::Circle);
            }
            _ => panic!("Expected NodeDef"),
        }
    }

    #[test]
    fn test_parse_diamond_node() {
        let code = "graph TD\nA{Decision}";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::NodeDef { id, label, shape } => {
                assert_eq!(id, "A");
                assert_eq!(label.as_deref(), Some("Decision"));
                assert_eq!(*shape, NodeShape::Diamond);
            }
            _ => panic!("Expected NodeDef"),
        }
    }

    #[test]
    fn test_parse_double_rounded_node() {
        let code = "graph TD\nA([Double])";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::NodeDef { id, label, shape } => {
                assert_eq!(id, "A");
                assert_eq!(label.as_deref(), Some("Double"));
                assert_eq!(*shape, NodeShape::Rounded);
            }
            _ => panic!("Expected NodeDef"),
        }
    }

    #[test]
    fn test_parse_subroutine_node() {
        let code = "graph TD\nA[[Subroutine]]";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::NodeDef { id, label, shape } => {
                assert_eq!(id, "A");
                assert_eq!(label.as_deref(), Some("Subroutine"));
                assert_eq!(*shape, NodeShape::Subroutine);
            }
            _ => panic!("Expected NodeDef"),
        }
    }

    #[test]
    fn test_parse_cylinder_node() {
        let code = "graph TD\nA[(Database)]";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::NodeDef { id, label, shape } => {
                assert_eq!(id, "A");
                assert_eq!(label.as_deref(), Some("Database"));
                assert_eq!(*shape, NodeShape::Cylinder);
            }
            _ => panic!("Expected NodeDef"),
        }
    }

    #[test]
    fn test_parse_double_circle_node() {
        let code = "graph TD\nA((Circle))";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::NodeDef { id, label, shape } => {
                assert_eq!(id, "A");
                assert_eq!(label.as_deref(), Some("Circle"));
                assert_eq!(*shape, NodeShape::DoubleCircle);
            }
            _ => panic!("Expected NodeDef"),
        }
    }

    #[test]
    fn test_parse_flag_node() {
        let code = "graph TD\nA>Flag]";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::NodeDef { id, label, shape } => {
                assert_eq!(id, "A");
                assert_eq!(label.as_deref(), Some("Flag"));
                assert_eq!(*shape, NodeShape::Flag);
            }
            _ => panic!("Expected NodeDef"),
        }
    }

    #[test]
    fn test_parse_rect_node_with_label() {
        let code = "graph TD\nA[Label]";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::NodeDef { id, label, shape } => {
                assert_eq!(id, "A");
                assert_eq!(label.as_deref(), Some("Label"));
                assert_eq!(*shape, NodeShape::Rect);
            }
            _ => panic!("Expected NodeDef"),
        }
    }

    #[test]
    fn test_parse_node_without_shape_syntax() {
        let code = "graph TD\nA-->B";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        // A and B have no shape syntax, so they should be Rect via EdgeDef only
        match &diagram.statements[0] {
            Statement::EdgeDef { from, to, .. } => {
                assert_eq!(from, "A");
                assert_eq!(to, "B");
            }
            _ => panic!("Expected EdgeDef"),
        }
    }

    #[test]
    fn test_parse_shape_with_edge() {
        let code = "graph TD\nA{Decision}-->B[Result]";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        // Should have: NodeDef(A, Decision, Diamond) + NodeDef(B, Result, Rect) + EdgeDef(A->B)
        assert_eq!(diagram.statements.len(), 3);
        match &diagram.statements[0] {
            Statement::NodeDef { id, shape, .. } => {
                assert_eq!(id, "A");
                assert_eq!(*shape, NodeShape::Diamond);
            }
            _ => panic!("Expected NodeDef for A"),
        }
        match &diagram.statements[1] {
            Statement::NodeDef { id, shape, .. } => {
                assert_eq!(id, "B");
                assert_eq!(*shape, NodeShape::Rect);
            }
            _ => panic!("Expected NodeDef for B"),
        }
    }
}
