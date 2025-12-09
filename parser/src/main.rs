mod error;
mod lexer;
mod parser;
mod token;

use crate::error::CompilerError;
use crate::lexer::Lexer;
use crate::parser::Parser;

fn main() -> Result<(), CompilerError> {
    let args: Vec<String> = std::env::args().collect();
    let buf = match args.get(1) {
        Some(buf) => buf.clone(),
        None => {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).expect("Failed to read input");
            buf
        }
    };

    let mut lexer = Lexer::new(&buf);
    let tokens = lexer.lex()?;

    let ast = Parser::new(tokens).parse()?;
    let v = ast.eval();
    println!("{}", v);

    Ok(())
}

// TODO:
// ゴール: 単項演算子をサポートする
//
// ### 調査:
// - [x] 単項演算子をサポートするアルゴリズムを3つ知る
// - [x] 実装の容易さ、拡張性を比較する
// - [x] 一般的なパーサーの使用するアルゴリズムを調査する
// - [ ] Precedence climbing parserのアルゴリズムを説明できるようになる
//
// ### 実装
// - [x] EBNFに単項演算子を追加
// - [x] テストを新しいAPIに変更する
// - [x] exprのパースを実装する
// - [x] primaryのパースを実装する
// - [x] Token::eval()を実装する

#[cfg(test)]
mod tests {
    use super::*;

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
    #[should_panic]
    fn unmatched_left_paren() {
        parse("(1+2").unwrap();
    }

    #[test]
    #[should_panic]
    fn unmatched_right_paren() {
        parse("1+2)").unwrap();
    }

    #[test]
    fn unary_minus() {
        let result = parse("-1");
        assert_eq!(result, Ok(-1));
    }
}
