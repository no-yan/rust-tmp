use std::error::Error;
use std::iter::Peekable;
use std::fmt;

mod lexer;
mod token;

use crate::lexer::Lexer;
use crate::token::Token;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let buf = match args.iter().nth(1) {
        Some(buf) => buf.clone(),
        None => {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf)?;
            buf
        }
    };

    let mut lexer = Lexer::new(&buf);
    let tokens = lexer.lex()?;

    let ast =  Parser::new(tokens).parse()?;
    let v = ast.eval()?;
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
// - [ ] テストを新しいAPIに変更する
// - [ ] exprのパースを実装する
// - [ ] primaryのパースを実装する
// - [ ] Token::eval()を実装する
// - []
//
//


enum Expression {
    Unary { op: Token, value: i32 },
    Binary {
        lhs: Box<Expression>,
        op: Token,
        rhs: Box<Expression>,
    }
}

impl Expression {
    fn eval(&self) -> Result<i32, Box<dyn Error>> {
        unimplemented!()
    }
}


#[derive(Debug)]
enum ParseError {

}

type ParseResult<T> = Result<T, ParseError>;
impl Error for ParseError{}
impl std::fmt::Display for ParseError {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}


/// ## EBNF
/// Program -> Expr(0)
/// Expr(p) ->  Primary { BinOp Expr(p) }
/// Primary -> Unary Expr(q) | "(" E ")" | v
/// BinOp   -> "+" | "-" | "*" | "/"
/// Unary   -> "-"
struct Parser{
    src: Peekable<std::vec::IntoIter<Token>>,
}

impl Parser {
    fn new(src: Vec<Token>) -> Self{
        Self{
            src: src.into_iter().peekable()
        }
    }

    fn parse(&self) -> ParseResult<Expression> {
        unimplemented!()
    }

    fn expr() -> ParseResult<Expression>{
        unimplemented!()
    }

    fn primary() -> ParseResult<Expression>{
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> i32 {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let ast = Parser::new(tokens).parse().unwrap();
        ast.eval().unwrap()
    }

    #[test]
    fn sum() {
        let result = parse( "1 + 2");
        assert_eq!(result, 3);
    }

    #[test]
    fn difference() {
        let result = parse("1 - 2 - 3");
        assert_eq!(result, -4);
    }

    #[test]
    fn sum_3_operand() {
        let result = parse("1 + 2 + 3");
        assert_eq!(result, 6);
    }

    #[test]
    fn prod_3_operand() {
        let result = parse("1*2*3");
        assert_eq!(result, 6);
    }

    #[test]
    fn process_with_priority() {
        let result = parse("1+2*3");
        assert_eq!(result, 7);
    }

    #[test]
    fn without_space() {
        let result = parse("1+2");
        assert_eq!(result, 3)
    }

    #[test]
    fn with_paren() {
        let result = parse("(1+2)");
        assert_eq!(result, 3);
    }

    #[test]
    fn with_paren_precedence() {
        let result = parse("(1+2)*3");
        assert_eq!(result, 9);
    }

    #[test]
    #[should_panic]
    fn unmatched_left_paren() {
        let _ = parse("(1+2");
    }

    #[test]
    #[should_panic]
    fn unmatched_right_paren() {
        let _ = parse("1+2)");
    }

    #[test]
    fn unary_minus() {
        let result = parse("-1");
        assert_eq!(result, -1);
    }
}
