mod ast;
mod codegen;
mod error;
mod lexer;
mod parser;
mod token;

use std::{
    fs::File,
    io::Write,
    process::{Command, ExitCode},
};

use crate::{
    codegen::CodeGenerator,
    error::{CompilerError, format_error},
    lexer::Lexer,
    parser::Parser,
};

// TODO: 重複しないラベル生成
// TODO: ローカル変数サポート
// TODO: return文のサポート
// TODO: 関数呼び出しサポート
// TODO: テスト再設計
// TODO: for文サポート
// TODO: while文サポート
// TODO: statement系でblock statement以外のbodyをパースできるようにする
fn run(input: &str) -> Result<(), CompilerError> {
    let tokens = Lexer::new(input).lex()?;
    let program = Parser::new(tokens).parse()?;
    let assembly_string = CodeGenerator::new().generate(&program);

    let mut f = File::create("test.s").unwrap();
    f.write_all(assembly_string.as_bytes()).unwrap();

    // Create object file
    let _ = Command::new("cc")
        .arg("-o")
        .arg("test")
        .arg("test.s")
        .output()
        .expect("failed to execute process");

    Ok(())
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
        let expr = Parser::new(tokens).parse()?;
        let mut evaluator = Evaluator::new();

        Ok(evaluator.eval(&expr))
    }

    #[test]
    fn sum() {
        let result = parse("1 + 2;");
        assert_eq!(result, Ok(3));
    }

    #[test]
    fn difference() {
        let result = parse("1 - 2 - 3;");
        assert_eq!(result, Ok(-4));
    }

    #[test]
    fn sum_3_operand() {
        let result = parse("1 + 2 + 3;");
        assert_eq!(result, Ok(6));
    }

    #[test]
    fn prod_3_operand() {
        let result = parse("1*2*3;");
        assert_eq!(result, Ok(6));
    }

    #[test]
    fn process_with_priority() {
        let result = parse("1+2*3;");
        assert_eq!(result, Ok(7));
    }

    #[test]
    fn without_space() {
        let result = parse("1+2;");
        assert_eq!(result, Ok(3));
    }

    #[test]
    fn with_paren() {
        let result = parse("(1+2);");
        assert_eq!(result, Ok(3));
    }
    #[test]
    fn with_paren_precedence() {
        let result = parse("(1+2)*3;");
        assert_eq!(result, Ok(9));
    }

    #[test]
    fn power() {
        let result = parse("10^2;");
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn gt_true() {
        let result = parse("1>0;");
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn gt_false() {
        let result = parse("1>2;");
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn gt_eq_true() {
        let result = parse("1>=1;");
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn gt_eq_false() {
        let result = parse("1>=2;");
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn lt_true() {
        let result = parse("1<2;");
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn lt_false() {
        let result = parse("1<0;");
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn lt_eq_true() {
        let result = parse("1<=1;");
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn lt_eq_false() {
        let result = parse("1<=0;");
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
        let result = parse("-1;");
        assert_eq!(result, Ok(-1));
    }

    #[test]
    fn unexpected_eof() {
        let result = parse("-");
        assert_eq!(result, Err(SyntaxError::UnexpectedEof.into()));
    }

    #[test]
    fn assignment() {
        let result = parse("x=2; x;");
        assert_eq!(result, Ok(2));
    }

    #[test]
    fn invalid_assignment() {
        let result = parse("1=2;");
        assert_eq!(
            result,
            Err(SyntaxError::InvalidAssignmentTarget(tok!(Assign, 1, 2)).into())
        );
    }

    #[test]
    fn if_statement() {
        let result = parse("x=0; if (1>=0) {x=2;} x;");

        assert_eq!(result, Ok(2),);
    }

    #[test]
    fn while_statement() {
        let result = parse("x=0; while(x<1){x=1;} x;");

        assert_eq!(result, Ok(1),);
    }

    #[test]
    fn for_statement() {
        let result = parse("for (ans=i=0; i<10; i=i+1) {ans = ans + i;} ans;");

        assert_eq!(result, Ok(45),);
    }

    #[test]
    fn for_with_empty_clause() {
        let result = parse("for (x=0;;) { x=1; } x;");
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn fibonatti() {
        let result =
            parse("n=10; a=0; b=1; for (i=0; i<n; i=i+1) { tmp = a; a = b; b = tmp + b;} a;");
        assert_eq!(result, Ok(55));
    }

    #[test]
    fn block_statement() {
        let result = parse("{ foo = 1; } foo;");
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn eq_true() {
        let result = parse("1==1;");
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn eq_false() {
        let result = parse("1==0;");
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn neq_true() {
        let result = parse("1!=0;");
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn neq_false() {
        let result = parse("1!=1;");
        assert_eq!(result, Ok(0));
    }
}
