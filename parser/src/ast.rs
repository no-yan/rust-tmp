use crate::token::TokenKind;

pub mod prec {
    pub const LOWEST: u8 = 0;
    pub const ASSIGN: u8 = 1;
    pub const COMPARE: u8 = 2;
    pub const PLUS: u8 = 3;
    pub const MUL: u8 = 4;
    pub const UNARY: u8 = 5;
    pub const POW: u8 = 6;
}

#[derive(Debug)]
pub enum Assoc {
    Left,
    Right,
}

/// 演算子の優先度と結合順序を表す。
#[derive(Debug)]
pub struct OpInfo {
    pub prec: u8,
    pub assoc: Assoc,
}

impl OpInfo {
    pub fn binds_at(&self, min_prec: u8) -> bool {
        self.prec >= min_prec
    }
}

#[derive(Debug)]
pub enum BinaryOp {
    Plus,
    Minus,
    Mul,
    Div,
    Pow,
    Eq,
    Neq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    Assign,
}

#[derive(Debug)]
pub enum UnaryOp {
    Minus,
}

impl TryFrom<&TokenKind> for BinaryOp {
    type Error = ();

    fn try_from(kind: &TokenKind) -> Result<Self, Self::Error> {
        use TokenKind::*;

        match kind {
            Plus => Ok(BinaryOp::Plus),
            Minus => Ok(BinaryOp::Minus),
            Mul => Ok(BinaryOp::Mul),
            Div => Ok(BinaryOp::Div),
            Pow => Ok(BinaryOp::Pow),
            Eq => Ok(BinaryOp::Eq),
            Neq => Ok(BinaryOp::Neq),
            Gt => Ok(BinaryOp::Gt),
            Lt => Ok(BinaryOp::Lt),
            GtEq => Ok(BinaryOp::GtEq),
            LtEq => Ok(BinaryOp::LtEq),
            Assign => Ok(BinaryOp::Assign),
            _ => Err(()),
        }
    }
}

impl BinaryOp {
    pub fn op_info(&self) -> OpInfo {
        use BinaryOp::*;

        match self {
            Eq | Neq | Gt | GtEq | Lt | LtEq => OpInfo {
                prec: prec::COMPARE,
                assoc: Assoc::Left,
            },
            Plus | Minus => OpInfo {
                prec: prec::PLUS,
                assoc: Assoc::Left,
            },
            Mul | Div => OpInfo {
                prec: prec::MUL,
                assoc: Assoc::Left,
            },
            Pow => OpInfo {
                prec: prec::POW,
                assoc: Assoc::Right,
            },
            Assign => OpInfo {
                prec: prec::ASSIGN,
                assoc: Assoc::Right,
            },
        }
    }
}

#[derive(Debug)]
pub enum Expression {
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    Binary {
        lhs: Box<Expression>,
        op: BinaryOp,
        rhs: Box<Expression>,
    },
    Value(i32),
    Var(String),
}

#[derive(Debug)]
pub struct If {
    pub cond: Expression,
    pub then: Vec<Statement>,
}

#[derive(Debug)]
pub struct While {
    pub cond: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct For {
    pub init: Option<Expression>,
    pub cond: Option<Expression>,
    pub update: Option<Expression>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    ExpressionStatement(Expression),
    BlockStatement(Vec<Statement>),
    If(If),
    While(While),
    For(For),
}

#[derive(Debug)]
pub struct Program {
    pub body: Vec<Statement>,
}
