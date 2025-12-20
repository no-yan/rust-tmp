use std::collections::HashMap;

use crate::ast::{BinaryOp, Expression, For, If, Program, Statement, UnaryOp, While};

struct Environment {
    register: HashMap<String, i32>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            register: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, n: i32) {
        self.register.insert(name.to_string(), n);
    }

    pub fn get(&self, name: &str) -> Option<i32> {
        self.register.get(name).copied()
    }
}

pub struct Evaluator {
    env: Environment,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
        }
    }

    pub fn eval(&mut self, program: &Program) -> i32 {
        let mut result = 0;
        for s in &program.body {
            result = self.stmt(s);
        }
        result
    }

    fn stmt(&mut self, stmt: &Statement) -> i32 {
        match stmt {
            Statement::ExpressionStatement(e) => self.expr(e),
            Statement::BlockStatement(stmt_list) => {
                let mut result = 0;
                for s in stmt_list {
                    result = self.stmt(s);
                }
                result
            }
            Statement::If(If { cond, then }) => {
                let mut result = 0;
                if self.expr(cond) > 0 {
                    for s in then {
                        result = self.stmt(s);
                    }
                }
                result
            }
            Statement::While(While { cond, body }) => {
                let mut result = 0;
                while self.expr(cond) > 0 {
                    for s in body {
                        result = self.stmt(s);
                    }
                }
                result
            }
            Statement::For(For {
                init,
                cond,
                update,
                body,
            }) => {
                let mut result = 0;

                if let Some(init) = init {
                    result = self.expr(init);
                }

                let Some(cond) = cond else {
                    // 簡単な実装のために、条件文がない場合にはblock
                    // statementを評価しない
                    // TODO:
                    return 0;
                };

                while self.expr(cond) > 0 {
                    for s in body {
                        result = self.stmt(s);
                    }

                    if let Some(update) = update {
                        self.expr(update);
                    }
                }
                result
            }
        }
    }

    fn expr(&mut self, expr: &Expression) -> i32 {
        match expr {
            Expression::Unary { op, expr } => match op {
                UnaryOp::Minus => -self.expr(expr),
            },
            Expression::Binary { lhs, op, rhs } => match op {
                BinaryOp::Plus => self.expr(lhs) + self.expr(rhs),
                BinaryOp::Minus => self.expr(lhs) - self.expr(rhs),
                BinaryOp::Mul => self.expr(lhs) * self.expr(rhs),
                BinaryOp::Div => self.expr(lhs) / self.expr(rhs),
                BinaryOp::Pow => self.expr(lhs).pow(self.expr(rhs) as u32),
                BinaryOp::Gt => {
                    if self.expr(lhs) > self.expr(rhs) {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::GtEq => {
                    if self.expr(lhs) >= self.expr(rhs) {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::Lt => {
                    if self.expr(lhs) < self.expr(rhs) {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::LtEq => {
                    if self.expr(lhs) <= self.expr(rhs) {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::Assign => {
                    let Expression::Var(name) = &**lhs else {
                        unreachable!("Parser guarantees LHS is Var for Assign");
                    };
                    let v = self.expr(rhs);
                    self.env.define(name, v);
                    v
                }
            },
            Expression::Value(v) => *v,
            Expression::Var(name) => self.env.get(name).unwrap(), // このロジックは未定義変数でパニックする
        }
    }
}
