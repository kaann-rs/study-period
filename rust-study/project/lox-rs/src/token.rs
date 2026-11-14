use std::fmt::{self};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Hash,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    Identifier,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Number,

    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
    String,
    Import,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "
            TokenType: {:?}
            Lexeme: {}
            Literal: {}
            Line: {}
            ",
            self.token_type,
            self.lexeme,
            self.literal.unwrap_or("Error"),
            self.line,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

pub struct SourceMap {}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub token_type: TokenType,
    // pub span: Span,
    lexeme: &'a str,
    pub literal: Option<&'a str>,
    line: i32,
}

impl<'a> Token<'a> {
    pub fn new(
        token_type: TokenType,
        lexeme: &'a str,
        literal: Option<&'a str>,
        line: i32,
    ) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}
