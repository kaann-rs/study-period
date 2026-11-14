use crate::{expr::LiteralValue, token::TokenType};

pub fn keyword(text: &str) -> Option<TokenType> {
    Some(match text {
        "and" => TokenType::And,
        "class" => TokenType::Class,
        "else" => TokenType::Else,
        "false" => TokenType::False,
        "for" => TokenType::For,
        "fun" => TokenType::Fun,
        "if" => TokenType::If,
        "nil" => TokenType::Nil,
        "or" => TokenType::Or,
        "print" => TokenType::Print,
        "return" => TokenType::Return,
        "super" => TokenType::Super,
        "this" => TokenType::This,
        "true" => TokenType::True,
        "var" => TokenType::Var,
        "while" => TokenType::While,
        "import" => TokenType::Import,
        _ => return None,
    })
}

pub fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

pub fn is_alpha(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == '_'
}

pub fn is_alpha_numeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}
