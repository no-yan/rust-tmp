mod ast;
mod error;
mod lexer;
mod parser;
mod token;

use std::process::ExitCode;

use crate::{
    error::{CompilerError, format_error},
    lexer::Lexer,
    parser::Parser,
};

// TODO: refactor: astモジュールの切り出し
// - [x] enum AST {expression(Expression)};
// - [x] parser::Expression::binary, unary, valueをAST::expressionに移動
//
// TODO: 式のサポート
// - [ ] EBNFを更新
// - [ ] ASTにProgram, Statementを追加
// - [ ] トークンに";"を追加
// - [ ] レキサで";"をサポート
// - [ ] パーサでprogram, stmtをサポート
//
// TODO: refactor: Evaluatorの切り出し
// - [ ] 計算式の評価をevaluatorに移動
// - [ ] Environment構造体を作成し、evaluatorで使用
// - [ ] eval_program, eval_expr()を実装
//
// TODO: 変数のサポート
// - [ ] Token::Ident(String)
// - [ ] Lex Ident
// - [ ] Parse Ident
// - [ ] Eval Ident
//
// TODO: 代入式のサポート
// - [ ] トークンに"="を追加
// - [ ] レキサで"="をサポート
// - [ ] EBNFを更新
// - [ ] パーサでstmtにAssignmentを追加
fn run(input: &str) -> Result<i32, CompilerError> {
    let tokens = Lexer::new(input).lex()?;
    let expr = Parser::new(tokens).parse()?;
    Ok(expr.eval())
}

fn main() -> ExitCode {
    // 引数で式が与えられた場合はそれを入力として扱う
    // それ以外は標準入力にフォールバックする
    let arg = std::env::args().nth(1);
    let input = arg.unwrap_or_else(|| {
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Failed to read input");
        buf.trim_end().to_owned()
    });

    run(&input)
        .inspect(|v| println!("{v}"))
        .inspect_err(|e| eprintln!("{}", format_error(e, &input)))
        .map_or(ExitCode::FAILURE, |_| ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser::SyntaxError, token::TokenKind::*};

    fn parse(input: &str) -> Result<i32, CompilerError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex()?;
        let ast = Parser::new(tokens).parse()?;
        Ok(ast.eval())
    }

    #[test]
    fn sum() {
        let result = parse("1 + 2");
        assert_eq!(result, Ok(3));
    }

    #[test]
    fn difference() {
        let result = parse("1 - 2 - 3");
        assert_eq!(result, Ok(-4));
    }

    #[test]
    fn sum_3_operand() {
        let result = parse("1 + 2 + 3");
        assert_eq!(result, Ok(6));
    }

    #[test]
    fn prod_3_operand() {
        let result = parse("1*2*3");
        assert_eq!(result, Ok(6));
    }

    #[test]
    fn process_with_priority() {
        let result = parse("1+2*3");
        assert_eq!(result, Ok(7));
    }

    #[test]
    fn without_space() {
        let result = parse("1+2");
        assert_eq!(result, Ok(3));
    }

    #[test]
    fn with_paren() {
        let result = parse("(1+2)");
        assert_eq!(result, Ok(3));
    }
    #[test]
    fn with_paren_precedence() {
        let result = parse("(1+2)*3");
        assert_eq!(result, Ok(9));
    }

    #[test]
    fn power() {
        let result = parse("10^2");
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn gt_true() {
        let result = parse("1>0");
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn gt_false() {
        let result = parse("1>2");
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn gt_eq_true() {
        let result = parse("1>=1");
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn gt_eq_false() {
        let result = parse("1>=2");
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn lt_true() {
        let result = parse("1<2");
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn lt_false() {
        let result = parse("1<0");
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn lt_eq_true() {
        let result = parse("1<=1");
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn lt_eq_false() {
        let result = parse("1<=0");
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn unmatched_left_paren() {
        let result = parse("(1+2");
        assert_eq!(
            result,
            Err(SyntaxError::UnmatchedLeftParen(tok!(LeftParen, 0, 1)).into())
        );
    }

    #[test]
    fn unmatched_right_paren() {
        let result = parse("1+2)");
        assert_eq!(
            result,
            Err(SyntaxError::UnexpectedToken(tok!(RightParen, 3, 4)).into())
        );
    }

    #[test]
    fn unary_minus() {
        let result = parse("-1");
        assert_eq!(result, Ok(-1));
    }

    #[test]
    fn unexpected_eof() {
        let result = parse("-");
        assert_eq!(result, Err(SyntaxError::UnexpectedEof.into()));
    }
}
