pub mod ast;
pub mod lexer;

pub use ast::{Diagram, DiagramType, NodeShape, Statement};
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
        let _direction = if let Ok(token) = self.peek() {
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

        // 解析语句，直到到达 end 或文件末尾
        while !self.is_at_end() {
            // 尝试解析一个语句
            match self.parse_statement() {
                Ok(Some(stmt)) => statements.push(stmt),
                Ok(None) => continue,
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
        })
    }

    fn parse_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        if self.is_at_end() {
            return Ok(None);
        }

        let token = self.peek()?;
        match &token.token_type {
            TokenType::Keyword(k) if k == "end" => {
                let _ = self.advance();
                Ok(None)
            }
            TokenType::Identifier(_) => {
                let stmt = self.parse_node_or_edge()?;
                Ok(Some(stmt))
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

    fn parse_node_or_edge(&mut self) -> Result<Statement, ParseError> {
        let from_id = self.parse_node_id()?;

        // 检查是否是边定义（有箭头）
        if let Ok(arrow_token) = self.peek() {
            if matches!(arrow_token.token_type, TokenType::Arrow) {
                let _ = self.advance(); // 跳过箭头
                let to_id = self.parse_node_id()?;
                return Ok(Statement::EdgeDef {
                    from: from_id,
                    to: to_id,
                    label: None,
                });
            }
        }

        // 否则是节点定义
        Ok(Statement::NodeDef {
            id: from_id,
            label: None,
            shape: NodeShape::Rect,
        })
    }

    fn parse_node_id(&mut self) -> Result<String, ParseError> {
        let token = self.advance()?;
        match &token.token_type {
            TokenType::Identifier(id) => Ok(id.clone()),
            _ => Err(self.error("Expected identifier")),
        }
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
}
