#[derive(Debug, PartialEq, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub trait Spanned {
    fn span(&self) -> Option<Span>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Plus,
    Minus,
    Mul,
    Div,
    Pow,
    Eq,

    Gt,   // >
    Lt,   // <
    GtEq, // >=
    LtEq, // <=

    Num(i32),
    Ident(String),

    LeftParen,
    RightParen,

    Semicolon,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

#[macro_export]
macro_rules! tok {
    ($kind:expr, $start:expr, $end:expr) => {{
        $crate::token::Token {
            kind: $kind,
            span: $crate::token::Span {
                start: $start,
                end: $end,
            },
        }
    }};
}
