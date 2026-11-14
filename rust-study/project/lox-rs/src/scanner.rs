use crate::{
    Lox,
    token::{Token, TokenType},
};

use crate::util;

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
    start: usize,
    current: usize,
    line: i32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            current: 0,
            line: 1,
            start: 0,
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn advance(&mut self) -> char {
        let c = self.source[self.current..].chars().next().unwrap();

        self.current += c.len_utf8();

        c
    }

    pub fn match_char(&mut self, expected: char) -> bool {
        if self.peek() != expected {
            return false;
        }

        self.current += expected.len_utf8();

        true
    }

    pub fn add_token(&mut self, token_type: TokenType, literal: Option<&'a str>) {
        let text: &str = &self.source[self.start..self.current];

        self.tokens
            .push(Token::new(token_type, text, literal, self.line));
    }

    pub fn add_token_without_literal(&mut self, token_type: TokenType) {
        self.add_token(token_type, Some("s"));
    }

    pub fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token_without_literal(TokenType::LeftParen),

            ')' => self.add_token_without_literal(TokenType::RightParen),

            '{' => self.add_token_without_literal(TokenType::LeftBrace),

            '}' => self.add_token_without_literal(TokenType::RightBrace),

            ',' => self.add_token_without_literal(TokenType::Comma),

            '.' => self.add_token_without_literal(TokenType::Dot),

            '-' => self.add_token_without_literal(TokenType::Minus),

            '+' => self.add_token_without_literal(TokenType::Plus),

            ';' => self.add_token_without_literal(TokenType::Semicolon),

            '*' => self.add_token_without_literal(TokenType::Star),

            '!' => {
                if self.match_char('=') {
                    self.add_token_without_literal(TokenType::BangEqual);
                } else {
                    self.add_token_without_literal(TokenType::Bang);
                }
            }

            '=' => {
                if self.match_char('=') {
                    self.add_token_without_literal(TokenType::EqualEqual);
                } else {
                    self.add_token_without_literal(TokenType::Equal);
                }
            }

            '<' => {
                if self.match_char('=') {
                    self.add_token_without_literal(TokenType::LessEqual);
                } else {
                    self.add_token_without_literal(TokenType::Less);
                }
            }

            '>' => {
                if self.match_char('=') {
                    self.add_token_without_literal(TokenType::GreaterEqual);
                } else {
                    self.add_token_without_literal(TokenType::Greater);
                }
            }

            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token_without_literal(TokenType::Slash);
                }
            }

            '"' => self.scan_string(),

            '#' => self.add_token_without_literal(TokenType::Hash),

            '\n' => self.line += 1,

            ' ' | '\r' | '\t' => {}

            i => {
                if util::is_digit(i) {
                    self.scan_number();
                } else if util::is_alpha(i) {
                    self.scan_identifier();
                } else {
                    Lox::error(self.line, "Unexpected Character");
                }
            }
        }
    }

    pub fn scan_number(&mut self) {
        while util::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && util::is_digit(self.peek_next()) {
            self.advance();
            while util::is_digit(self.peek()) {
                self.advance();
            }
        }

        let literal = &self.source[self.start..self.current];
        self.add_token(TokenType::Number, Some(literal));
    }

    pub fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        let mut it = self.source[self.current..].chars();
        it.next();
        it.next().unwrap_or('\0')
    }

    pub fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source[self.current..].chars().next().unwrap()
    }

    pub fn scan_tokens(mut self) -> Vec<Token<'a>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "a", None, self.line));

        self.tokens
    }

    pub fn scan_string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            Lox::error(self.line, "Unterminated string");
            return;
        }

        self.advance();

        let value = &self.source[self.start + 1..self.current - 1];

        self.add_token(TokenType::String, Some(value));
    }

    fn scan_identifier(&mut self) {
        while util::is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_type = util::keyword(text).unwrap_or(TokenType::Identifier);

        self.add_token_without_literal(token_type);
    }
}
