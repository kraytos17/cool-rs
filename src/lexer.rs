use std::str::Chars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    BoolLiteral(bool),
    Case,
    Class,
    ClassName(String),
    Else,
    Eof,
    Esac,
    Fi,
    Ident(String),
    If,
    In,
    Illegal,
    Inherits,
    IsVoid,
    IntLiteral(i64),
    LBrace,
    Let,
    Loop,
    New,
    Not,
    Of,
    Pool,
    RBrace,
    Semicolon,
    StringLiteral(String),
    Then,
    While,
}

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    start: Pos,
    end: Pos,
}

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    offset: usize,
    line: usize,
    col: usize,
}

impl Pos {
    pub fn new(offset: usize, line: usize, col: usize) -> Self {
        Self { offset, line, col }
    }

    fn advance(&mut self, c: char) {
        self.offset += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexerError {
    InvalidChar(char),
    StringConstantTooLong(usize),
    StringContainsNull,
    StringUnterminated,
    StringContainsEof,
    UnmatchedComment,
    UnterminatedComment,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidChar(c) => write!(f, "Invalid character: {}", c),
            Self::StringConstantTooLong(len) => {
                write!(f, "String constant too long: {} chars", len)
            }
            Self::StringContainsNull => write!(f, "String contains null character"),
            Self::StringUnterminated => write!(f, "Unterminated string constant"),
            Self::StringContainsEof => write!(f, "EOF in string constant"),
            Self::UnmatchedComment => write!(f, "Unmatched comment"),
            Self::UnterminatedComment => write!(f, "EOF in comment"),
        }
    }
}

impl std::error::Error for LexerError {}

pub struct Lexer<'a> {
    buffer: Chars<'a>,
    current: Option<char>,
    pos: Pos,
    peek: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(buffer: &'a str) -> Self {
        let mut chars = buffer.chars();
        let curr = chars.next();

        Self {
            buffer: chars,
            current: curr,
            pos: Pos::new(0, 1, 0),
            peek: None,
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();
        let start = self.pos;

        let token_type = match self.current {
            None => TokenType::Eof,
            Some(c) => match c {
                '{' => {
                    self.advance();
                    TokenType::LBrace
                }
                '}' => {
                    self.advance();
                    TokenType::RBrace
                }
                '"' => self.tokenize_string()?,
                '0'..='9' => self.tokenize_number()?,
                'a'..='z' | 'A'..='Z' | '_' => self.tokenize_ident()?,
                c => return Err(LexerError::InvalidChar(c)),
            },
        };

        Ok(Token {
            token_type,
            span: Span {
                start,
                end: self.pos,
            },
        })
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current {
            if !ch.is_whitespace() {
                break;
            }
            self.advance();
        }
    }

    fn advance(&mut self) -> Option<char> {
        let curr = self.current;
        if let Some(c) = curr {
            self.pos.advance(c);
        }

        self.current = if let Some(next) = self.peek.take() {
            Some(next)
        } else {
            self.buffer.next()
        };

        curr
    }

    fn peek(&mut self) -> Option<char> {
        if self.peek.is_none() {
            self.peek = self.buffer.next();
        }

        self.peek
    }

    fn tokenize_string(&mut self) -> Result<TokenType, LexerError> {
        let mut string = String::new();
        self.advance();
        
        while let Some(ch) = self.current {
            match ch {
                '"' => {
                    self.advance();
                    return Ok(TokenType::StringLiteral(string));
                }
                '\0' => return Err(LexerError::StringContainsNull),
                '\n' => return Err(LexerError::StringUnterminated),
                ch => {
                    string.push(ch);
                    self.advance();
                }
            }
        }

        Err(LexerError::StringContainsEof)
    }

    fn tokenize_number(&mut self) -> Result<TokenType, LexerError> {
        let mut num = String::new();
        while let Some(ch) = self.current {
            if !ch.is_ascii_digit() {
                break;
            }
            num.push(ch);
            self.advance();
        }

        num.parse::<i64>()
            .map(TokenType::IntLiteral)
            .map_err(|_| LexerError::InvalidChar('0'))
    }

    fn tokenize_ident(&mut self) -> Result<TokenType, LexerError> {
        let mut ident = String::new();
        while let Some(ch) = self.current {
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }

            ident.push(ch);
            self.advance();
        }

        let id = match ident.as_str() {
            "true" => TokenType::BoolLiteral(true),
            "false" => TokenType::BoolLiteral(false),
            "if" => TokenType::If,
            "fi" => TokenType::Fi,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "case" => TokenType::Case,
            "esac" => TokenType::Esac,
            "isvoid" => TokenType::IsVoid,
            "let" => TokenType::Let,
            "loop" => TokenType::Loop,
            "new" => TokenType::New,
            "not" => TokenType::Not,
            "of" => TokenType::Of,
            "pool" => TokenType::Pool,
            "then" => TokenType::Then,
            "in" => TokenType::In,
            "inherits" => TokenType::Inherits,
            "class" => TokenType::Class,
            _ => {
                if ident.starts_with(char::is_uppercase) {
                    TokenType::ClassName(ident)
                } else {
                    TokenType::Ident(ident)
                }
            }
        };

        Ok(id)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(token) => {
                if token.token_type == TokenType::Eof {
                    None
                } else {
                    Some(Ok(token))
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}
