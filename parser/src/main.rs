use std::error::Error;

mod lexer;
mod token;

use crate::lexer::Lexer;
use crate::token::Token;

fn main() -> Result<(), Box<dyn Error>> {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;

    let mut lexer = Lexer::new(&buf);
    let tokens = lexer.lex()?;

    match Calculator::calc(tokens) {
        Ok(val) => println!("{}", val),
        Err(err) => eprintln!("{:?}", err),
    };

    Ok(())
}

// TODO:
// ゴール: かっこつきの演算をサポートする
// ## 調査:
// Shunting yard algorithmでカッコの演算処理をどうするか確認する
//
// ## 実装:
// 1. トークン"(", ")"を追加する
// 2. トークナイズ処理を追加する
// 3. Calculatorの判定にカッコの処理を追加する
//
struct Calculator;

/// Shunting yard algorithm (See: https://en.wikipedia.org/wiki/Shunting_yard_algorithm)
///
/// 演算子がオペランドの間におかれる構文を解析するアルゴリズム。得られる出力は逆ポーランド記法になる。
/// 以下の手順で出力を得る:
/// (出力用のベクタと、演算子を一時的に保管するStackを用意する)
/// 入力からトークンをpopする
/// 1. 数値: 出力にpush
/// 2. 演算子:
///      Stackのtopがより高い優先順位を持つ場合:
///          stackをpop, 出力にpush
///      Stackにpush
/// 3. 入力が空になったら:
///      Stackを空になるまでpopし出力にpushする
impl Calculator {
    fn calc(input: Vec<Token>) -> Result<i32, Box<dyn Error>> {

        // 1. 計算の順序
        //
        // (*, /) → (+, -)
        //
        // 2. EBNF
        //
        // Expr      = UnaryExpr
        //           | Expr BinaryOp Expr
        // BinaryOp  = AddOp | MulOp
        // AddOp     = "+" | "-"
        // MulOp     = "*" | "/"
        // UnaryExpr = Num

        let rpn = Self::infix_to_rpn(input);
        Self::evaluate_rpn(rpn)
    }

    fn infix_to_rpn(input: Vec<Token>) -> Vec<Token> {
        let mut output = vec![];
        let mut ops_stack: Vec<Token> = vec![];

        // 入力が空になるまで、次のことを続ける
        // 1. トークンを一つ読み出す
        // 2. トークン種別に応じて次のことを行う
        //    a. 数値: output.push
        //    b. 演算子: ops_stack.push

        let input = input.into_iter();
        for tok in input {
            match tok {
                Token::Num(_) => output.push(tok),
                Token::Plus | Token::Minus | Token::Mul | Token::Div => {
                    while let Some(op) = ops_stack.last()
                        && op.precedence() >= tok.precedence()
                    {
                        let op = ops_stack.pop().unwrap();
                        output.push(op);
                    }
                    ops_stack.push(tok);
                }
                _ => todo!(),
            }
        }
        while let Some(op) = ops_stack.pop() {
            output.push(op);
        }

        output
    }

    fn evaluate_rpn(rpn: Vec<Token>) -> Result<i32, Box<dyn Error>> {
        let mut stack: Vec<i32> = Vec::new();

        for tok in rpn.into_iter() {
            match tok {
                Token::Num(n) => stack.push(n),
                Token::Plus | Token::Minus | Token::Mul | Token::Div => {
                    let rhs = stack.pop().ok_or("stack underflow (rhs)")?;
                    let lhs = stack.pop().ok_or("stack underflow (lhs)")?;
                    let val = Self::apply_op(tok, lhs, rhs);
                    stack.push(val);
                }
                _ => unreachable!(""),
            }
        }

        assert!(stack.len() == 1);
        Ok(stack[0])
    }

    fn apply_op(tok: Token, lhs: i32, rhs: i32) -> i32 {
        match tok {
            Token::Plus => lhs + rhs,
            Token::Minus => lhs - rhs,
            Token::Mul => lhs * rhs,
            Token::Div => lhs / rhs,
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum() {
        let input = "1 + 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = Calculator::calc(tokens);

        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn difference() {
        let input = "1 - 2 - 3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = Calculator::calc(tokens);

        assert_eq!(result.unwrap(), -4);
    }

    #[test]
    fn sum_3_operand() {
        let input = "1 + 2 + 3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = Calculator::calc(tokens);

        assert_eq!(result.unwrap(), 6);
    }

    #[test]
    fn prod_3_operand() {
        let input = "1*2*3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = Calculator::calc(tokens);

        assert_eq!(result.unwrap(), 6);
    }

    #[test]
    fn process_with_priority() {
        let input = "1+2*3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = Calculator::calc(tokens);

        assert_eq!(result.unwrap(), 7);
    }

    #[test]
    fn without_space() {
        let input = "1+2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = Calculator::calc(tokens).unwrap();

        assert_eq!(result, 3)
    }

    #[test]
    fn with_paren() {
        let input = "(1+2)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = Calculator::calc(tokens).unwrap();

        assert_eq!(result, 3);
    }

    #[test]
    fn with_paren_precedence() {
        let input = "(1+2)*3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = Calculator::calc(tokens).unwrap();

        assert_eq!(result, 3);
    }
}
