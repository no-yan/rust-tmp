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
    pub fn prec(&self) -> u8 {
        use Token::*;

        match self {
            Plus | Minus => 1,
            Mul | Div => 2,
            LeftParen | RightParen => 3,
            Num(_) => u8::MAX,
        }
    }

    pub fn precedes(&self, curr_prec: u8) -> bool {
        self.prec() >= curr_prec
    }

    pub fn is_op(&self) -> bool {
        use Token::*;
        matches!(self, Plus | Minus | Mul | Div)
    }
}
