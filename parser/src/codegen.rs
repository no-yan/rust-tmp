use crate::{
    ast,
    ast::{BinaryOp, Expression, Program, Statement, UnaryOp},
};

pub struct CodeGenerator {
    output: Vec<String>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self { output: vec![] }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.output.push("    .globl _main".to_string());
        self.output.push("_main:".to_string());

        for stmt in &program.body {
            self.stmt(stmt);
        }

        self.output.push("    ldr x0, [sp], #16".to_string());
        self.output.push("    ret".to_string());

        self.print()
    }

    fn print(&self) -> String {
        self.output.join("\n")
    }

    fn stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::ExpressionStatement(expr) => {
                self.expr(expr);
                self.output.push("    ldr x0, [sp], #16".to_string());
            }
            Statement::If(ast::If { cond, then }) => {
                self.expr(cond);

                // 1. cmpで比較
                //   true:  ジャンプしない
                //   false: .LelseXXXにジャンプ
                // 2. trueの末尾で、.LendXXXにジャンプ
                self.output.push("    ldr x0, [sp], #16".to_string());
                // truthy判定の実装を簡単にするため、x0が0の場合、else文にジャンプしている
                self.output.push("    cmp x0, #0".to_string());
                self.output.push("    b.eq .LelseXXX".to_string());

                for s in then {
                    self.stmt(s);
                }
                self.output.push("    b .LendXXX".to_string());
                self.output.push(".LelseXXX:".to_string());
                self.output.push(".LendXXX:".to_string());
            }
            Statement::BlockStatement(_) => unimplemented!(),
            Statement::While(_) => unimplemented!(),
            Statement::For(_) => unimplemented!(),
        };
    }

    fn expr(&mut self, expr: &Expression) {
        match expr {
            Expression::Unary { op, expr } => match op {
                UnaryOp::Minus => {
                    self.expr(expr);
                    self.output.push("    ldr x0, [sp], #16".to_string());
                    self.output.push("    neg x0, x0".to_string());
                    self.output.push("    str x0, [sp, #-16]!".to_string())
                }
            },
            Expression::Binary { lhs, op, rhs } => match op {
                BinaryOp::Plus => {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.output.push("    ldr x1, [sp], #16".to_string());
                    self.output.push("    ldr x0, [sp], #16".to_string());
                    self.output.push("    add x0, x0, x1".to_string());
                    self.output.push("    str x0, [sp, #-16]!".to_string());
                }
                BinaryOp::Minus => {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.output.push("    ldr x1, [sp], #16".to_string());
                    self.output.push("    ldr x0, [sp], #16".to_string());
                    self.output.push("    sub x0, x0, x1".to_string());
                    self.output.push("    str x0, [sp, #-16]!".to_string());
                }
                BinaryOp::Mul => {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.output.push("    ldr x1, [sp], #16".to_string());
                    self.output.push("    ldr x0, [sp], #16".to_string());
                    self.output.push("    mul x0, x0, x1".to_string());
                    self.output.push("    str x0, [sp, #-16]!".to_string());
                }
                BinaryOp::Div => {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.output.push("    ldr x1, [sp], #16".to_string());
                    self.output.push("    ldr x0, [sp], #16".to_string());
                    // CAUTION: sdivはゼロ除算がエラーにならず、0を出力する
                    self.output.push("    sdiv x0, x0, x1".to_string());
                    self.output.push("    str x0, [sp, #-16]!".to_string());
                }
                BinaryOp::Pow => {
                    self.expr(lhs);
                    self.expr(rhs);

                    // result *= a; b--; if (b != 0) goto L;
                    // x0 = a, x1 = b
                    self.output.push("    ldr x1, [sp], #16".to_string());
                    self.output.push("    ldr x0, [sp], #16".to_string());
                    self.output.push("    mov x2, #1".to_string());
                    self.output.push("0:  ".to_string());
                    self.output.push("    mul x2, x2, x0".to_string());
                    self.output
                        .push("    subs x1, x1, #1  ; b-- and set flags".to_string());
                    self.output.push("    b.ne 0b".to_string());
                    self.output.push("1:  ".to_string());
                    self.output.push("    mov x0, x2".to_string());
                    self.output.push("    str x0, [sp, #-16]!".to_string());
                }
                BinaryOp::Eq => {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.output.push("    ldr x1, [sp], #16".to_string());
                    self.output.push("    ldr x0, [sp], #16".to_string());
                    self.output.push("    cmp x0, x1".to_string());
                    self.output
                        .push("    cset x0, eq  ; x0 = 1 if x0 == x1".to_string());
                    self.output.push("    str x0, [sp, #-16]!".to_string());
                }
                BinaryOp::Neq => {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.output.push("    ldr x1, [sp], #16".to_string());
                    self.output.push("    ldr x0, [sp], #16".to_string());
                    self.output.push("    cmp x0, x1".to_string());
                    self.output
                        .push("    cset x0, ne  ; x0 = 1 if x0 != x1".to_string());
                    self.output.push("    str x0, [sp, #-16]!".to_string());
                }
                BinaryOp::Gt => {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.output.push("    ldr x1, [sp], #16".to_string());
                    self.output.push("    ldr x0, [sp], #16".to_string());
                    self.output.push("    cmp x0, x1".to_string());
                    self.output
                        .push("    cset x0, gt  ; x0 = 1 if x0 > x1".to_string());
                    self.output.push("    str x0, [sp, #-16]!".to_string());
                }
                BinaryOp::GtEq => {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.output.push("    ldr x1, [sp], #16".to_string());
                    self.output.push("    ldr x0, [sp], #16".to_string());
                    self.output.push("    cmp x0, x1".to_string());
                    self.output
                        .push("    cset x0, ge  ; x0 = 1 if x0 >= x1".to_string());
                    self.output.push("    str x0, [sp, #-16]!".to_string());
                }
                BinaryOp::Lt => {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.output.push("    ldr x1, [sp], #16".to_string());
                    self.output.push("    ldr x0, [sp], #16".to_string());
                    self.output.push("    cmp x0, x1".to_string());
                    self.output
                        .push("    cset x0, lt  ; x0 = 1 if x0 < x1".to_string());
                    self.output.push("    str x0, [sp, #-16]!".to_string());
                }
                BinaryOp::LtEq => {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.output.push("    ldr x1, [sp], #16".to_string());
                    self.output.push("    ldr x0, [sp], #16".to_string());
                    self.output.push("    cmp x0, x1".to_string());
                    self.output
                        .push("    cset x0, le  ; x0 = 1 if x0 <= x1".to_string());
                    self.output.push("    str x0, [sp, #-16]!".to_string());
                }
                BinaryOp::Assign => {
                    unimplemented!();
                }
            },
            Expression::Value(n) => {
                self.output.push(format!("    mov x0, #{}", n));
                self.output.push("    str x0, [sp, #-16]!".to_string());
            }
            Expression::Var(_name) => {
                unimplemented!();
            }
        };
    }
}
