use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword(String),
    Identifier(String),
    Arrow,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Pipe,
    String(String),
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::Keyword(s) => write!(f, "Keyword({})", s),
            TokenType::Identifier(s) => write!(f, "Identifier({})", s),
            TokenType::Arrow => write!(f, "Arrow"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Pipe => write!(f, "|"),
            TokenType::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 0,
            column: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                break;
            }

            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }

        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        if self.is_at_end() {
            return None;
        }

        let start_line = self.line;
        let start_column = self.column;
        let ch = self.current_char()?;

        Some(match ch {
            '[' => {
                self.advance();
                Token {
                    token_type: TokenType::LeftBracket,
                    line: start_line,
                    column: start_column,
                }
            }
            ']' => {
                self.advance();
                Token {
                    token_type: TokenType::RightBracket,
                    line: start_line,
                    column: start_column,
                }
            }
            '(' => {
                self.advance();
                Token {
                    token_type: TokenType::LeftParen,
                    line: start_line,
                    column: start_column,
                }
            }
            ')' => {
                self.advance();
                Token {
                    token_type: TokenType::RightParen,
                    line: start_line,
                    column: start_column,
                }
            }
            '{' => {
                self.advance();
                Token {
                    token_type: TokenType::LeftBrace,
                    line: start_line,
                    column: start_column,
                }
            }
            '}' => {
                self.advance();
                Token {
                    token_type: TokenType::RightBrace,
                    line: start_line,
                    column: start_column,
                }
            }
            ';' => {
                self.advance();
                Token {
                    token_type: TokenType::Semicolon,
                    line: start_line,
                    column: start_column,
                }
            }
            '|' => {
                self.advance();
                Token {
                    token_type: TokenType::Pipe,
                    line: start_line,
                    column: start_column,
                }
            }
            '-' | '=' => self.read_arrow_or_minus(),
            '"' | '\'' => self.read_string(),
            _ if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
            _ => {
                self.advance();
                return self.next_token();
            }
        })
    }

    fn read_arrow_or_minus(&mut self) -> Token {
        let start_line = self.line;
        let start_column = self.column;
        let ch = self.current_char().unwrap();

        self.advance();
        if let Some(next_ch) = self.current_char() {
            if (ch == '-' && (next_ch == '-' || next_ch == '>')) || (ch == '=' && next_ch == '=') {
                self.advance();

                // 继续检查是否有 >
                if let Some(third_ch) = self.current_char() {
                    if third_ch == '>' {
                        self.advance();
                    }
                }

                return Token {
                    token_type: TokenType::Arrow,
                    line: start_line,
                    column: start_column,
                };
            }
        }

        Token {
            token_type: TokenType::Identifier("-".to_string()),
            line: start_line,
            column: start_column,
        }
    }

    fn read_string(&mut self) -> Token {
        let start_line = self.line;
        let start_column = self.column;
        let quote = self.current_char().unwrap();
        self.advance();

        let mut value = String::new();
        while !self.is_at_end() && self.current_char() != Some(quote) {
            if let Some(ch) = self.current_char() {
                value.push(ch);
                self.advance();
            }
        }

        if !self.is_at_end() {
            self.advance();
        }

        Token {
            token_type: TokenType::String(value),
            line: start_line,
            column: start_column,
        }
    }

    fn read_identifier(&mut self) -> Token {
        let start_line = self.line;
        let start_column = self.column;
        let mut ident = String::new();

        while !self.is_at_end() {
            if let Some(ch) = self.current_char() {
                if ch.is_alphanumeric() || ch == '_' {
                    ident.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let token_type = match ident.as_str() {
            "graph" | "flowchart" | "sequenceDiagram" | "classDiagram" | "stateDiagram"
            | "gantt" | "pie" | "erDiagram" => TokenType::Keyword(ident),
            "TD" | "BT" | "LR" | "RL" => TokenType::Keyword(ident),
            "end" | "subgraph" => TokenType::Keyword(ident),
            _ => TokenType::Identifier(ident),
        };

        Token {
            token_type,
            line: start_line,
            column: start_column,
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            if let Some(ch) = self.current_char() {
                if ch.is_whitespace() {
                    if ch == '\n' {
                        self.line += 1;
                        self.column = 0;
                    } else {
                        self.column += 1;
                    }
                    self.advance();
                } else if ch == '%' && self.peek(1) == Some('%') {
                    // 注释：跳过到行末
                    while !self.is_at_end() && self.current_char() != Some('\n') {
                        self.advance();
                    }
                } else {
                    break;
                }
            }
        }
    }

    fn current_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        if self.position + offset < self.input.len() {
            Some(self.input[self.position + offset])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
            self.column += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic_tokens() {
        let mut lexer = Lexer::new("graph TD");
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(ref s) if s == "graph"));
        assert!(matches!(tokens[1].token_type, TokenType::Keyword(ref s) if s == "TD"));
    }

    #[test]
    fn test_lexer_arrows() {
        let mut lexer = Lexer::new("A-->B");
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0].token_type, TokenType::Identifier(_)));
        assert!(matches!(tokens[1].token_type, TokenType::Arrow));
        assert!(matches!(tokens[2].token_type, TokenType::Identifier(_)));
    }

    #[test]
    fn test_lexer_brackets() {
        let mut lexer = Lexer::new("A[label]");
        let tokens = lexer.tokenize();
        assert!(tokens
            .iter()
            .any(|t| matches!(t.token_type, TokenType::LeftBracket)));
        assert!(tokens
            .iter()
            .any(|t| matches!(t.token_type, TokenType::RightBracket)));
    }

    #[test]
    fn test_lexer_comments() {
        let mut lexer = Lexer::new("%% This is a comment\nA-->B");
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0].token_type, TokenType::Identifier(_)));
    }
}
