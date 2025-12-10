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
