use crate::ast::Expression;
use crate::ast::BinaryOp;
use crate::ast::UnaryOp;

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn eval(&self, expr: &Expression) -> i32 {
        match expr {
            Expression::Unary { op, expr } => match op {
                UnaryOp::Minus => -self.eval(expr),
            },
            Expression::Binary { lhs, op, rhs } => match op {
                BinaryOp::Plus => self.eval(lhs) + self.eval(rhs),
                BinaryOp::Minus => self.eval(lhs) - self.eval(rhs),
                BinaryOp::Mul => self.eval(lhs) * self.eval(rhs),
                BinaryOp::Div => self.eval(lhs) / self.eval(rhs),
                BinaryOp::Pow => self.eval(lhs).pow(self.eval(rhs) as u32),
                BinaryOp::Gt => {
                    if self.eval(lhs) > self.eval(rhs) {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::GtEq => {
                    if self.eval(lhs) >= self.eval(rhs) {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::Lt => {
                    if self.eval(lhs) < self.eval(rhs) {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::LtEq => {
                    if self.eval(lhs) <= self.eval(rhs) {
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
