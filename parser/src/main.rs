use std::error::Error;
use std::fmt;
use std::iter::Peekable;

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

    let ast = Parser::new(tokens).parse()?;
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
// - [x] テストを新しいAPIに変更する
// - [x] exprのパースを実装する
// - [x] primaryのパースを実装する
// - [x] Token::eval()を実装する

#[derive(Debug)]
enum Expression {
    Unary {
        op: Token,
        expr: Box<Expression>,
    },
    Binary {
        lhs: Box<Expression>,
        op: Token,
        rhs: Box<Expression>,
    },
    Value(i32),
}

impl Expression {
    fn eval(&self) -> Result<i32, Box<dyn Error>> {
        match self {
            Expression::Unary { op, expr } => Ok(match op {
                Token::Minus => -expr.eval()?,
                _ => unreachable!(),
            }),
            Expression::Binary { lhs, op, rhs } => Ok(match op {
                Token::Plus => lhs.eval()? + rhs.eval()?,
                Token::Minus => lhs.eval()? - rhs.eval()?,
                Token::Mul => lhs.eval()? * rhs.eval()?,
                Token::Div => lhs.eval()? / rhs.eval()?,
                _ => unreachable!(""),
            }),
            Expression::Value(v) => Ok(*v),
        }
    }
}

#[derive(Debug)]
enum ParseError {}

type ParseResult<T> = Result<T, ParseError>;
impl Error for ParseError {}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

/// ## EBNF
/// E -> Expr(0)
/// Expr(p) ->  Primary { BinOp Expr(q) }
/// Primary -> Unary Expr(q) | "(" E ")" | v
/// BinOp   -> "+" | "-" | "*" | "/"
/// Unary   -> "-"
struct Parser {
    src: Peekable<std::vec::IntoIter<Token>>,
}

impl Parser {
    fn new(src: Vec<Token>) -> Self {
        Self {
            src: src.into_iter().peekable(),
        }
    }

    fn parse(&mut self) -> ParseResult<Expression> {
        match self.expr(0) {
            Ok(expr) => {
                debug_assert!(self.src.next().is_none());
                Ok(expr)
            },
            Err(e) => Err(e),
        }
    }

    fn expr(&mut self, prec: u8) -> ParseResult<Expression> {
        let mut lhs = self.primary()?;

        while let Some(tok) = self.src.peek() {
            if !tok.is_op() {
                break;
            }
            if !tok.precedes(prec) {
                break;
            }
            let tok = self.src.next().unwrap();

            // NOTE:
            // 右連結の演算子を導入する場合、同じPrecedenceもrhsに含めてよい
            let rhs = self.expr(tok.prec() + 1)?;
            lhs = Expression::Binary {
                lhs: Box::new(lhs),
                op: tok.clone(),
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn primary(&mut self) -> ParseResult<Expression> {
        let primary = match self.src.next() {
            Some(Token::Num(n)) => Expression::Value(n),
            Some(Token::Minus) => {
                let expr = self.expr(4)?; // TODO: どこかに配置
                Expression::Unary {
                    op: Token::Minus,
                    expr: Box::new(expr),
                }
            }
            Some(Token::LeftParen) => {
                let expr = self.expr(0)?;
                let next  = self.src.next();
                if !matches!(next, Some(Token::RightParen)) {
                    panic!("Unmatched LeftParen");
                }
                    expr
            }
            _ => unreachable!("unknown parser error"),
        };

        Ok(primary)
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
        let result = parse("1 + 2");
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
