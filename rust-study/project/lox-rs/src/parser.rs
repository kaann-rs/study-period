use crate::expr::{Expr, LiteralValue};
use crate::token::{Token, TokenType};

#[derive(Debug)]
enum ParserError {
    UnexpectedToken(String),
    Undefined(String),
}

#[derive(Debug)]
pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, current: 0 }
    }

    fn peek(&self) -> &Token<'a> {
        self.tokens.get(self.current).unwrap()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn previous(&self) -> &Token<'a> {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn advance(&mut self) -> &Token<'a> {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn check(&self, typ: TokenType) -> bool {
        !self.is_at_end() && self.peek().token_type == typ
    }

    fn match_token(&mut self, typ: &[TokenType]) -> bool {
        if typ.iter().any(|f| self.check(*f)) {
            self.advance();

            true
        } else {
            false
        }
    }

    pub fn expression(&mut self) -> Expr<'a> {
        self.equality()
    }

    fn primary(&mut self) -> Result<Expr<'a>, ParserError> {
        let token = self.advance().token_type;

        match token {
            TokenType::False => Ok(Expr::literal(LiteralValue::Bool(false))),
            TokenType::True => Ok(Expr::literal(LiteralValue::Bool(true))),
            TokenType::Nil => Ok(Expr::literal(LiteralValue::Nil)),
            TokenType::Number => {
                let value = self.previous().literal.unwrap();
                Ok(Expr::literal(LiteralValue::Number(
                    value.parse::<f64>().unwrap(),
                )))
            }
            TokenType::String => {
                let value = self.previous().literal.unwrap();
                Ok(Expr::literal(LiteralValue::String(value)))
            }
            TokenType::LeftParen => {
                let expr = self.expression();

                self.consume(
                    TokenType::RightParen,
                    "Expect ')' after expression".to_string(),
                )
                .unwrap();

                Ok(Expr::grouping(Box::new(expr)))
            }
            _ => Err(ParserError::Undefined(String::from("Undefined type"))),
        }
    }

    fn consume(&mut self, typ: TokenType, message: String) -> Result<&Token<'a>, std::io::Error> {
        if self.check(typ) {
            Ok(self.advance())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                message,
            ))
        }
    }

    fn unary(&mut self) -> Expr<'a> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().token_type;

            let right = self.unary();

            return Expr::unary(operator, Box::new(right));
        }

        self.primary().unwrap()
    }

    fn equality(&mut self) -> Expr<'a> {
        let mut expr = self.comparison();

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().token_type;

            let right = self.comparison();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn comparison(&mut self) -> Expr<'a> {
        let mut expr = self.term();

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().token_type;

            let right = self.term();

            expr = Expr::binary(Box::new(expr), operator, Box::new(right));
        }

        expr
    }

    fn term(&mut self) -> Expr<'a> {
        let mut expr = self.factor();

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().token_type;

            let right = self.factor();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        expr
    }

    fn factor(&mut self) -> Expr<'a> {
        let mut expr = self.unary();

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().token_type;

            let right = self.unary();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        expr
    }
}
