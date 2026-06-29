pub mod ast;
pub mod lexer;

pub use ast::{ArrowType, ClassMemberType, ClassRelationType, ClassVisibility, Diagram, DiagramType, ErCardinality, GanttStatus, NodeShape, NotePosition, Statement, Subgraph};
pub use lexer::{Lexer, Token, TokenType};

use std::fmt;

/// Represents a parse error with location information.
#[cfg_attr(feature = "json", derive(serde::Serialize))]
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

        // 检查第一个关键字
        let diagram_type = match self.peek()?.token_type {
            TokenType::Keyword(ref k) if k == "graph" => {
                let _ = self.advance();
                DiagramType::Flowchart
            }
            TokenType::Keyword(ref k) if k == "flowchart" => {
                let _ = self.advance();
                DiagramType::Flowchart
            }
            TokenType::Keyword(ref k) if k == "sequenceDiagram" => {
                let _ = self.advance();
                DiagramType::Sequence
            }
            TokenType::Keyword(ref k) if k == "pie" => {
                let _ = self.advance();
                DiagramType::Pie
            }
            TokenType::Keyword(ref k) if k == "classDiagram" => {
                let _ = self.advance();
                DiagramType::Class
            }
            TokenType::Keyword(ref k) if k == "stateDiagram" => {
                let _ = self.advance();
                // Check for -v2 suffix
                if let Ok(token) = self.peek() {
                    if let TokenType::Identifier(ref id) = token.token_type {
                        if id == "-" {
                            let _ = self.advance();
                            // consume v2
                            if let Ok(t) = self.peek() {
                                if let TokenType::Identifier(ref v) = t.token_type {
                                    if v == "v2" {
                                        let _ = self.advance();
                                    }
                                }
                            }
                        }
                    }
                }
                DiagramType::State
            }
            TokenType::Keyword(ref k) if k == "erDiagram" => {
                let _ = self.advance();
                DiagramType::Er
            }
            TokenType::Keyword(ref k) if k == "gantt" => {
                let _ = self.advance();
                DiagramType::Gantt
            }
            TokenType::Keyword(ref k) if k == "mindmap" => {
                let _ = self.advance();
                let statements = self.parse_mindmap()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Mindmap,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "gitGraph" => {
                let _ = self.advance();
                return self.parse_gitgraph();
            }
            TokenType::Keyword(ref k) if k == "timeline" => {
                let _ = self.advance();
                let statements = self.parse_timeline()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Timeline,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "journey" => {
                let _ = self.advance();
                let statements = self.parse_journey()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Journey,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "kanban" => {
                let _ = self.advance();
                let statements = self.parse_kanban()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Kanban,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "venn" => {
                let _ = self.advance();
                let statements = self.parse_venn()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Venn,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "block" => {
                let _ = self.advance();
                let statements = self.parse_block()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Block,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "treemap" => {
                let _ = self.advance();
                let statements = self.parse_treemap()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Treemap,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "sankey" => {
                let _ = self.advance();
                let statements = self.parse_sankey()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Sankey,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "xychart" => {
                let _ = self.advance();
                let statements = self.parse_xychart()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::XyChart,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "architecture" => {
                let _ = self.advance();
                let statements = self.parse_architecture()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Architecture,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "C4Context" || k == "C4Container" || k == "C4Component" => {
                let _ = self.advance();
                let statements = self.parse_c4()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::C4,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "requirementDiagram" => {
                let _ = self.advance();
                let statements = self.parse_requirement()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Requirement,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "zenuml" => {
                let _ = self.advance();
                let statements = self.parse_sequence()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::ZenUml,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "quadrantChart" => {
                let _ = self.advance();
                let statements = self.parse_quadrant()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Quadrant,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "ishikawa" => {
                let _ = self.advance();
                let statements = self.parse_ishikawa()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Ishikawa,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "radar" => {
                let _ = self.advance();
                let statements = self.parse_radar()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Radar,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            TokenType::Keyword(ref k) if k == "packet" => {
                let _ = self.advance();
                let statements = self.parse_packet()?;
                return Ok(Diagram {
                    diagram_type: DiagramType::Packet,
                    statements,
                    direction: None,
                    subgraphs: vec![],
                    title: None,
                });
            }
            _ => {
                return Err(self.error("Expected 'graph', 'flowchart', 'sequenceDiagram', 'pie', 'classDiagram', 'stateDiagram', 'erDiagram', 'gantt', 'mindmap', 'gitgraph', 'timeline', 'journey', 'kanban', 'venn', 'packet', 'radar', 'ishikawa', 'quadrantChart', 'zenuml', or 'requirementDiagram', or 'block', or 'C4Context', or 'architecture', or 'xychart', or 'sankey', or 'treemap'"));
            }
        };

        // 序列图走独立解析路径
        if diagram_type == DiagramType::Sequence {
            let statements = self.parse_sequence()?;
            return Ok(Diagram {
                diagram_type,
                statements,
                direction: None,
                subgraphs: vec![],
                title: None,
            });
        }

        // 饼图走独立解析路径
        if diagram_type == DiagramType::Pie {
            let (title, statements) = self.parse_pie()?;
            return Ok(Diagram {
                diagram_type,
                statements,
                direction: None,
                subgraphs: vec![],
                title,
            });
        }

        // 类图走独立解析路径
        if diagram_type == DiagramType::Class {
            let statements = self.parse_class()?;
            return Ok(Diagram {
                diagram_type,
                statements,
                direction: None,
                subgraphs: vec![],
                title: None,
            });
        }

        // 状态图走独立解析路径
        if diagram_type == DiagramType::State {
            let statements = self.parse_state()?;
            return Ok(Diagram {
                diagram_type,
                statements,
                direction: None,
                subgraphs: vec![],
                title: None,
            });
        }

        // ER图走独立解析路径
        if diagram_type == DiagramType::Er {
            let statements = self.parse_er()?;
            return Ok(Diagram {
                diagram_type,
                statements,
                direction: None,
                subgraphs: vec![],
                title: None,
            });
        }

        // 甘特图走独立解析路径
        if diagram_type == DiagramType::Gantt {
            let (title, statements) = self.parse_gantt()?;
            return Ok(Diagram {
                diagram_type,
                statements,
                direction: None,
                subgraphs: vec![],
                title,
            });
        }

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
            title: None,
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

    // ============================================================
    // Sequence Diagram Parsing
    // ============================================================

    /// Parse a sequence diagram: `sequenceDiagram\n participant Alice\n Alice->Bob: msg\n ...`
    fn parse_sequence(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(stmt) = self.parse_sequence_statement()? {
                statements.push(stmt);
            }
        }

        Ok(statements)
    }

    /// Parse a single sequence diagram statement.
    fn parse_sequence_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        if self.is_at_end() {
            return Ok(None);
        }
        let token = self.peek()?.clone();

        match &token.token_type {
            // participant declaration
            TokenType::Keyword(k) if k == "participant" => {
                let _ = self.advance();
                let id = self.parse_node_id()?;
                // optional: `as Label` (no colon)
                let label = if let Ok(next) = self.peek() {
                    if let TokenType::Keyword(ref kw) = next.token_type {
                        if kw == "as" {
                            let _ = self.advance(); // skip 'as'
                            // read rest of line as label (no colon required)
                            let line = self.peek()?.line;
                            let mut text = String::new();
                            while !self.is_at_end() {
                                let t = self.peek()?;
                                if t.line != line { break; }
                                match &t.token_type {
                                    TokenType::Identifier(s) => {
                                        if !text.is_empty() { text.push(' '); }
                                        text.push_str(s);
                                        let _ = self.advance();
                                    }
                                    TokenType::Keyword(kw) => {
                                        if !text.is_empty() { text.push(' '); }
                                        text.push_str(kw);
                                        let _ = self.advance();
                                    }
                                    _ => break,
                                }
                            }
                            if text.is_empty() { None } else { Some(text) }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                Ok(Some(Statement::Participant { id, label }))
            }
            // note: Note right of Alice / Note left of Alice / Note over Alice
            TokenType::Keyword(k) if k == "Note" || k == "note" => {
                let _ = self.advance();
                let position = match self.parse_node_id()?.as_str() {
                    "right" | "Right" => {
                        // expect "of"
                        let _ = self.parse_node_id()?; // "of"
                        NotePosition::Right
                    }
                    "left" | "Left" => {
                        let _ = self.parse_node_id()?; // "of"
                        NotePosition::Left
                    }
                    "over" | "Over" => NotePosition::Over,
                    _ => return Err(self.error("Expected 'right of', 'left of', or 'over'")),
                };
                let target = self.parse_node_id()?;
                let text = self.parse_sequence_label()?;
                Ok(Some(Statement::Note {
                    target,
                    text,
                    position,
                }))
            }
            // block keywords: alt, loop, opt, par
            TokenType::Keyword(k)
                if matches!(k.as_str(), "alt" | "loop" | "opt" | "par") =>
            {
                let keyword = k.clone();
                let _ = self.advance();
                let condition = self.try_parse_sequence_rest_of_line();
                let nested = self.parse_sequence_block_body()?;
                Ok(Some(Statement::Block {
                    keyword,
                    condition,
                    statements: nested,
                }))
            }
            // 'end' keyword — signal end of block (caller handles)
            TokenType::Keyword(k) if k == "end" => {
                let _ = self.advance();
                Ok(None)
            }
            // activate/deactivate
            TokenType::Keyword(k) if k == "activate" => {
                let _ = self.advance();
                let participant = self.parse_node_id()?;
                Ok(Some(Statement::Activate { participant }))
            }
            TokenType::Keyword(k) if k == "deactivate" => {
                let _ = self.advance();
                let participant = self.parse_node_id()?;
                Ok(Some(Statement::Deactivate { participant }))
            }
            // message: Alice->Bob: message text
            TokenType::Identifier(_) => self.parse_sequence_message(),
            // skip semicolons
            TokenType::Semicolon => {
                let _ = self.advance();
                Ok(None)
            }
            _ => {
                let _ = self.advance();
                Ok(None)
            }
        }
    }

    /// Parse the body of a block (alt/loop/opt/par), handling `else` branches.
    fn parse_sequence_block_body(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut nested = Vec::new();

        while !self.is_at_end() {
            let next = self.peek()?;
            match &next.token_type {
                // 'end' closes the block — don't consume it, let the caller handle
                TokenType::Keyword(k) if k == "end" => {
                    break;
                }
                // 'else' is part of an alt block — parse as a nested block
                TokenType::Keyword(k) if k == "else" => {
                    let _ = self.advance(); // consume 'else'
                    let condition = self.try_parse_sequence_rest_of_line();
                    let else_body = self.parse_sequence_block_body()?;
                    nested.push(Statement::Block {
                        keyword: "else".to_string(),
                        condition,
                        statements: else_body,
                    });
                }
                // nested block keywords
                TokenType::Keyword(k)
                    if matches!(k.as_str(), "alt" | "loop" | "opt" | "par") =>
                {
                    if let Some(stmt) = self.parse_sequence_statement()? {
                        nested.push(stmt);
                    }
                }
                _ => {
                    if let Some(stmt) = self.parse_sequence_statement()? {
                        nested.push(stmt);
                    }
                }
            }
        }

        // consume the 'end' keyword if present
        if let Ok(next) = self.peek() {
            if let TokenType::Keyword(k) = &next.token_type {
                if k == "end" {
                    let _ = self.advance();
                }
            }
        }

        Ok(nested)
    }

    /// Parse a sequence message: `Alice->Bob: message text`
    fn parse_sequence_message(&mut self) -> Result<Option<Statement>, ParseError> {
        let from = self.parse_node_id()?;

        // parse arrow type
        let arrow = self.advance()?;
        let arrow_type = match &arrow.token_type {
            TokenType::Arrow => {
                // need to distinguish -> vs --> vs ->> vs -x etc
                // the lexer produces Arrow tokens; we check the raw text
                let raw = &self.tokens[self.current - 1].raw;
                match raw.as_str() {
                    "->" => ArrowType::Solid,
                    "-->" => ArrowType::Dashed,
                    "->>" => ArrowType::SolidCross,
                    "-->>" => ArrowType::DashedCross,
                    "-)" => ArrowType::SolidOpen,
                    "--)" => ArrowType::DashedOpen,
                    _ => ArrowType::Solid,
                }
            }
            _ => return Err(self.error("Expected arrow (->)")),
        };

        let to = self.parse_node_id()?;

        // parse label after colon
        let label = self.parse_sequence_label()?;

        Ok(Some(Statement::Message {
            from,
            to,
            label,
            arrow_type,
        }))
    }

    /// Parse label text after `:` in sequence diagram.
    fn parse_sequence_label(&mut self) -> Result<String, ParseError> {
        // expect colon
        let token = self.peek()?;
        let colon_line = token.line;
        match &token.token_type {
            TokenType::Keyword(k) if k == ":" => {
                let _ = self.advance();
            }
            _ => return Err(self.error("Expected ':'")),
        }

        // read rest of line as label text (stop at newline only)
        let mut text = String::new();
        while !self.is_at_end() {
            let token = self.peek()?;
            // stop at newline (different line from colon)
            if token.line != colon_line {
                break;
            }
            match &token.token_type {
                TokenType::Identifier(s) => {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(s);
                    let _ = self.advance();
                }
                TokenType::Keyword(k) => {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(k);
                    let _ = self.advance();
                }
                _ => break,
            }
        }

        if text.is_empty() {
            return Err(self.error("Expected message text after ':'"));
        }

        Ok(text)
    }

    /// Try to parse rest of line as condition text (for alt/loop/opt).
    /// Handles both `alt condition` and `loop: condition` syntax.
    fn try_parse_sequence_rest_of_line(&mut self) -> Option<String> {
        let start_line = if let Ok(token) = self.peek() {
            token.line
        } else {
            return None;
        };

        // optionally consume colon
        if let Ok(token) = self.peek() {
            if let TokenType::Keyword(k) = &token.token_type {
                if k == ":" {
                    let _ = self.advance();
                }
            }
        }

        let mut text = String::new();
        while !self.is_at_end() {
            if let Ok(token) = self.peek() {
                if token.line != start_line {
                    break;
                }
                match &token.token_type {
                    TokenType::Identifier(s) => {
                        if !text.is_empty() {
                            text.push(' ');
                        }
                        text.push_str(s);
                        let _ = self.advance();
                    }
                    TokenType::Keyword(k) => {
                        if matches!(
                            k.as_str(),
                            "alt" | "loop" | "opt" | "par" | "else" | "end" | "participant"
                        ) {
                            break;
                        }
                        if !text.is_empty() {
                            text.push(' ');
                        }
                        text.push_str(k);
                        let _ = self.advance();
                    }
                    _ => break,
                }
            }
        }

        if text.is_empty() {
            None
        } else {
            Some(text)
        }
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

    // ============================================================
    // Pie Chart Parsing
    // ============================================================

    /// Parse a pie chart: `pie title Title\n "label" : value\n ...`
    fn parse_pie(&mut self) -> Result<(Option<String>, Vec<Statement>), ParseError> {
        // Optional: title
        let title = if let Ok(token) = self.peek() {
            if let TokenType::Keyword(ref k) = token.token_type {
                if k == "title" {
                    let _ = self.advance(); // consume 'title'
                    // read rest of line as title
                    let title_line = self.peek()?.line;
                    let mut text = String::new();
                    while !self.is_at_end() {
                        let t = self.peek()?;
                        if t.line != title_line {
                            break;
                        }
                        match &t.token_type {
                            TokenType::Identifier(s) => {
                                if !text.is_empty() { text.push(' '); }
                                text.push_str(s);
                                let _ = self.advance();
                            }
                            TokenType::Keyword(k) => {
                                if !text.is_empty() { text.push(' '); }
                                text.push_str(k);
                                let _ = self.advance();
                            }
                            TokenType::String(s) => {
                                if !text.is_empty() { text.push(' '); }
                                text.push_str(s);
                                let _ = self.advance();
                            }
                            _ => break,
                        }
                    }
                    if text.is_empty() { None } else { Some(text) }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let mut slices = Vec::new();

        while !self.is_at_end() {
            if let Some(slice) = self.parse_pie_slice()? {
                slices.push(slice);
            }
        }

        Ok((title, slices))
    }

    /// Parse a single pie slice: `"label" : value`
    fn parse_pie_slice(&mut self) -> Result<Option<Statement>, ParseError> {
        if self.is_at_end() {
            return Ok(None);
        }

        let token = self.peek()?.clone();
        match &token.token_type {
            // Quoted label: "Dogs" : 386
            TokenType::String(label) => {
                let label = label.clone();
                let _ = self.advance();
                // expect colon
                self.expect_colon()?;
                // parse numeric value
                let value = self.parse_number()?;
                Ok(Some(Statement::PieSlice { label, value }))
            }
            // Unquoted label: Dogs : 386
            TokenType::Identifier(id) => {
                let label = id.clone();
                let _ = self.advance();
                self.expect_colon()?;
                let value = self.parse_number()?;
                Ok(Some(Statement::PieSlice { label, value }))
            }
            _ => {
                let _ = self.advance();
                Ok(None)
            }
        }
    }

    /// Expect and consume a `:` token.
    fn expect_colon(&mut self) -> Result<(), ParseError> {
        let token = self.peek()?;
        if let TokenType::Keyword(ref k) = token.token_type {
            if k == ":" {
                let _ = self.advance();
                return Ok(());
            }
        }
        Err(self.error("Expected ':'"))
    }

    /// Parse a numeric value (integer or float).
    fn parse_number(&mut self) -> Result<f64, ParseError> {
        let token = self.advance()?;
        match &token.token_type {
            TokenType::Identifier(s) => {
                s.parse::<f64>().map_err(|_| self.error("Expected numeric value"))
            }
            _ => Err(self.error("Expected numeric value")),
        }
    }

    // ============================================================
    // Class Diagram Parsing
    // ============================================================

    /// Parse a class diagram: `classDiagram\n class Animal { ... }\n Animal <|-- Dog`
    fn parse_class(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(stmts) = self.parse_class_statement()? {
                statements.extend(stmts);
            }
        }

        Ok(statements)
    }

    /// Parse a single class diagram statement.
    fn parse_class_statement(&mut self) -> Result<Option<Vec<Statement>>, ParseError> {
        if self.is_at_end() {
            return Ok(None);
        }

        let token = self.peek()?.clone();
        match &token.token_type {
            // class ClassName { ... }
            TokenType::Keyword(k) if k == "class" => {
                let _ = self.advance();
                let class_name = self.parse_node_id()?;

                // Check for stereotype: class Animal <<interface>>
                let stereotype = self.parse_optional_stereotype()?;

                let mut stmts = vec![Statement::ClassDef {
                    name: class_name.clone(),
                    stereotype,
                }];

                // Check for body: { ... }
                if let Ok(t) = self.peek() {
                    if matches!(t.token_type, TokenType::LeftBrace) {
                        let _ = self.advance(); // consume {
                        let members = self.parse_class_body(&class_name)?;
                        stmts.extend(members);
                        // consume }
                        if let Ok(t) = self.peek() {
                            if matches!(t.token_type, TokenType::RightBrace) {
                                let _ = self.advance();
                            }
                        }
                    }
                }

                Ok(Some(stmts))
            }
            // Relationship: ClassA <|-- ClassB, ClassA --> ClassB, etc.
            TokenType::Identifier(_) | TokenType::LessThan | TokenType::GreaterThan => {
                self.parse_class_relation()
            }
            // Skip semicolons
            TokenType::Semicolon => {
                let _ = self.advance();
                Ok(None)
            }
            _ => {
                let _ = self.advance();
                Ok(None)
            }
        }
    }

    /// Parse optional stereotype: <<interface>>, <<abstract>>, <<annotation>>
    fn parse_optional_stereotype(&mut self) -> Result<Option<String>, ParseError> {
        if let Ok(token) = self.peek() {
            if matches!(token.token_type, TokenType::LessThan) {
                let _ = self.advance(); // consume <
                if let Ok(token2) = self.peek() {
                    if matches!(token2.token_type, TokenType::LessThan) {
                        let _ = self.advance(); // consume second <
                        // read identifier
                        let name = self.parse_node_id()?;
                        // consume >>
                        if let Ok(t) = self.peek() {
                            if matches!(t.token_type, TokenType::GreaterThan) {
                                let _ = self.advance();
                                if let Ok(t2) = self.peek() {
                                    if matches!(t2.token_type, TokenType::GreaterThan) {
                                        let _ = self.advance();
                                    }
                                }
                            }
                        }
                        return Ok(Some(name));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Parse class body: { ... } containing attributes and methods
    fn parse_class_body(&mut self, class_name: &str) -> Result<Vec<Statement>, ParseError> {
        let mut members = Vec::new();

        while !self.is_at_end() {
            if let Ok(token) = self.peek() {
                if matches!(token.token_type, TokenType::RightBrace) {
                    break;
                }
            }

            if let Some(member) = self.parse_class_member(class_name)? {
                members.push(member);
            }
        }

        Ok(members)
    }

    /// Parse a single class member: `+String name` or `+isMammal() bool`
    fn parse_class_member(&mut self, class_name: &str) -> Result<Option<Statement>, ParseError> {
        if self.is_at_end() {
            return Ok(None);
        }

        let token = self.peek()?.clone();
        match &token.token_type {
            // Visibility modifier: +, -, #, ~
            TokenType::Identifier(s) if s == "+" || s == "-" || s == "#" || s == "~" => {
                let visibility = match s.as_str() {
                    "+" => ClassVisibility::Public,
                    "-" => ClassVisibility::Private,
                    "#" => ClassVisibility::Protected,
                    "~" => ClassVisibility::Package,
                    _ => ClassVisibility::Public,
                };
                let _ = self.advance();

                // Read type (optional) and name
                let first_ident = self.parse_node_id()?;

                // Check if it's a method: has parentheses
                if let Ok(token) = self.peek() {
                    if matches!(token.token_type, TokenType::LeftParen) {
                        let _ = self.advance(); // consume (
                        // consume )
                        if let Ok(t) = self.peek() {
                            if matches!(t.token_type, TokenType::RightParen) {
                                let _ = self.advance();
                            }
                        }
                        // Optional return type
                        let return_type = if let Ok(t) = self.peek() {
                            if let TokenType::Identifier(ref id) = t.token_type {
                                if id != "+" && id != "-" && id != "#" && id != "~" {
                                    let rt = id.clone();
                                    let _ = self.advance();
                                    Some(rt)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        return Ok(Some(Statement::ClassMember {
                            class_name: class_name.to_string(),
                            member_type: ClassMemberType::Method,
                            visibility,
                            name: first_ident,
                            type_annotation: return_type,
                        }));
                    }
                }

                // It's a field: type name (or just name if no type)
                // Check if there's another identifier (the field name)
                let (type_ann, field_name) = if let Ok(t) = self.peek() {
                    if let TokenType::Identifier(ref id) = t.token_type {
                        if id != "+" && id != "-" && id != "#" && id != "~" {
                            let fname = id.clone();
                            let _ = self.advance();
                            (Some(first_ident), fname)
                        } else {
                            (None, first_ident)
                        }
                    } else {
                        (None, first_ident)
                    }
                } else {
                    (None, first_ident)
                };

                Ok(Some(Statement::ClassMember {
                    class_name: class_name.to_string(),
                    member_type: ClassMemberType::Field,
                    visibility,
                    name: field_name,
                    type_annotation: type_ann,
                }))
            }
            _ => {
                let _ = self.advance();
                Ok(None)
            }
        }
    }

    /// Parse a class relationship: `ClassA <|-- ClassB` or `ClassA --> ClassB : label`
    fn parse_class_relation(&mut self) -> Result<Option<Vec<Statement>>, ParseError> {
        let from = self.parse_node_id()?;

        // Parse relation type
        let relation_type = self.parse_relation_arrow()?;

        let to = self.parse_node_id()?;

        // Optional label after colon
        let label = if let Ok(token) = self.peek() {
            if let TokenType::Keyword(ref k) = token.token_type {
                if k == ":" {
                    let _ = self.advance();
                    let line = self.peek()?.line;
                    let mut text = String::new();
                    while !self.is_at_end() {
                        let t = self.peek()?;
                        if t.line != line {
                            break;
                        }
                        match &t.token_type {
                            TokenType::Identifier(s) => {
                                if !text.is_empty() { text.push(' '); }
                                text.push_str(s);
                                let _ = self.advance();
                            }
                            TokenType::Keyword(k) => {
                                if !text.is_empty() { text.push(' '); }
                                text.push_str(k);
                                let _ = self.advance();
                            }
                            _ => break,
                        }
                    }
                    if text.is_empty() { None } else { Some(text) }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(Some(vec![Statement::ClassRelation {
            from,
            to,
            relation_type,
            from_cardinality: None,
            to_cardinality: None,
            label,
        }]))
    }

    /// Parse relation arrow type: <|--, *--, o--, -->, ..>, ..|>, --
    fn parse_relation_arrow(&mut self) -> Result<ClassRelationType, ParseError> {
        // Handle <|-- pattern: < is LessThan, then |-- (Arrow or Identifier)
        if let Ok(token) = self.peek() {
            if matches!(token.token_type, TokenType::LessThan) {
                let _ = self.advance(); // consume <
                // expect |-- (could be Arrow or Identifier)
                let next = self.advance()?;
                match &next.token_type {
                    TokenType::Arrow => {
                        // <|--
                        return Ok(ClassRelationType::Inheritance);
                    }
                    TokenType::Pipe => {
                        // <| then -- (Arrow)
                        if let Ok(t) = self.peek() {
                            if matches!(t.token_type, TokenType::Arrow) {
                                let _ = self.advance();
                            }
                        }
                        return Ok(ClassRelationType::Inheritance);
                    }
                    _ => return Ok(ClassRelationType::Inheritance),
                }
            }
        }

        let token = self.advance()?;
        match &token.token_type {
            TokenType::Arrow => {
                match token.raw.as_str() {
                    "-->" => Ok(ClassRelationType::Association),
                    "--" => Ok(ClassRelationType::Link),
                    _ => Ok(ClassRelationType::Association),
                }
            }
            TokenType::Identifier(s) => {
                match s.as_str() {
                    "*" => {
                        // expect -- (Arrow)
                        if let Ok(t) = self.peek() {
                            if matches!(t.token_type, TokenType::Arrow) {
                                let _ = self.advance();
                                return Ok(ClassRelationType::Composition);
                            }
                        }
                        Ok(ClassRelationType::Composition)
                    }
                    "o" => {
                        if let Ok(t) = self.peek() {
                            if matches!(t.token_type, TokenType::Arrow) {
                                let _ = self.advance();
                                return Ok(ClassRelationType::Aggregation);
                            }
                        }
                        Ok(ClassRelationType::Aggregation)
                    }
                    ".." => {
                        if let Ok(t) = self.peek() {
                            if let TokenType::Identifier(ref id) = t.token_type {
                                if id == ">" {
                                    let _ = self.advance();
                                    return Ok(ClassRelationType::Dependency);
                                }
                            }
                        }
                        Ok(ClassRelationType::Dependency)
                    }
                    _ => Ok(ClassRelationType::Association),
                }
            }
            _ => Ok(ClassRelationType::Association),
        }
    }

    // ============================================================
    // State Diagram Parsing
    // ============================================================

    /// Parse a state diagram: `stateDiagram-v2\n [*] --> Still\n Still --> Moving`
    fn parse_state(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(stmts) = self.parse_state_statement()? {
                statements.extend(stmts);
            }
        }

        Ok(statements)
    }

    /// Parse a single state diagram statement.
    fn parse_state_statement(&mut self) -> Result<Option<Vec<Statement>>, ParseError> {
        if self.is_at_end() {
            return Ok(None);
        }

        let token = self.peek()?.clone();
        match &token.token_type {
            // state StateName : label
            TokenType::Keyword(k) if k == "state" => {
                let _ = self.advance();
                let id = self.parse_node_id()?;
                let label = if let Ok(t) = self.peek() {
                    if let TokenType::Keyword(ref kw) = t.token_type {
                        if kw == ":" {
                            let _ = self.advance();
                            let line = self.peek()?.line;
                            let mut text = String::new();
                            while !self.is_at_end() {
                                let t = self.peek()?;
                                if t.line != line { break; }
                                if let TokenType::Identifier(s) = &t.token_type {
                                    if !text.is_empty() { text.push(' '); }
                                    text.push_str(s);
                                    let _ = self.advance();
                                } else {
                                    break;
                                }
                            }
                            if text.is_empty() { None } else { Some(text) }
                        } else { None }
                    } else { None }
                } else { None };
                Ok(Some(vec![Statement::StateDef { id, label }]))
            }
            // [*] --> StateName (start transition)
            TokenType::LeftBracket => {
                let _ = self.advance(); // consume [
                // expect *
                if let Ok(t) = self.peek() {
                    if let TokenType::Identifier(ref id) = t.token_type {
                        if id == "*" {
                            let _ = self.advance();
                            // expect ]
                            if let Ok(t2) = self.peek() {
                                if matches!(t2.token_type, TokenType::RightBracket) {
                                    let _ = self.advance();
                                    // expect -->
                                    self.parse_state_arrow()?;
                                    let to = self.parse_node_id()?;
                                    return Ok(Some(vec![Statement::StateTransition {
                                        from: "[*]".to_string(),
                                        to,
                                        label: None,
                                    }]));
                                }
                            }
                        }
                    }
                }
                Ok(None)
            }
            // StateName --> [*] (end transition) or StateName --> StateName
            TokenType::Identifier(_) => {
                let from = self.parse_node_id()?;
                // Check for arrow
                if let Ok(t) = self.peek() {
                    if matches!(t.token_type, TokenType::Arrow) || matches!(t.token_type, TokenType::Identifier(ref id) if id == "-") {
                        self.parse_state_arrow()?;
                        // Check if target is [*]
                        if let Ok(t2) = self.peek() {
                            if matches!(t2.token_type, TokenType::LeftBracket) {
                                let _ = self.advance();
                                if let Ok(t3) = self.peek() {
                                    if let TokenType::Identifier(ref id) = t3.token_type {
                                        if id == "*" {
                                            let _ = self.advance();
                                            if let Ok(t4) = self.peek() {
                                                if matches!(t4.token_type, TokenType::RightBracket) {
                                                    let _ = self.advance();
                                                    return Ok(Some(vec![Statement::StateTransition {
                                                        from,
                                                        to: "[*]".to_string(),
                                                        label: None,
                                                    }]));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        let to = self.parse_node_id()?;
                        // Optional label after colon
                        let label = if let Ok(t) = self.peek() {
                            if let TokenType::Keyword(ref k) = t.token_type {
                                if k == ":" {
                                    let _ = self.advance();
                                    let line = self.peek()?.line;
                                    let mut text = String::new();
                                    while !self.is_at_end() {
                                        let t = self.peek()?;
                                        if t.line != line { break; }
                                        if let TokenType::Identifier(s) = &t.token_type {
                                            if !text.is_empty() { text.push(' '); }
                                            text.push_str(s);
                                            let _ = self.advance();
                                        } else { break; }
                                    }
                                    if text.is_empty() { None } else { Some(text) }
                                } else { None }
                            } else { None }
                        } else { None };
                        return Ok(Some(vec![Statement::StateTransition { from, to, label }]));
                    }
                }
                Ok(Some(vec![Statement::StateDef { id: from, label: None }]))
            }
            TokenType::Semicolon => {
                let _ = self.advance();
                Ok(None)
            }
            _ => {
                let _ = self.advance();
                Ok(None)
            }
        }
    }

    /// Parse state transition arrow (--> or -->)
    fn parse_state_arrow(&mut self) -> Result<(), ParseError> {
        let token = self.peek()?;
        match &token.token_type {
            TokenType::Arrow => {
                let _ = self.advance();
                Ok(())
            }
            TokenType::Identifier(ref id) if id == "-" => {
                let _ = self.advance();
                // expect another - or >
                let t = self.peek()?.clone();
                if let TokenType::Identifier(ref id2) = t.token_type {
                    if id2 == "-" || id2 == ">" {
                        let _ = self.advance();
                    }
                }
                if matches!(t.token_type, TokenType::GreaterThan) {
                    let _ = self.advance();
                }
                Ok(())
            }
            _ => Err(self.error("Expected '-->'")),
        }
    }

    // ============================================================
    // ER Diagram Parsing
    // ============================================================

    /// Parse an ER diagram: `erDiagram\n CUSTOMER ||--o{ ORDER : places`
    fn parse_er(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(stmts) = self.parse_er_statement()? {
                statements.extend(stmts);
            }
        }

        Ok(statements)
    }

    /// Parse a single ER diagram statement.
    fn parse_er_statement(&mut self) -> Result<Option<Vec<Statement>>, ParseError> {
        if self.is_at_end() {
            return Ok(None);
        }

        let token = self.peek()?.clone();
        match &token.token_type {
            TokenType::Identifier(_) | TokenType::Pipe => {
                // If it starts with Pipe, it's a relationship starting with ||
                let entity = if matches!(token.token_type, TokenType::Pipe) {
                    // This is a relationship starting with ||
                    // We need to parse the cardinality first
                    // But we already consumed the first |, so we need to handle this differently
                    // Actually, let's just advance and parse normally
                    String::new() // placeholder
                } else {
                    self.parse_node_id()?
                };

                // Check if this is a relationship: ENTITY1 ||--o{ ENTITY2 : label
                // The next token should be Pipe (|) for cardinality
                let is_relationship = if matches!(token.token_type, TokenType::Pipe) {
                    true
                } else if let Ok(t) = self.peek() {
                    matches!(t.token_type, TokenType::Pipe)
                } else {
                    false
                };

                if is_relationship {
                    let (from_card, to_card) = self.parse_er_cardinality()?;
                    let to = self.parse_node_id()?;
                    let label = self.parse_er_label()?;
                    return Ok(Some(vec![Statement::ErRelation {
                        from: entity,
                        to,
                        from_cardinality: from_card,
                        to_cardinality: to_card,
                        label,
                    }]));
                }

                // Otherwise, it's an entity definition with optional attributes
                let mut stmts = vec![Statement::ErEntity { name: entity.clone() }];

                // Check for attribute block: { ... }
                if let Ok(t) = self.peek() {
                    if matches!(t.token_type, TokenType::LeftBrace) {
                        let _ = self.advance(); // consume {
                        let attrs = self.parse_er_attributes(&entity)?;
                        stmts.extend(attrs);
                        // consume }
                        if let Ok(t) = self.peek() {
                            if matches!(t.token_type, TokenType::RightBrace) {
                                let _ = self.advance();
                            }
                        }
                    }
                }

                Ok(Some(stmts))
            }
            TokenType::Semicolon => {
                let _ = self.advance();
                Ok(None)
            }
            _ => {
                let _ = self.advance();
                Ok(None)
            }
        }
    }

    /// Parse ER cardinality: ||, }|, |{, }o, o{, etc.
    /// The cardinality is split across multiple tokens by the lexer.
    /// We parse: <char1><char2> -- <char3><char4>
    fn parse_er_cardinality(&mut self) -> Result<(ErCardinality, ErCardinality), ParseError> {
        // Read first character of left cardinality
        let t1 = self.advance()?;
        let c1 = match &t1.token_type {
            TokenType::Pipe => '|',
            TokenType::Identifier(s) if s == "o" || s == "}" || s == "{" => s.chars().next().unwrap(),
            TokenType::RightBrace => '}',
            TokenType::LeftBrace => '{',
            _ => return Ok((ErCardinality::ExactlyOne, ErCardinality::ExactlyOne)),
        };

        // Read second character of left cardinality
        let t2 = self.advance()?;
        let c2 = match &t2.token_type {
            TokenType::Pipe => '|',
            TokenType::Identifier(s) if s == "o" || s == "}" || s == "{" => s.chars().next().unwrap(),
            TokenType::RightBrace => '}',
            TokenType::LeftBrace => '{',
            _ => return Ok((ErCardinality::ExactlyOne, ErCardinality::ExactlyOne)),
        };

        // Expect -- (Arrow)
        let _ = self.advance()?; // consume --

        // Read first character of right cardinality
        let t3 = self.advance()?;
        let c3 = match &t3.token_type {
            TokenType::Pipe => '|',
            TokenType::Identifier(s) if s == "o" || s == "}" || s == "{" => s.chars().next().unwrap(),
            TokenType::RightBrace => '}',
            TokenType::LeftBrace => '{',
            _ => return Ok((ErCardinality::ExactlyOne, ErCardinality::ExactlyOne)),
        };

        // Read second character of right cardinality
        let t4 = self.advance()?;
        let c4 = match &t4.token_type {
            TokenType::Pipe => '|',
            TokenType::Identifier(s) if s == "o" || s == "}" || s == "{" => s.chars().next().unwrap(),
            TokenType::RightBrace => '}',
            TokenType::LeftBrace => '{',
            _ => return Ok((ErCardinality::ExactlyOne, ErCardinality::ExactlyOne)),
        };

        let left_str = format!("{}{}", c1, c2);
        let right_str = format!("{}{}", c3, c4);

        let left_card = match left_str.as_str() {
            "||" => ErCardinality::ExactlyOne,
            "|o" | "o|" => ErCardinality::ZeroOrOne,
            "|{" | "}|" => ErCardinality::OneOrMore,
            "o{" | "}o" => ErCardinality::ZeroOrMore,
            _ => ErCardinality::ExactlyOne,
        };

        let right_card = match right_str.as_str() {
            "||" => ErCardinality::ExactlyOne,
            "|o" | "o|" => ErCardinality::ZeroOrOne,
            "|{" | "}|" => ErCardinality::OneOrMore,
            "o{" | "}o" => ErCardinality::ZeroOrMore,
            _ => ErCardinality::ExactlyOne,
        };

        Ok((left_card, right_card))
    }

    /// Parse optional label after `:` in ER relationship.
    fn parse_er_label(&mut self) -> Result<Option<String>, ParseError> {
        if let Ok(token) = self.peek() {
            if let TokenType::Keyword(ref k) = token.token_type {
                if k == ":" {
                    let _ = self.advance();
                    let line = self.peek()?.line;
                    let mut text = String::new();
                    while !self.is_at_end() {
                        let t = self.peek()?;
                        if t.line != line { break; }
                        if let TokenType::Identifier(s) = &t.token_type {
                            if !text.is_empty() { text.push(' '); }
                            text.push_str(s);
                            let _ = self.advance();
                        } else { break; }
                    }
                    return Ok(if text.is_empty() { None } else { Some(text) });
                }
            }
        }
        Ok(None)
    }

    /// Parse ER entity attributes: `{ type name PK\n ... }`
    fn parse_er_attributes(&mut self, entity: &str) -> Result<Vec<Statement>, ParseError> {
        let mut attrs = Vec::new();

        while !self.is_at_end() {
            if let Ok(token) = self.peek() {
                if matches!(token.token_type, TokenType::RightBrace) {
                    break;
                }
            }

            if let Some(attr) = self.parse_er_attribute(entity)? {
                attrs.push(attr);
            }
        }

        Ok(attrs)
    }

    /// Parse a single ER attribute: `type name PK` or `type name`
    fn parse_er_attribute(&mut self, entity: &str) -> Result<Option<Statement>, ParseError> {
        if self.is_at_end() {
            return Ok(None);
        }

        let token = self.peek()?.clone();
        match &token.token_type {
            TokenType::Identifier(type_name) => {
                let type_name = type_name.clone();
                let _ = self.advance();

                // Read attribute name
                let attr_name = if let Ok(t) = self.peek() {
                    if let TokenType::Identifier(ref id) = t.token_type {
                        let name = id.clone();
                        let _ = self.advance();
                        name
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                };

                // Check for PK marker
                let is_pk = if let Ok(t) = self.peek() {
                    if let TokenType::Identifier(ref id) = t.token_type {
                        if id == "PK" || id == "pk" {
                            let _ = self.advance();
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Check for nullable marker
                let is_null = if let Ok(t) = self.peek() {
                    if let TokenType::Identifier(ref id) = t.token_type {
                        if id == "NULL" || id == "null" {
                            let _ = self.advance();
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                Ok(Some(Statement::ErAttribute {
                    entity: entity.to_string(),
                    name: attr_name,
                    type_annotation: Some(type_name),
                    is_pk,
                    is_null,
                }))
            }
            _ => {
                let _ = self.advance();
                Ok(None)
            }
        }
    }

    // ============================================================
    // Gantt Chart Parsing
    // ============================================================

    /// Parse a Gantt chart: `gantt\n title...\n dateFormat...\n section...\n task...`
    fn parse_gantt(&mut self) -> Result<(Option<String>, Vec<Statement>), ParseError> {
        let mut statements = Vec::new();
        let mut title = None;

        while !self.is_at_end() {
            if let Some((t, stmts)) = self.parse_gantt_statement()? {
                if t.is_some() {
                    title = t;
                }
                statements.extend(stmts);
            }
        }

        Ok((title, statements))
    }

    /// Parse a single Gantt chart statement.
    #[allow(clippy::type_complexity)]
    fn parse_gantt_statement(&mut self) -> Result<Option<(Option<String>, Vec<Statement>)>, ParseError> {
        if self.is_at_end() {
            return Ok(None);
        }

        let token = self.peek()?.clone();
        match &token.token_type {
            // title
            TokenType::Keyword(k) if k == "title" => {
                let _ = self.advance();
                let line = self.peek()?.line;
                let mut text = String::new();
                while !self.is_at_end() {
                    let t = self.peek()?;
                    if t.line != line { break; }
                    if let TokenType::Identifier(s) = &t.token_type {
                        if !text.is_empty() { text.push(' '); }
                        text.push_str(s);
                        let _ = self.advance();
                    } else { break; }
                }
                Ok(Some((Some(text), vec![])))
            }
            // dateFormat (skip)
            TokenType::Keyword(k) if k == "dateFormat" => {
                let _ = self.advance();
                // skip rest of line
                let line = self.peek()?.line;
                while !self.is_at_end() {
                    let t = self.peek()?;
                    if t.line != line { break; }
                    let _ = self.advance();
                }
                Ok(Some((None, vec![])))
            }
            // section
            TokenType::Keyword(k) if k == "section" => {
                let _ = self.advance();
                let line = self.peek()?.line;
                let mut name = String::new();
                while !self.is_at_end() {
                    let t = self.peek()?;
                    if t.line != line { break; }
                    if let TokenType::Identifier(s) = &t.token_type {
                        if !name.is_empty() { name.push(' '); }
                        name.push_str(s);
                        let _ = self.advance();
                    } else { break; }
                }
                Ok(Some((None, vec![Statement::GanttSection { name }])))
            }
            // Task: name :id, start, duration
            TokenType::Identifier(_) => {
                let line = self.peek()?.line;
                let mut task_name = String::new();

                // Read task name (until colon or comma)
                while !self.is_at_end() {
                    let t = self.peek()?;
                    if t.line != line { break; }
                    match &t.token_type {
                        TokenType::Keyword(k) if k == ":" => break,
                        TokenType::Identifier(s) => {
                            if !task_name.is_empty() { task_name.push(' '); }
                            task_name.push_str(s);
                            let _ = self.advance();
                        }
                        _ => break,
                    }
                }

                let mut task_id = None;
                let mut status = None;
                let mut start = None;
                let mut duration = None;
                let mut after = None;

                // Parse optional :id, status, start, duration
                if let Ok(t) = self.peek() {
                    if let TokenType::Keyword(ref k) = t.token_type {
                        if k == ":" {
                            let _ = self.advance(); // consume :
                            // Read id or status or start
                            let first = self.read_gantt_value()?;

                            // Check if it's a status keyword
                            match first.as_str() {
                                "done" => {
                                    status = Some(GanttStatus::Done);
                                    // Read next value (could be id or start)
                                    if let Ok(v) = self.try_read_gantt_value() {
                                        task_id = Some(v);
                                    }
                                }
                                "active" => {
                                    status = Some(GanttStatus::Active);
                                    if let Ok(v) = self.try_read_gantt_value() {
                                        task_id = Some(v);
                                    }
                                }
                                "crit" => {
                                    status = Some(GanttStatus::Critical);
                                    if let Ok(v) = self.try_read_gantt_value() {
                                        task_id = Some(v);
                                    }
                                }
                                _ => {
                                    task_id = Some(first);
                                }
                            }

                            // Read start or "after" dependency
                            if let Ok(v) = self.try_read_gantt_value() {
                                if v.starts_with("after") {
                                    after = Some(v);
                                } else {
                                    start = Some(v);
                                }
                            }

                            // Read duration
                            if let Ok(v) = self.try_read_gantt_value() {
                                duration = Some(v);
                            }
                        }
                    }
                }

                Ok(Some((None, vec![Statement::GanttTask {
                    name: task_name,
                    id: task_id,
                    status,
                    start,
                    duration,
                    after,
                }])))
            }
            TokenType::Semicolon => {
                let _ = self.advance();
                Ok(None)
            }
            _ => {
                let _ = self.advance();
                Ok(None)
            }
        }
    }

    /// Read a Gantt value (comma-separated or end of line).
    fn read_gantt_value(&mut self) -> Result<String, ParseError> {
        let mut value = String::new();
        while !self.is_at_end() {
            let t = self.peek()?;
            match &t.token_type {
                TokenType::Keyword(k) if k == "," => {
                    let _ = self.advance(); // consume comma
                    break;
                }
                TokenType::Identifier(s) => {
                    if !value.is_empty() { value.push(' '); }
                    value.push_str(s);
                    let _ = self.advance();
                }
                _ => break,
            }
        }
        Ok(value)
    }

    /// Try to read a Gantt value, return error if none found.
    fn try_read_gantt_value(&mut self) -> Result<String, ParseError> {
        self.read_gantt_value()
    }

    // ============================================================
    // Mindmap Parsing
    // ============================================================

    /// Parse a mindmap: `mindmap\n  Root\n    Branch\n      Leaf`
    fn parse_mindmap(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();

        // Find the first line that contains "mindmap" keyword
        let start_idx = lines
            .iter()
            .position(|l| l.trim() == "mindmap")
            .map(|i| i + 1)
            .unwrap_or(lines.len());

        // Parse lines after the mindmap keyword
        let mut parsed_nodes: Vec<(usize, String)> = Vec::new();
        for line in &lines[start_idx..] {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let leading_spaces = line.len() - trimmed.len();
            // 2 spaces per indentation level
            let level = if leading_spaces == 0 { 0 } else { leading_spaces / 2 };
            parsed_nodes.push((level, trimmed.to_string()));
        }

        let mut node_counter: usize = 0;
        let result = Self::build_mindmap_level(
            &mut parsed_nodes.into_iter().peekable(),
            0,
            &mut node_counter,
        );
        Ok(result)
    }

    /// Recursively build mindmap nodes at a given indentation level.
    fn build_mindmap_level(
        iter: &mut std::iter::Peekable<impl Iterator<Item = (usize, String)>>,
        parent_level: usize,
        counter: &mut usize,
    ) -> Vec<Statement> {
        let mut children = Vec::new();
        while let Some(&(level, _)) = iter.peek() {
            if level <= parent_level {
                break;
            }
            let (_, label) = iter.next().unwrap();
            let id = format!("mm{}", *counter);
            *counter += 1;
            let sub_children = Self::build_mindmap_level(iter, level, counter);
            children.push(Statement::MindmapNode {
                id,
                label,
                children: sub_children,
            });
        }
        children
    }

    // ============================================================
    // GitGraph Parsing
    // ============================================================

    /// Parse gitgraph-specific statements using source text.
    fn parse_gitgraph(&mut self) -> Result<Diagram, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();

        // Find the line containing "gitGraph"
        let start_idx = lines
            .iter()
            .position(|l| l.trim() == "gitGraph")
            .map(|i| i + 1)
            .unwrap_or(lines.len());

        let mut statements = Vec::new();

        for line in &lines[start_idx..] {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, |c| c == ' ' || c == '\t').collect();
            let cmd = parts[0];
            let rest = parts.get(1).map(|s| s.trim()).unwrap_or("");

            match cmd {
                "commit" => {
                    let mut id = None;
                    let mut tag = None;

                    // Parse attributes: id:"value" tag:"value"
                    let mut pos = 0;
                    let rest_chars: Vec<char> = rest.chars().collect();
                    while pos < rest_chars.len() {
                        // Skip whitespace
                        while pos < rest_chars.len() && rest_chars[pos].is_whitespace() {
                            pos += 1;
                        }
                        if pos >= rest_chars.len() {
                            break;
                        }

                        // Read attribute name
                        let attr_start = pos;
                        while pos < rest_chars.len() && rest_chars[pos].is_alphabetic() {
                            pos += 1;
                        }
                        let attr_name: String = rest_chars[attr_start..pos].iter().collect();

                        // Expect ':'
                        if pos < rest_chars.len() && rest_chars[pos] == ':' {
                            pos += 1; // skip ':'
                        } else {
                            break;
                        }

                        // Skip whitespace after colon
                        while pos < rest_chars.len() && rest_chars[pos].is_whitespace() {
                            pos += 1;
                        }

                        // Read value (quoted or unquoted)
                        let value = if pos < rest_chars.len() && rest_chars[pos] == '"' {
                            pos += 1; // skip opening quote
                            let v_start = pos;
                            while pos < rest_chars.len() && rest_chars[pos] != '"' {
                                pos += 1;
                            }
                            let v: String = rest_chars[v_start..pos].iter().collect();
                            if pos < rest_chars.len() {
                                pos += 1; // skip closing quote
                            }
                            v
                        } else {
                            let v_start = pos;
                            while pos < rest_chars.len() && !rest_chars[pos].is_whitespace() {
                                pos += 1;
                            }
                            rest_chars[v_start..pos].iter().collect()
                        };

                        match attr_name.as_str() {
                            "id" => id = Some(value),
                            "tag" => tag = Some(value),
                            _ => {}
                        }
                    }

                    statements.push(Statement::GitCommit { id, tag, branch: None });
                }
                "branch" => {
                    statements.push(Statement::GitBranch { name: rest.to_string() });
                }
                "checkout" => {
                    statements.push(Statement::GitCheckout { name: rest.to_string() });
                }
                "merge" => {
                    let merge_parts: Vec<&str> = rest.splitn(3, |c: char| c.is_whitespace()).collect();
                    let branch_name = merge_parts[0].to_string();
                    let mut tag = None;

                    if merge_parts.len() >= 3 && merge_parts[1] == "tag" {
                        let tag_val = merge_parts[2].trim_matches('"');
                        tag = Some(tag_val.to_string());
                    }

                    statements.push(Statement::GitMerge {
                        branch: branch_name,
                        tag,
                    });
                }
                _ => {}
            }
        }

        Ok(Diagram {
            diagram_type: DiagramType::GitGraph,
            statements,
            direction: None,
            subgraphs: vec![],
            title: None,
        })
    }

    // ============================================================
    // Timeline Parsing
    // ============================================================

    /// Parse timeline diagram: `timeline\ntitle <text>\nsection <name>\n<time> : <desc>`
    fn parse_timeline(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();

        let start_idx = lines
            .iter()
            .position(|l| l.trim() == "timeline")
            .map(|i| i + 1)
            .unwrap_or(lines.len());

        let mut statements = Vec::new();
        for line in &lines[start_idx..] {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("title ") {
                continue; // Title is handled by Diagram.title in parser dispatch
            } else if trimmed.starts_with("section ") {
                let name = trimmed["section ".len()..].trim().to_string();
                statements.push(Statement::TimelineSection { name });
            } else if let Some(pos) = trimmed.find(':') {
                let time = trimmed[..pos].trim().to_string();
                let description = trimmed[pos + 1..].trim().to_string();
                statements.push(Statement::TimelineEvent { time, description });
            }
        }

        Ok(statements)
    }

    // ============================================================
    // Journey Parsing
    // ============================================================

    /// Parse journey diagram: `journey\ntitle <t>\nsection <n>\n<task>:<score>:<actor1>,<actor2>`
    fn parse_journey(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();

        let start_idx = lines
            .iter()
            .position(|l| l.trim() == "journey")
            .map(|i| i + 1)
            .unwrap_or(lines.len());

        let mut statements = Vec::new();
        for line in &lines[start_idx..] {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("title ") {
                continue; // Title handled by parser dispatch
            } else if trimmed.starts_with("section ") {
                let name = trimmed["section ".len()..].trim().to_string();
                statements.push(Statement::JourneySection { name });
            } else if let Some(pos1) = trimmed.find(':') {
                let rest = &trimmed[pos1 + 1..];
                let name = trimmed[..pos1].trim().to_string();
                if let Some(pos2) = rest.find(':') {
                    let score_str = rest[..pos2].trim().to_string();
                    let score: f64 = score_str.parse().unwrap_or(0.0);
                    let actors_str = rest[pos2 + 1..].trim();
                    let actors: Vec<String> = actors_str
                        .split(',')
                        .map(|a| a.trim().to_string())
                        .filter(|a| !a.is_empty())
                        .collect();
                    statements.push(Statement::JourneyTask { name, score, actors });
                }
            }
        }

        Ok(statements)
    }

    // ============================================================
    // Kanban Parsing
    // ============================================================

    /// Parse kanban diagram: columns and tasks with [brackets]
    fn parse_kanban(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();
        let start_idx = lines.iter()
            .position(|l| l.trim() == "kanban")
            .map(|i| i + 1)
            .unwrap_or(lines.len());

        let mut statements = Vec::new();
        for line in &lines[start_idx..] {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(end_bracket) = trimmed.find(']') {
                // Task: [Name] description
                let name = trimmed[1..end_bracket].trim().to_string();
                let desc = trimmed[end_bracket + 1..].trim().to_string();
                let description = if desc.is_empty() { None } else { Some(desc) };
                statements.push(Statement::KanbanTask { name, description });
            } else {
                // Column name (no brackets)
                statements.push(Statement::KanbanColumn { name: trimmed.to_string() });
            }
        }

        Ok(statements)
    }

    // ============================================================
    // Venn Parsing
    // ============================================================

    /// Parse venn diagram: \`venn\n  a : Cats\n  b : Dogs\n  ab : Both\`
    fn parse_venn(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();
        let start_idx = lines.iter()
            .position(|l| l.trim() == "venn")
            .map(|i| i + 1)
            .unwrap_or(lines.len());

        let mut statements = Vec::new();
        for line in &lines[start_idx..] {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            if let Some(pos) = trimmed.find(':') {
                let id = trimmed[..pos].trim().to_string();
                let label = trimmed[pos + 1..].trim().to_string();
                statements.push(Statement::VennSet { id, label });
            }
        }
        Ok(statements)
    }

    // ============================================================
    // Packet Parsing
    // ============================================================

    /// Parse packet diagram: \`packet\n  0-7: Source Port\n  8-15: Dest Port\`
    fn parse_packet(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();
        let start_idx = lines.iter()
            .position(|l| l.trim() == "packet" || l.trim().starts_with("packet"))
            .map(|i| i + 1)
            .unwrap_or(lines.len());

        let mut statements = Vec::new();
        for line in &lines[start_idx..] {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            if let Some(colon_pos) = trimmed.find(':') {
                let range_part = trimmed[..colon_pos].trim();
                let label = trimmed[colon_pos + 1..].trim().to_string();
                if let Some(dash_pos) = range_part.find('-') {
                    let start_str = range_part[..dash_pos].trim();
                    let end_str = range_part[dash_pos + 1..].trim();
                    if let (Ok(start), Ok(end)) = (start_str.parse::<u32>(), end_str.parse::<u32>()) {
                        statements.push(Statement::PacketField { start_bit: start, end_bit: end, label });
                    }
                }
            }
        }
        Ok(statements)
    }

    // ============================================================
    // Radar Parsing
    // ============================================================

    /// Parse radar chart: \`radar\n  Speed: 80\n  Power: 65\`
    fn parse_radar(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();
        let start_idx = lines.iter()
            .position(|l| l.trim() == "radar")
            .map(|i| i + 1)
            .unwrap_or(lines.len());

        let mut statements = Vec::new();
        for line in &lines[start_idx..] {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            if let Some(pos) = trimmed.find(':') {
                let label = trimmed[..pos].trim().to_string();
                let val_str = trimmed[pos + 1..].trim();
                let value: f64 = val_str.parse().unwrap_or(0.0);
                statements.push(Statement::RadarAxis { label, value });
            }
        }
        Ok(statements)
    }

    // ============================================================
    // Ishikawa Parsing
    // ============================================================

    /// Parse ishikawa diagram: \`ishikawa\n  root P\n  category C\n    cause1\`
    fn parse_ishikawa(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();
        let start_idx = lines.iter()
            .position(|l| l.trim() == "ishikawa")
            .map(|i| i + 1)
            .unwrap_or(lines.len());

        let mut statements = Vec::new();
        let mut in_category = false;

        for line in &lines[start_idx..] {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("title") { continue; }
            if trimmed.starts_with("root ") {
                statements.push(Statement::IshikawaRoot { label: trimmed[5..].trim().to_string() });
                in_category = false;
            } else if trimmed.starts_with("category ") {
                statements.push(Statement::IshikawaCategory { label: trimmed[9..].trim().to_string() });
                in_category = true;
            } else if in_category && !trimmed.is_empty() {
                statements.push(Statement::IshikawaCause { label: trimmed.to_string() });
            }
        }
        Ok(statements)
    }

    // ============================================================
    // Quadrant Chart Parsing
    // ============================================================

    /// Parse quadrant chart: \`quadrantChart\ntitle ...\nx-axis ...\ny-axis ...\nquadrant-1 ...\nPoint: [0.3, 0.6]\`
    fn parse_quadrant(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();
        let start_idx = lines.iter()
            .position(|l| l.trim() == "quadrantChart")
            .map(|i| i + 1)
            .unwrap_or(lines.len());

        let mut statements = Vec::new();
        for line in &lines[start_idx..] {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            if let Some(title) = trimmed.strip_prefix("title ") {
                statements.push(Statement::QuadrantTitle(title.trim().to_string()));
            } else if let Some(x_axis) = trimmed.strip_prefix("x-axis ") {
                statements.push(Statement::QuadrantXAxis(x_axis.trim().to_string()));
            } else if let Some(y_axis) = trimmed.strip_prefix("y-axis ") {
                statements.push(Statement::QuadrantYAxis(y_axis.trim().to_string()));
            } else if let Some(rest) = trimmed.strip_prefix("quadrant-") {
                let parts: Vec<&str> = rest.splitn(2, |c: char| c.is_whitespace()).collect();
                if let Ok(num) = parts[0].parse::<u32>() {
                    let label = if parts.len() > 1 { parts[1].trim().to_string() } else { String::new() };
                    statements.push(Statement::QuadrantLabel { quadrant: num, label });
                }
            } else if let Some(bracket_pos) = trimmed.find('[') {
                let label = trimmed[..bracket_pos].trim().trim_end_matches(':').to_string();
                let coords = trimmed[bracket_pos..].trim_matches('[').trim_matches(']');
                if let Some(comma_pos) = coords.find(',') {
                    let x = coords[..comma_pos].trim().parse::<f64>().unwrap_or(0.5);
                    let y = coords[comma_pos + 1..].trim().parse::<f64>().unwrap_or(0.5);
                    statements.push(Statement::QuadrantPoint { label, x, y });
                }
            }
        }
        Ok(statements)
    }

    // ============================================================
    // Requirement Diagram Parsing
    // ============================================================

    /// Parse requirement diagram with blocks and relations
    fn parse_requirement(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();
        let start = lines.iter()
            .position(|l| l.trim() == "requirementDiagram")
            .map(|i| i + 1)
            .unwrap_or(lines.len());

        let mut stmts = Vec::new();
        let mut cur_name = String::new();
        let mut cur_id = String::new();
        let mut cur_text = String::new();
        let mut cur_risk = String::new();
        let mut cur_vm = String::new();
        let mut in_block = false;
        let mut is_req = false;

        for line in &lines[start..] {
            let t = line.trim();
            if t.is_empty() { continue; }

            if t == "}" && in_block {
                if is_req {
                    stmts.push(Statement::RequirementDef {
                        name: cur_name.clone(),
                        req_id: cur_id.clone(),
                        text: cur_text.clone(),
                        risk: cur_risk.clone(),
                        verify_method: cur_vm.clone(),
                    });
                }
                in_block = false;
                continue;
            }

            if t.starts_with("requirement ") {
                in_block = true; is_req = true;
                cur_name = t["requirement ".len()..].trim_end_matches('{').trim().to_string();
                cur_id.clear(); cur_text.clear(); cur_risk.clear(); cur_vm.clear();
                continue;
            }
            if t.starts_with("element ") {
                in_block = true; is_req = false;
                let name = t["element ".len()..].trim_end_matches('{').trim().to_string();
                // element type is inside the block, so defer creation
                cur_name = name;
                continue;
            }

            if in_block {
                if let Some(pos) = t.find(':') {
                    let key = t[..pos].trim();
                    let val = t[pos+1..].trim().to_string();
                    if is_req {
                        match key {
                            "id" => cur_id = val,
                            "text" => cur_text = val,
                            "risk" => cur_risk = val,
                            "verifymethod" => cur_vm = val,
                            _ => {}
                        }
                    } else {
                        if key == "type" {
                            stmts.push(Statement::RequirementElement {
                                name: cur_name.clone(),
                                element_type: val,
                            });
                        }
                    }
                }
                continue;
            }

            // Relation: X - type -> Y
            if t.contains("->") {
                let arrow_pos = t.find("->").unwrap();
                let before = t[..arrow_pos].trim();
                let after = t[arrow_pos+2..].trim();
                // before is "X - type" or "X -  type"
                let before_parts: Vec<&str> = before.splitn(2, |c: char| c.is_whitespace()).collect();
                let from = before_parts[0].to_string();
                let rel_type = if before_parts.len() > 1 {
                    before_parts[1].trim_matches('-').trim().to_string()
                } else {
                    String::new()
                };
                let to = after.trim_start_matches('>').trim().to_string();
                stmts.push(Statement::RequirementRelation { from, to, relation_type: rel_type });
            }
        }

        // Flush if closing brace missing
        if in_block && is_req {
            stmts.push(Statement::RequirementDef {
                name: cur_name, req_id: cur_id, text: cur_text,
                risk: cur_risk, verify_method: cur_vm,
            });
        }

        Ok(stmts)
    }

    // ============================================================
    // Block Diagram Parsing
    // ============================================================

    /// Parse block diagram: indentation-based hierarchy like mindmap
    fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();
        let start_idx = lines.iter()
            .position(|l| l.trim() == "block")
            .map(|i| i + 1)
            .unwrap_or(lines.len());

        let mut nodes: Vec<(usize, String)> = Vec::new();
        for line in &lines[start_idx..] {
            let t = line.trim();
            if t.is_empty() { continue; }
            let level = (line.len() - t.len()) / 2;
            nodes.push((level, t.to_string()));
        }

        let mut counter = 0usize;
        Ok(Self::build_block_level(&mut nodes.into_iter().peekable(), 0, &mut counter))
    }

    fn build_block_level(
        iter: &mut std::iter::Peekable<impl Iterator<Item = (usize, String)>>,
        parent_level: usize,
        counter: &mut usize,
    ) -> Vec<Statement> {
        let mut children = Vec::new();
        while let Some(&(level, _)) = iter.peek() {
            if level <= parent_level { break; }
            let (_, label) = iter.next().unwrap();
            let id = format!("b{}", *counter);
            *counter += 1;
            let sub = Self::build_block_level(iter, level, counter);
            children.push(Statement::BlockNode { id, label, children: sub });
        }
        children
    }

    // ============================================================
    // C4 Parsing
    // ============================================================

    /// Parse C4 diagram: elements with parenthesized arg tuples
    fn parse_c4(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let mut stmts = Vec::new();

        for line in source.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with("C4Context") || t.starts_with("C4Container") || t.starts_with("C4Component") { continue; }
            if t.starts_with("System_Boundary") || t.starts_with("Enterprise_Boundary") { continue; }

            // Element definitions: Type(alias, "label", "desc")
            let result = if let Some(inner) = t.strip_prefix("Person(") {
                parse_c4_triple(inner).map(|(a, l, d)| Statement::C4Person { alias: a, label: l, description: d })
            } else if let Some(inner) = t.strip_prefix("System(") {
                parse_c4_triple(inner).map(|(a, l, d)| Statement::C4System { alias: a, label: l, description: d })
            } else if let Some(inner) = t.strip_prefix("Container(") {
                parse_c4_triple(inner).map(|(a, l, d)| Statement::C4Container { alias: a, label: l, description: d })
            } else if let Some(inner) = t.strip_prefix("Component(") {
                parse_c4_triple(inner).map(|(a, l, d)| Statement::C4Component { alias: a, label: l, description: d })
            } else if t.starts_with("Rel(") || t.starts_with("Rel_Up(") || t.starts_with("BiRel(") {
                let inner = t.strip_prefix("Rel(").or_else(|| t.strip_prefix("Rel_Up(")).or_else(|| t.strip_prefix("BiRel(")).unwrap_or("");
                parse_c4_rel_triple(inner).map(|(f, t, l)| Statement::C4Rel { from: f, to: t, label: l })
            } else { None };

            if let Some(stmt) = result {
                stmts.push(stmt);
            }
        }
        Ok(stmts)
    }

    // ============================================================
    // Architecture Diagram Parsing
    // ============================================================

    /// Parse architecture diagram: services, databases, queues, and relations
    fn parse_architecture(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();
        let start = lines.iter().position(|l| l.trim() == "architecture").map(|i| i+1).unwrap_or(lines.len());

        let mut stmts = Vec::new();
        for line in &lines[start..] {
            let t = line.trim();
            if t.is_empty() { continue; }
            if t.starts_with("service ") {
                let rest = t["service ".len()..].trim();
                if let Some(paren) = rest.find('(') {
                    let id = rest[..paren].trim().to_string();
                    let label = rest[paren+1..].trim_end_matches(')').trim().to_string();
                    stmts.push(Statement::ArchService { id, label });
                }
            } else if t.starts_with("database ") {
                let rest = t["database ".len()..].trim();
                if let Some(paren) = rest.find('[') {
                    let id = rest[..paren].trim().to_string();
                    let inner = rest[paren..].trim();
                    let label = inner.trim_start_matches('[').trim_start_matches('(').trim_end_matches(']').trim_end_matches(')').trim().to_string();
                    stmts.push(Statement::ArchDatabase { id, label });
                }
            } else if t.starts_with("queue ") {
                let rest = t["queue ".len()..].trim();
                if let Some(paren) = rest.find('(') {
                    let id = rest[..paren].trim().to_string();
                    let label = rest[paren+1..].trim_end_matches(')').trim().to_string();
                    stmts.push(Statement::ArchQueue { id, label });
                }
            } else if t.contains("->") {
                let parts: Vec<&str> = t.splitn(2, "->").collect();
                if parts.len() == 2 {
                    stmts.push(Statement::ArchRelation { from: parts[0].trim().to_string(), to: parts[1].trim().to_string() });
                }
            }
        }
        Ok(stmts)
    }

    // ============================================================
    // XY Chart Parsing
    // ============================================================

    /// Parse XY chart: \`xychart-beta\ntitle "X"\nx-axis "Y" [...]\nbar [...]\nline [...]\`
    fn parse_xychart(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();
        let start = lines.iter().position(|l| l.trim() == "xychart" || l.trim().starts_with("xychart")).map(|i| i+1).unwrap_or(lines.len());

        let mut stmts = Vec::new();
        for line in &lines[start..] {
            let t = line.trim();
            if t.is_empty() { continue; }
            if let Some(title) = t.strip_prefix("title ") {
                stmts.push(Statement::XyTitle(title.trim().trim_matches('"').to_string()));
            } else if let Some(rest) = t.strip_prefix("x-axis ") {
                let r = rest.trim();
                let label = r.trim_matches('"').to_string();
                let cats = if r.contains('[') {
                    r.split('[').nth(1).and_then(|s| s.split(']').next())
                        .map(|s| s.split(',').map(|c| c.trim().trim_matches('"').to_string()).collect())
                        .unwrap_or_default()
                } else { vec![] };
                stmts.push(Statement::XyXAxis { label, categories: cats });
            } else if let Some(rest) = t.strip_prefix("y-axis ") {
                let r = rest.trim();
                let label = r.trim_matches('"').to_string();
                let (min, max) = if r.contains("-->") {
                    let parts: Vec<&str> = r.split("-->").collect();
                    (parts[0].trim().parse::<f64>().unwrap_or(0.0), parts[1].trim().parse::<f64>().unwrap_or(100.0))
                } else { (0.0, 100.0) };
                stmts.push(Statement::XyYAxis { label, min, max });
            } else if let Some(rest) = t.strip_prefix("bar ") {
                let data = rest.trim_matches('[').trim_matches(']').split(',').filter_map(|v| v.trim().parse::<f64>().ok()).collect();
                stmts.push(Statement::XyBar { data });
            } else if let Some(rest) = t.strip_prefix("line ") {
                let data = rest.trim_matches('[').trim_matches(']').split(',').filter_map(|v| v.trim().parse::<f64>().ok()).collect();
                stmts.push(Statement::XyLine { data });
            }
        }
        Ok(stmts)
    }

    // ============================================================
    // Sankey Diagram Parsing
    // ============================================================

    fn parse_sankey(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();
        let start = lines.iter().position(|l| l.trim() == "sankey").map(|i| i+1).unwrap_or(lines.len());

        let mut stmts = Vec::new();
        for line in &lines[start..] {
            let t = line.trim();
            if t.is_empty() { continue; }
            if let Some(arrow) = t.find("->") {
                let source = t[..arrow].trim().to_string();
                let rest = t[arrow+2..].trim();
                if let Some(col) = rest.find(':') {
                    let target = rest[..col].trim().to_string();
                    let val = rest[col+1..].trim().parse::<f64>().unwrap_or(0.0);
                    stmts.push(Statement::SankeyLink { source, target, value: val });
                }
            }
        }
        Ok(stmts)
    }

    fn parse_treemap(&mut self) -> Result<Vec<Statement>, ParseError> {
        let source = &self._source;
        let lines: Vec<&str> = source.lines().collect();
        let start = lines.iter().position(|l| l.trim() == "treemap").map(|i| i+1).unwrap_or(lines.len());
        let mut stmts = Vec::new();
        for line in &lines[start..] {
            let t = line.trim();
            if t.is_empty() { continue; }
            if let Some(col) = t.find(':') {
                let label = t[..col].trim().to_string();
                let val = t[col+1..].trim().parse::<f64>().unwrap_or(0.0);
                stmts.push(Statement::TreemapItem { label, value: val });
            }
        }
        Ok(stmts)
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

// C4 helper functions

/// Parse three comma-separated arguments inside parens for a C4 element.
fn parse_c4_triple(inner: &str) -> Option<(String, String, String)> {
    let inner = inner.trim_end().strip_suffix(')')?;
    let args = parse_c4_args(inner);
    if args.len() == 3 { Some((args[0].clone(), args[1].clone(), args[2].clone())) } else { None }
}

/// Parse three comma-separated arguments for a C4 relationship.
fn parse_c4_rel_triple(inner: &str) -> Option<(String, String, String)> {
    parse_c4_triple(inner)
}

/// Parse comma-separated arguments respecting quoted strings.
fn parse_c4_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in input.trim().chars() {
        if in_quotes {
            if ch == '"' { in_quotes = false; } else { current.push(ch); }
        } else if ch == '"' { current.clear(); in_quotes = true; }
        else if ch == ',' { args.push(current.trim().to_string()); current = String::new(); }
        else { current.push(ch); }
    }
    args.push(current.trim().to_string());
    args
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

    // ---- 序列图解析测试 ----

    #[test]
    fn test_parse_sequence_keyword() {
        let code = "sequenceDiagram\n    Alice->Bob: Hello";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Sequence);
    }

    #[test]
    fn test_parse_sequence_participant() {
        let code = "sequenceDiagram\n    participant Alice\n    participant Bob";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 2);
        match &diagram.statements[0] {
            Statement::Participant { id, label } => {
                assert_eq!(id, "Alice");
                assert!(label.is_none());
            }
            _ => panic!("Expected Participant"),
        }
    }

    #[test]
    fn test_parse_sequence_participant_alias() {
        let code = "sequenceDiagram\n    participant A as Alice";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::Participant { id, label } => {
                assert_eq!(id, "A");
                assert_eq!(label.as_deref(), Some("Alice"));
            }
            _ => panic!("Expected Participant"),
        }
    }

    #[test]
    fn test_parse_sequence_message() {
        let code = "sequenceDiagram\n    Alice->Bob: Hello Bob";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 1);
        match &diagram.statements[0] {
            Statement::Message { from, to, label, arrow_type } => {
                assert_eq!(from, "Alice");
                assert_eq!(to, "Bob");
                assert_eq!(label, "Hello Bob");
                assert_eq!(*arrow_type, ArrowType::Solid);
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_parse_sequence_arrow_types() {
        let code = "sequenceDiagram\n    A->B: solid\n    A-->B: dashed\n    A->>B: cross\n    A-->>B: dashed cross\n    A-)B: open\n    A--)B: dashed open";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        let expected = [
            ArrowType::Solid, ArrowType::Dashed, ArrowType::SolidCross,
            ArrowType::DashedCross, ArrowType::SolidOpen, ArrowType::DashedOpen,
        ];
        for (i, exp) in expected.iter().enumerate() {
            match &diagram.statements[i] {
                Statement::Message { arrow_type, .. } => assert_eq!(arrow_type, exp),
                _ => panic!("Expected Message at index {}", i),
            }
        }
    }

    #[test]
    fn test_parse_sequence_auto_participant() {
        // participants auto-discovered from messages
        let code = "sequenceDiagram\n    Alice->Bob: Hello";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 1);
        // no explicit participant, but message references Alice and Bob
        match &diagram.statements[0] {
            Statement::Message { from, to, .. } => {
                assert_eq!(from, "Alice");
                assert_eq!(to, "Bob");
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_parse_sequence_alt_block() {
        let code = "sequenceDiagram\n    alt success\n        Alice->Bob: OK\n    else failure\n        Alice->Bob: Error\n    end";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 1);
        match &diagram.statements[0] {
            Statement::Block { keyword, condition, statements } => {
                assert_eq!(keyword, "alt");
                assert_eq!(condition.as_deref(), Some("success"));
                // should contain: Message + else Block
                assert_eq!(statements.len(), 2);
                assert!(matches!(&statements[0], Statement::Message { .. }));
                assert!(matches!(&statements[1], Statement::Block { keyword, .. } if keyword == "else"));
            }
            _ => panic!("Expected Block"),
        }
    }

    #[test]
    fn test_parse_sequence_loop_block() {
        let code = "sequenceDiagram\n    loop retry\n        Alice->Bob: Try\n    end";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::Block { keyword, condition, statements } => {
                assert_eq!(keyword, "loop");
                assert_eq!(condition.as_deref(), Some("retry"));
                assert_eq!(statements.len(), 1);
            }
            _ => panic!("Expected Block"),
        }
    }

    #[test]
    fn test_parse_sequence_opt_block() {
        let code = "sequenceDiagram\n    opt if needed\n        Alice->Bob: Maybe\n    end";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::Block { keyword, condition, .. } => {
                assert_eq!(keyword, "opt");
                assert_eq!(condition.as_deref(), Some("if needed"));
            }
            _ => panic!("Expected Block"),
        }
    }

    #[test]
    fn test_parse_sequence_note() {
        let code = "sequenceDiagram\n    Note right of Alice: This is a note";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::Note { target, text, position } => {
                assert_eq!(target, "Alice");
                assert_eq!(text, "This is a note");
                assert_eq!(*position, NotePosition::Right);
            }
            _ => panic!("Expected Note"),
        }
    }

    #[test]
    fn test_parse_sequence_multiline_messages() {
        let code = "sequenceDiagram\n    A->B: msg1\n    B->C: msg2\n    C->A: msg3";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 3);
    }

    #[test]
    fn test_parse_sequence_question_mark_in_label() {
        let code = "sequenceDiagram\n    A->B: How are you?\n    B->A: Fine";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 2);
        match &diagram.statements[0] {
            Statement::Message { label, .. } => assert_eq!(label, "How are you"),
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_parse_sequence_activate() {
        let code = "sequenceDiagram\nactivate Alice\nAlice->Bob: Hello\ndeactivate Alice";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 3);
        match &diagram.statements[0] {
            Statement::Activate { participant } => assert_eq!(participant, "Alice"),
            _ => panic!("Expected Activate"),
        }
        match &diagram.statements[2] {
            Statement::Deactivate { participant } => assert_eq!(participant, "Alice"),
            _ => panic!("Expected Deactivate"),
        }
    }

    #[test]
    fn test_parse_sequence_note_left() {
        let code = "sequenceDiagram\nNote left of Alice: A note";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::Note { target, text, position } => {
                assert_eq!(target, "Alice");
                assert_eq!(text, "A note");
                assert_eq!(*position, NotePosition::Left);
            }
            _ => panic!("Expected Note"),
        }
    }

    #[test]
    fn test_parse_sequence_note_over() {
        let code = "sequenceDiagram\nNote over Alice: Over both";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::Note { target, text, position } => {
                assert_eq!(target, "Alice");
                assert_eq!(text, "Over both");
                assert_eq!(*position, NotePosition::Over);
            }
            _ => panic!("Expected Note"),
        }
    }

    // ---- Pie Chart 解析测试 ----

    #[test]
    fn test_parse_pie_basic() {
        let code = "pie title Pets\n\"Dogs\" : 386\n\"Cats\" : 85\n\"Rats\" : 15";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Pie);
        assert_eq!(diagram.title.as_deref(), Some("Pets"));
        assert_eq!(diagram.statements.len(), 3);
    }

    #[test]
    fn test_parse_pie_slice_values() {
        let code = "pie title Pets\n\"Dogs\" : 386\n\"Cats\" : 85";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::PieSlice { label, value } => {
                assert_eq!(label, "Dogs");
                assert_eq!(*value, 386.0);
            }
            _ => panic!("Expected PieSlice"),
        }
        match &diagram.statements[1] {
            Statement::PieSlice { label, value } => {
                assert_eq!(label, "Cats");
                assert_eq!(*value, 85.0);
            }
            _ => panic!("Expected PieSlice"),
        }
    }

    #[test]
    fn test_parse_pie_no_title() {
        let code = "pie\n\"A\" : 50\n\"B\" : 50";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Pie);
        assert!(diagram.title.is_none());
        assert_eq!(diagram.statements.len(), 2);
    }

    #[test]
    fn test_parse_pie_unquoted_labels() {
        let code = "pie title Test\nDogs : 100\nCats : 200";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 2);
        match &diagram.statements[0] {
            Statement::PieSlice { label, value } => {
                assert_eq!(label, "Dogs");
                assert_eq!(*value, 100.0);
            }
            _ => panic!("Expected PieSlice"),
        }
    }

    #[test]
    fn test_parse_pie_float_values() {
        let code = "pie\n\"A\" : 33.5\n\"B\" : 66.5";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::PieSlice { value, .. } => assert_eq!(*value, 33.5),
            _ => panic!("Expected PieSlice"),
        }
    }

    // ---- Class Diagram 解析测试 ----

    #[test]
    fn test_parse_class_basic() {
        let code = "classDiagram\nclass Animal";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Class);
        assert_eq!(diagram.statements.len(), 1);
        match &diagram.statements[0] {
            Statement::ClassDef { name, .. } => assert_eq!(name, "Animal"),
            _ => panic!("Expected ClassDef"),
        }
    }

    #[test]
    fn test_parse_class_with_members() {
        let code = "classDiagram\nclass Animal {\n+String name\n+int age\n+isMammal() bool\n}";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        // Should have ClassDef + 3 members
        assert!(diagram.statements.len() >= 4, "Expected at least 4 statements, got {}", diagram.statements.len());
    }

    #[test]
    fn test_parse_class_relationship() {
        let code = "classDiagram\nAnimal <|-- Dog";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        let rel = diagram.statements.iter().find(|s| matches!(s, Statement::ClassRelation { .. }));
        assert!(rel.is_some(), "Expected a ClassRelation");
        match rel.unwrap() {
            Statement::ClassRelation { from, to, relation_type, .. } => {
                assert_eq!(from, "Animal");
                assert_eq!(to, "Dog");
                assert_eq!(*relation_type, ClassRelationType::Inheritance);
            }
            _ => panic!("Expected ClassRelation"),
        }
    }

    #[test]
    fn test_parse_class_association() {
        let code = "classDiagram\nAnimal --> Food : eats";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        let rel = diagram.statements.iter().find(|s| matches!(s, Statement::ClassRelation { .. }));
        assert!(rel.is_some());
        match rel.unwrap() {
            Statement::ClassRelation { from, to, label, .. } => {
                assert_eq!(from, "Animal");
                assert_eq!(to, "Food");
                assert_eq!(label.as_deref(), Some("eats"));
            }
            _ => panic!("Expected ClassRelation"),
        }
    }

    #[test]
    fn test_parse_class_visibility() {
        let code = "classDiagram\nclass Animal {\n+String name\n-int age\n#isMammal() bool\n}";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        // Check that we have members with different visibilities
        let members: Vec<_> = diagram.statements.iter().filter(|s| matches!(s, Statement::ClassMember { .. })).collect();
        assert!(members.len() >= 3, "Expected at least 3 members");
    }

    // ---- State Diagram 解析测试 ----

    #[test]
    fn test_parse_state_basic() {
        let code = "stateDiagram-v2\n[*] --> Still\nStill --> [*]";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::State);
        assert_eq!(diagram.statements.len(), 2);
    }

    #[test]
    fn test_parse_state_transitions() {
        let code = "stateDiagram-v2\n[*] --> Still\nStill --> Moving\nMoving --> Still";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 3);
        match &diagram.statements[0] {
            Statement::StateTransition { from, to, .. } => {
                assert_eq!(from, "[*]");
                assert_eq!(to, "Still");
            }
            _ => panic!("Expected StateTransition"),
        }
    }

    #[test]
    fn test_parse_state_with_label() {
        let code = "stateDiagram-v2\nstate Moving : This is moving";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        match &diagram.statements[0] {
            Statement::StateDef { id, label } => {
                assert_eq!(id, "Moving");
                assert_eq!(label.as_deref(), Some("This is moving"));
            }
            _ => panic!("Expected StateDef"),
        }
    }

    // ---- Parser edge case 测试 ----

    #[test]
    fn test_parse_flowchart_with_mixed_semicolons() {
        let codes = [
            "graph TD; A-->B",
            "graph TD A-->B",
            "graph TD;\nA-->B",
            "graph TD\nA-->B",
        ];
        for code in &codes {
            let mut parser = Parser::new(code);
            let result = parser.parse();
            assert!(result.is_ok(), "Failed to parse: {}", code);
        }
    }

    #[test]
    fn test_parse_flowchart_direction_variants() {
        for dir in &["TD", "LR", "RL", "BT"] {
            let code = format!("graph {}; A-->B", dir);
            let mut parser = Parser::new(&code);
            let diagram = parser.parse().unwrap();
            assert_eq!(diagram.direction.as_deref(), Some(*dir));
        }
    }

    #[test]
    fn test_parse_flowchart_multiple_edges() {
        let code = "graph TD; A-->B; B-->C; C-->D; D-->E";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        let edges = diagram.get_edges();
        assert_eq!(edges.len(), 4);
    }

    #[test]
    fn test_parse_flowchart_edge_with_label() {
        let code = "graph TD; A-->|Yes|B; C-->|No|D";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        let edges = diagram.get_edges_with_labels();
        let labels: Vec<_> = edges.iter().filter_map(|(_, _, l)| l.clone()).collect();
        assert!(labels.contains(&"Yes".to_string()));
        assert!(labels.contains(&"No".to_string()));
    }

    #[test]
    fn test_parse_empty_lines_and_comments() {
        let code = "graph TD\n%% comment\nA-->B\n\n%% another\nC-->D";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 2);
    }

    #[test]
    fn test_parse_sequence_with_extra_indentation() {
        let code = "sequenceDiagram\n        participant Alice\n            Alice->Bob: Hello";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Extra indentation should be ok");
    }

    #[test]
    fn test_parse_case_sensitivity() {
        let code = "graph TD; A[Start]-->B[End]";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_subgraph_edge() {
        let code = "graph TD\nsubgraph S\nA-->B\nend";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert!(!diagram.subgraphs.is_empty());
        assert_eq!(diagram.subgraphs[0].statements.len(), 1);
    }

    #[test]
    fn test_parse_node_shape_complex() {
        // Multiple node shapes in one diagram
        let code = "graph TD\nA[Rect]-->B(Round)\nB-->C{Diamond}\nC-->D((Double))";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sequence_actor_method_syntax() {
        let code = "sequenceDiagram\nAlice->>Bob: Auth\nBob-->>Alice: Token";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Cross arrows should parse: {:?}", result.err());
    }

    #[test]
    fn test_parse_class_multiline_body() {
        let code = "classDiagram\nclass Animal {\n+String name\n+int age\n+isMammal() bool\n+eat(food: String) void\n}";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_er_relationship_all_types() {
        let cases = vec![
            "||--||",
            "|o--o|",
        ];
        for code_fragment in &cases {
            let code = format!("erDiagram\nCUSTOMER {} ORDER : label", code_fragment);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            assert!(result.is_ok(), "Failed to parse {}: {:?}", code_fragment, result.err());
        }
    }

    #[test]
    fn test_parse_state_multiple_transitions() {
        let code = "stateDiagram-v2\n[*] --> Idle\nIdle --> Processing\nProcessing --> Completed\nProcessing --> Failed\nCompleted --> [*]\nFailed --> Idle";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok());
        let diagram = result.unwrap();
        let transitions: Vec<_> = diagram.statements.iter()
            .filter(|s| matches!(s, Statement::StateTransition { .. }))
            .collect();
        assert_eq!(transitions.len(), 6);
    }

    #[test]
    fn test_parse_gantt_multiple_sections() {
        let code = "gantt\n    section S1\n    T1 :t1, 1, 5d\n    section S2\n    T2 :t2, 10, 3d";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_pie_chart_title_with_spaces() {
        let code = "pie title My Awesome Chart\n\"A\" : 50\n\"B\" : 50";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.title.as_deref(), Some("My Awesome Chart"));
    }

    // ---- Error handling 测试 ----

    #[test]
    fn test_parse_error_location() {
        let mut parser = Parser::new("invalid%content%%");
        let err = parser.parse().err().unwrap();
        assert!(err.message.len() > 0, "Error should have a message");
    }

    #[test]
    fn test_parse_partial_recovery() {
        // Should not panic on bad input
        let mut parser = Parser::new("!!! graph @@@ TD ###");
        let _ = parser.parse();
    }

    #[test]
    fn test_parse_very_long_identifier() {
        let long_id = "A".repeat(100);
        let code = format!("graph TD; {}-->B", long_id);
        let mut parser = Parser::new(&code);
        let result = parser.parse();
        assert!(result.is_ok(), "Long identifiers should parse");
    }

    #[test]
    fn test_parse_unicode_in_sequence() {
        let code = "sequenceDiagram\n    User->System: 登录\n    System-->User: 成功";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Unicode should parse: {:?}", result.err());
    }

    #[test]
    fn test_parse_state_auto_discover() {
        let code = "stateDiagram-v2\nA --> B\nB --> C";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        // Should have 2 transitions
        let transitions: Vec<_> = diagram.statements.iter()
            .filter(|s| matches!(s, Statement::StateTransition { .. }))
            .collect();
        assert_eq!(transitions.len(), 2);
    }

    // ---- ER Diagram 解析测试 ----

    #[test]
    fn test_parse_er_basic() {
        let code = "erDiagram\nCUSTOMER ||--o{ ORDER : places";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Er);
        assert!(diagram.statements.len() >= 1);
    }

    #[test]
    fn test_parse_er_entity() {
        let code = "erDiagram\nCUSTOMER {\nstring name\nstring custNumber\n}";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Er);
        // Should have entity + attributes
        let entities: Vec<_> = diagram.statements.iter()
            .filter(|s| matches!(s, Statement::ErEntity { .. }))
            .collect();
        assert_eq!(entities.len(), 1);
        let attrs: Vec<_> = diagram.statements.iter()
            .filter(|s| matches!(s, Statement::ErAttribute { .. }))
            .collect();
        assert_eq!(attrs.len(), 2);
    }

    #[test]
    fn test_parse_er_relationship() {
        let code = "erDiagram\nCUSTOMER ||--o{ ORDER : places";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        let rels: Vec<_> = diagram.statements.iter()
            .filter(|s| matches!(s, Statement::ErRelation { .. }))
            .collect();
        assert_eq!(rels.len(), 1);
        match rels[0] {
            Statement::ErRelation { from, to, label, .. } => {
                assert_eq!(from, "CUSTOMER");
                assert_eq!(to, "ORDER");
                assert_eq!(label.as_deref(), Some("places"));
            }
            _ => panic!("Expected ErRelation"),
        }
    }

    #[test]
    fn test_parse_er_multiple_relationships() {
        let code = "erDiagram\nCUSTOMER ||--o{ ORDER : places\nORDER ||--|{ LINE-ITEM : contains";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        let rels: Vec<_> = diagram.statements.iter()
            .filter(|s| matches!(s, Statement::ErRelation { .. }))
            .collect();
        assert_eq!(rels.len(), 2);
    }

    // ---- Gantt Chart 解析测试 ----

    #[test]
    fn test_parse_gantt_basic() {
        let code = "gantt\n    title A Gantt Diagram\n    section Section\n    A task :a1, 2024-01-01, 30d";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Gantt);
        assert_eq!(diagram.title.as_deref(), Some("A Gantt Diagram"));
    }

    #[test]
    fn test_parse_gantt_section() {
        let code = "gantt\n    section My Section\n    Task 1 :t1, 2024-01-01, 10d";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        let sections: Vec<_> = diagram.statements.iter()
            .filter(|s| matches!(s, Statement::GanttSection { .. }))
            .collect();
        assert_eq!(sections.len(), 1);
    }

    #[test]
    fn test_parse_gantt_task() {
        let code = "gantt\n    Task 1 :t1, 2024-01-01, 10d";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        let tasks: Vec<_> = diagram.statements.iter()
            .filter(|s| matches!(s, Statement::GanttTask { .. }))
            .collect();
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn test_parse_gantt_no_title() {
        let code = "gantt\n    section S\n    Task :t1, 1, 5d";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert!(diagram.title.is_none());
    }

    #[test]
    fn test_parse_edge_label_pipe() {
        let code = "graph TD\nA-->|Yes|B\nB-->|No|C";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        let edges = diagram.get_edges_with_labels();
        assert_eq!(edges.len(), 2);
        assert_eq!(edges[0].2.as_deref(), Some("Yes"));
        assert_eq!(edges[1].2.as_deref(), Some("No"));
    }

    #[test]
    fn test_parse_diamond_node_on_both_ends() {
        let code = "graph TD\nA{Yes}-->B{No}";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 3);
    }

    #[test]
    fn test_parse_flag_node_on_both_ends() {
        let code = "graph TD\nA>In]-->B>Out]";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.statements.len(), 3);
    }

    #[test]
    fn test_parse_subgraph_missing_end() {
        let code = "graph TD\nsubgraph S\nA-->B";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Subgraph without end should still parse");
        let diagram = result.unwrap();
        assert!(!diagram.subgraphs.is_empty(), "Should have a subgraph");
    }

    #[test]
    fn test_parse_multiple_diagram_headers() {
        let code = "graph TD\nA-->B\ngraph LR\nC-->D";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Flowchart);
        // Should parse without crashing; second 'graph' is treated as content
        assert!(diagram.get_edges().len() >= 1);
    }

    #[test]
    fn test_parse_flowchart_lr_direction_edges() {
        let code = "graph LR\nA[Left]-->B[Right]\nB-->C[Far]";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.direction.as_deref(), Some("LR"));
        let edges = diagram.get_edges();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_parse_nested_subgraph() {
        let code = "graph TD\nsubgraph Outer\nA-->B\nsubgraph Inner\nC-->D\nend\nend";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Nested subgraph should parse: {:?}", result.err());
    }

    #[test]
    fn test_parse_large_flowchain() {
        let mut parts = vec!["graph TD".to_string()];
        for i in 0..20 {
            parts.push(format!("N{}-->N{}", i, i + 1));
        }
        let code = parts.join("\n");
        let mut parser = Parser::new(&code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.get_edges().len(), 20);
    }

    #[test]
    fn test_parse_mixed_content_comments() {
        let code = "graph TD\n%% comment line\nA-->B\n%% another comment\nC-->D";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert_eq!(diagram.get_edges().len(), 2);
    }

    #[test]
    fn test_parse_subgraph_with_node_id_used_as_title() {
        let code = "graph TD\nsubgraph MyGroup\nA-->B\nend";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        assert!(!diagram.subgraphs.is_empty(), "Should have subgraph");
        assert_eq!(diagram.subgraphs[0].id, "MyGroup", "Identifier after subgraph becomes id");
        assert_eq!(diagram.subgraphs[0].statements.len(), 1, "Should have inner edges");
    }
}
