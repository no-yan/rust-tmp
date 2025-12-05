#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Plus,
    Minus,
    Mul,
    Div,

    Num(i32),

    LeftParen,
    RightParen,
}

impl Token {
    // The higher precedes the lower.
    pub fn precedence(&self) -> i32 {
        use crate::Token::*;

        match self {
            Plus | Minus => 1,
            Mul | Div => 2,
            LeftParen | RightParen => 3,
            Num(_) => 999,
        }
    }
}
