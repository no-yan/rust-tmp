use std::error::Error;
use std::fmt;
use std::iter::Peekable;

use crate::error::Spanned;
use crate::token::{Span, Token, TokenKind};

#[derive(Debug, PartialEq)]
pub enum SyntaxError {
    UnmatchedLeftParen(Token),
    UnexpectedToken(Token),
    UnexpectedEof,
}

impl Error for SyntaxError {}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::UnmatchedLeftParen(_) => write!(f, "Unmatched left parenthesis"),
            SyntaxError::UnexpectedToken(tok) => write!(f, "Unexpected token: {:?}", tok.kind),
            SyntaxError::UnexpectedEof => write!(f, "Unexpected end of file"),
        }
    }
}

impl Spanned for SyntaxError {
    fn span(&self) -> Option<Span> {
        match self {
            SyntaxError::UnmatchedLeftParen(tok) | SyntaxError::UnexpectedToken(tok) => {
                Some(tok.span.clone())
            }
            _ => None,
        }
    }
}

pub type ParseResult<T> = Result<T, SyntaxError>;

#[derive(Debug)]
pub enum Expression {
    Unary {
        op: TokenKind,
        expr: Box<Expression>,
    },
    Binary {
        lhs: Box<Expression>,
        op: TokenKind,
        rhs: Box<Expression>,
    },
    Value(i32),
}

impl Expression {
    pub fn eval(&self) -> i32 {
        match self {
            Expression::Unary { op, expr } => match op {
                TokenKind::Minus => -expr.eval(),
                _ => unreachable!(),
            },
            Expression::Binary { lhs, op, rhs } => match op {
                TokenKind::Plus => lhs.eval() + rhs.eval(),
                TokenKind::Minus => lhs.eval() - rhs.eval(),
                TokenKind::Mul => lhs.eval() * rhs.eval(),
                TokenKind::Div => lhs.eval() / rhs.eval(),
                TokenKind::Pow => lhs.eval().pow(rhs.eval() as u32),
                _ => unreachable!(),
            },
            Expression::Value(v) => *v,
        }
    }
}

#[derive(Debug)]
enum Assoc {
    Left,
    Right,
}

/// 演算子の優先度と結合順序を表す。
#[derive(Debug)]
struct OpInfo {
    prec: u8,
    assoc: Assoc,
}

impl OpInfo {
    fn binds_at(&self, min_prec: u8) -> bool {
        self.prec >= min_prec
    }
}

/// 二項演算子としてトークンが持つ[`OpInfo`]を返す。
/// トークンが二項演算子ではない場合、Noneを返す。
fn binary_op(tok: &TokenKind) -> Option<OpInfo> {
    use crate::token::TokenKind::*;

    match tok {
        Plus | Minus => Some(OpInfo {
            prec: 1,
            assoc: Assoc::Left,
        }),
        Mul | Div => Some(OpInfo {
            prec: 2,
            assoc: Assoc::Left,
        }),
        Pow => Some(OpInfo {
            prec: 4,
            assoc: Assoc::Right,
        }),
        _ => None,
    }
}

/// 単項演算子としてトークンが持つ[`OpInfo`]を返す。
/// トークンが単項演算子ではない場合、Noneを返す。
fn unary_op(tok: &TokenKind) -> Option<OpInfo> {
    use crate::token::TokenKind::*;

    match tok {
        Minus => Some(OpInfo {
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
/// - 二項演算子: "+", "-", "*", "/", "*"
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
            let Some(op_info) = binary_op(&tok.kind) else {
                break;
            };

            if !op_info.binds_at(min_prec) {
                break;
            }

            // トークンを消費
            let tok = self.src.next().unwrap();

            let next_prec = match op_info.assoc {
                Assoc::Left => op_info.prec + 1,
                Assoc::Right => op_info.prec,
            };
            let rhs = self.expr(next_prec)?;
            lhs = Expression::Binary {
                lhs: Box::new(lhs),
                op: tok.kind,
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn primary(&mut self) -> ParseResult<Expression> {
        let tok = self.src.next();
        let primary = match tok.clone().map(|tok| tok.kind) {
            Some(TokenKind::Num(n)) => Expression::Value(n),
            Some(TokenKind::Minus) => {
                let info = unary_op(&TokenKind::Minus).unwrap();
                let expr = self.expr(info.prec)?;
                Expression::Unary {
                    op: TokenKind::Minus,
                    expr: Box::new(expr),
                }
            }
            Some(TokenKind::LeftParen) => {
                let expr = self.expr(0)?;
                if self.expect(TokenKind::RightParen).is_err() {
                    return Err(SyntaxError::UnmatchedLeftParen(tok.unwrap()));
                };
                expr
            }
            Some(_) => return Err(SyntaxError::UnexpectedToken(tok.unwrap())),
            None => return Err(SyntaxError::UnexpectedEof),
        };

        Ok(primary)
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), SyntaxError> {
        match self.src.next() {
            Some(ref tok) if tok.kind == expected => Ok(()),
            Some(tok) => Err(SyntaxError::UnexpectedToken(tok)),
            None => Err(SyntaxError::UnexpectedEof),
        }
    }
}
