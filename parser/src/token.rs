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
    pub fn is_op(&self) -> bool {
        use Token::*;
        matches!(self, Plus | Minus | Mul | Div)
    }
}
