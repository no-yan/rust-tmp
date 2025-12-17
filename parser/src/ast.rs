use crate::token::TokenKind;

pub mod prec {
    pub const LOWEST: u8 = 0;
    pub const COMPARE: u8 = 1;
    pub const PLUS: u8 = 2;
    pub const MUL: u8 = 3;
    pub const UNARY: u8 = 3;
    pub const POW: u8 = 5;
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
    Gt,
    GtEq,
    Lt,
    LtEq,
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
            Gt => Ok(BinaryOp::Gt),
            Lt => Ok(BinaryOp::Lt),
            GtEq => Ok(BinaryOp::GtEq),
            LtEq => Ok(BinaryOp::LtEq),
            _ => Err(()),
        }
    }
}

impl BinaryOp {
    pub fn op_info(&self) -> OpInfo {
        use BinaryOp::*;

        match self {
            Gt | GtEq | Lt | LtEq => OpInfo {
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
}

impl Expression {
    pub fn eval(&self) -> i32 {
        match self {
            Expression::Unary { op, expr } => match op {
                UnaryOp::Minus => -expr.eval(),
            },
            Expression::Binary { lhs, op, rhs } => match op {
                BinaryOp::Plus => lhs.eval() + rhs.eval(),
                BinaryOp::Minus => lhs.eval() - rhs.eval(),
                BinaryOp::Mul => lhs.eval() * rhs.eval(),
                BinaryOp::Div => lhs.eval() / rhs.eval(),
                BinaryOp::Pow => lhs.eval().pow(rhs.eval() as u32),
                BinaryOp::Gt => {
                    if lhs.eval() > rhs.eval() {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::GtEq => {
                    if lhs.eval() >= rhs.eval() {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::Lt => {
                    if lhs.eval() < rhs.eval() {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::LtEq => {
                    if lhs.eval() <= rhs.eval() {
                        1
                    } else {
                        0
                    }
                }
            },
            Expression::Value(v) => *v,
        }
    }
}
