use crate::token::TokenType;

#[derive(Debug, Clone)]
pub enum LiteralValue<'a> {
    Number(f64),
    String(&'a str),
    Bool(bool),
    Nil,
}

pub fn evolute(expr: Expr) -> LiteralValue {
    match expr {
        Expr::Literal { value } => value,

        Expr::Grouping { expression } => evolute(*expression),

        Expr::Unary { operator, right } => {
            let right = evolute(*right);

            match (operator, right) {
                (TokenType::Minus, LiteralValue::Number(right)) => LiteralValue::Number(-right),
                (TokenType::Plus, LiteralValue::Number(right)) => LiteralValue::Number(right),
                (TokenType::Star, LiteralValue::Number(right)) => LiteralValue::Number(right),
                _ => todo!(),
            }
        }

        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left = evolute(*left);
            let right = evolute(*right);

            match (left, operator, right) {
                (LiteralValue::Number(c), TokenType::Star, LiteralValue::Number(y)) => {
                    LiteralValue::Number(c * y)
                }
                (LiteralValue::Number(c), TokenType::Minus, LiteralValue::Number(y)) => {
                    LiteralValue::Number(c - y)
                }
                (LiteralValue::Number(c), TokenType::Plus, LiteralValue::Number(y)) => {
                    LiteralValue::Number(c + y)
                }
                (LiteralValue::Number(c), TokenType::Greater, LiteralValue::Number(y)) => {
                    LiteralValue::Bool(c > y)
                }
                (LiteralValue::Number(c), TokenType::GreaterEqual, LiteralValue::Number(y)) => {
                    LiteralValue::Bool(c >= y)
                }
                _ => todo!(),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Binary {
        left: Box<Expr<'a>>,
        operator: TokenType,
        right: Box<Expr<'a>>,
    },

    Grouping {
        expression: Box<Expr<'a>>,
    },

    Literal {
        value: LiteralValue<'a>,
    },

    Unary {
        operator: TokenType,
        right: Box<Expr<'a>>,
    },
}

impl<'a> Expr<'a> {
    pub fn binary(left: Box<Expr<'a>>, operator: TokenType, right: Box<Expr<'a>>) -> Self {
        Self::Binary {
            left,
            operator,
            right,
        }
    }

    pub fn grouping(expression: Box<Expr<'a>>) -> Expr<'a> {
        Self::Grouping { expression }
    }

    pub fn literal(value: LiteralValue<'a>) -> Expr<'a> {
        Self::Literal { value }
    }

    pub fn unary(operator: TokenType, right: Box<Expr<'a>>) -> Expr<'a> {
        Self::Unary { operator, right }
    }
}
