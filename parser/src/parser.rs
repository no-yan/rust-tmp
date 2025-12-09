use std::error::Error;
use std::fmt;
use std::iter::Peekable;

use crate::token::Token;

#[derive(Debug, PartialEq)]
pub enum SyntaxError {
    UnmatchedLeftParen,
    UnexpectedToken(Token),
}

impl Error for SyntaxError {}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::UnmatchedLeftParen => write!(f, "Unmatched left parenthesis"),
            SyntaxError::UnexpectedToken(tok) => write!(f, "Unexpected token: {:?}", tok),
        }
    }
}

pub type ParseResult<T> = Result<T, SyntaxError>;

#[derive(Debug)]
pub enum Expression {
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
    pub fn eval(&self) -> i32 {
        match self {
            Expression::Unary { op, expr } => match op {
                Token::Minus => -expr.eval(),
                _ => unreachable!(),
            },
            Expression::Binary { lhs, op, rhs } => match op {
                Token::Plus => lhs.eval() + rhs.eval(),
                Token::Minus => lhs.eval() - rhs.eval(),
                Token::Mul => lhs.eval() * rhs.eval(),
                Token::Div => lhs.eval() / rhs.eval(),
                _ => unreachable!(),
            },
            Expression::Value(v) => *v,
        }
    }
}

#[derive(Debug)]
enum Assoc {
    Left,
    // Right,
}

#[derive(Debug)]
struct PrecInfo {
    prec: u8,
    assoc: Assoc,
}

impl PrecInfo {
    fn binds_at(&self, min_prec: u8) -> bool {
        self.prec >= min_prec
    }
}

fn binary_prec(tok: &Token) -> Option<PrecInfo> {
    use crate::token::Token::*;

    match tok {
        Plus | Minus => Some(PrecInfo {
            prec: 1,
            assoc: Assoc::Left,
        }),
        Mul | Div => Some(PrecInfo {
            prec: 2,
            assoc: Assoc::Left,
        }),
        _ => None,
    }
}

fn unary_prec(tok: &Token) -> Option<PrecInfo> {
    use crate::token::Token::*;

    match tok {
        Minus => Some(PrecInfo {
            prec: 3,
            assoc: Assoc::Left,
        }),
        _ => None,
    }
}

/// 計算式を構文解析し、[`Expression`]を構築するパーサー。
///
/// ## サポートする演算子
///
/// - 二項演算子: "+", "-", "*", "/"
/// - 単項演算子: "-"
///
/// # AST の構造
///
/// 構築される AST は優先度が低い演算子が根に、高い演算子が葉に配置される。
///
/// 例: `1 + 2 * 3` は以下の構造になる:
/// ```text
///       +
///      / \
///     1   *
///        / \
///       2   3
/// ```
/// パーサーは浮動小数点数をサポートせず、パースに失敗した場合にエラーを返す
///
/// ## Example
///
/// ```rust
/// let mut lexer = Lexer::new("1+2");
/// let token = lexer.lex()?;
///
/// let expr = Parser::new(token).parse()?;
/// let v = expr.eval();
/// assert_eq!(v, 3);
/// ```
pub struct Parser {
    src: Peekable<std::vec::IntoIter<Token>>,
}

impl Parser {
    pub fn new(src: Vec<Token>) -> Self {
        Self {
            src: src.into_iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> ParseResult<Expression> {
        // Precedence climbing algorithmを使用してパースを行う。
        // see: https://www.engr.mun.ca/~theo/Misc/exp_parsing.htm#climbing

        // ## 文法
        //
        // E -> Expr(0)
        // Expr(p) ->  Primary { BinOp Expr(q) }
        // Primary -> Unary Expr(q) | "(" E ")" | v
        // BinOp   -> "+" | "-" | "*" | "/"
        // Unary   -> "-"
        //
        match self.expr(0) {
            Ok(expr) => {
                if let Some(tok) = self.src.next() {
                    Err(SyntaxError::UnexpectedToken(tok))
                } else {
                    Ok(expr)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn expr(&mut self, min_prec: u8) -> ParseResult<Expression> {
        let mut lhs = self.primary()?;

        while let Some(tok) = self.src.peek() {
            if !tok.is_op() {
                break;
            }

            let Some(prec_info) = binary_prec(tok) else {
                return Err(SyntaxError::UnexpectedToken(tok.clone()));
            };
            if !prec_info.binds_at(min_prec) {
                break;
            }

            let tok = self.src.next().unwrap();

            let next_prec = match prec_info.assoc {
                Assoc::Left => prec_info.prec + 1,
                // Assoc::Right => prec_info.prec,
            };
            let rhs = self.expr(next_prec)?;
            lhs = Expression::Binary {
                lhs: Box::new(lhs),
                op: tok,
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn primary(&mut self) -> ParseResult<Expression> {
        let primary = match self.src.next() {
            Some(Token::Num(n)) => Expression::Value(n),
            Some(Token::Minus) => {
                let info = unary_prec(&Token::Minus).unwrap();
                let expr = self.expr(info.prec)?;
                Expression::Unary {
                    op: Token::Minus,
                    expr: Box::new(expr),
                }
            }
            Some(Token::LeftParen) => {
                let expr = self.expr(0)?;
                let next = self.src.next();
                if !matches!(next, Some(Token::RightParen)) {
                    Err(SyntaxError::UnmatchedLeftParen)
                } else {
                    Ok(expr)
                }
            }?,
            Some(tok) => return Err(SyntaxError::UnexpectedToken(tok)),
            None => unimplemented!(),
        };

        Ok(primary)
    }
}
