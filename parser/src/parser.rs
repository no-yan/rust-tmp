use std::{error::Error, fmt, iter::Peekable};

use crate::{
    ast::{Assoc, BinaryOp, Expression, If, Program, Statement, UnaryOp, prec},
    token::{Span, Spanned, Token, TokenKind},
};

#[derive(Debug, PartialEq)]
pub enum SyntaxError {
    UnmatchedLeftParen(Token),
    UnexpectedToken(Token),
    InvalidAssignmentTarget(Token),
    UnexpectedEof,
}

impl Error for SyntaxError {}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::UnmatchedLeftParen(_) => write!(f, "Unmatched left parenthesis"),
            SyntaxError::UnexpectedToken(tok) => write!(f, "Unexpected token: {:?}", tok.kind),
            SyntaxError::InvalidAssignmentTarget(tok) => {
                write!(f, "Invalid assignment target: {:?}", tok.kind)
            }
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

/// 計算式を構文解析し、[`Expression`]を構築するパーサー。
///
/// ## 仕様
/// ### サポートする演算子
///
/// - 二項演算子: `+`, `-`, `*`, `/`, `^`, `>`, `<`, `>=`, `<=`
/// - 単項演算子: `-`
///
/// ### 優先順位
///
/// 下に行くほど優先度が高い
/// 1. `<` `<=` `>` `>=`
/// 2. `+` `-`
/// 3. `*` `/`
/// 4. 単項`-`
/// 5. `^`
/// 6. `(` `)`
///
/// ### 結合性
///
/// - 右結合: `^`
/// - 左結合: その他全て
///
/// ### 文法
///
/// Program -> Stmt { Stmt }
/// Stmt    -> If | E
/// If      -> "if" "(" E ")" "{" { Stmt ";" } "}"
///
/// E       -> Expr(0) ";"
/// Expr(p) -> Primary { BinOp Expr(q) }
/// Primary -> Unary Expr(q) | "(" E ")" | Ident | v
/// Ident   -> letter { letter | unicode_digit }
/// BinOp   -> "=" | "+" | "-" | "*" | "/" | "^" | ">" | "<" | ">=" | "<="
/// Unary   -> "-"
///
/// ### AST の構造
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
/// let tokens = lexer.lex()?;
///
/// let program = Parser::new(tokens).parse()?;
/// let mut evaluator = Evaluator::new();
/// let v = evaluator.eval(&program);
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

    pub fn parse(&mut self) -> ParseResult<Program> {
        // Precedence climbing algorithmを使用してパースを行う。
        // see: https://www.engr.mun.ca/~theo/Misc/exp_parsing.htm#climbing

        self.program()
    }

    fn program(&mut self) -> ParseResult<Program> {
        let mut stmt_list = vec![];

        let stmt = self.stmt()?;
        stmt_list.push(stmt);
        while !self.is_eof() {
            let stmt = self.stmt()?;
            stmt_list.push(stmt);
        }

        Ok(Program { body: stmt_list })
    }

    fn stmt(&mut self) -> ParseResult<Statement> {
        let tok = self.src.peek().ok_or(SyntaxError::UnexpectedEof)?;

        match tok.kind {
            TokenKind::If => Ok(self.r#if()?),
            _ => {
                let expr = self.expr(prec::LOWEST)?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Statement::ExpressionStatement(expr))
            }
        }
    }

    fn r#if(&mut self) -> ParseResult<Statement> {
        // If      -> "if" "(" E ")" "{" { Stmt ";" } "}"
        self.src.next();
        self.expect(TokenKind::LeftParen)?;
        let cond = self.expr(prec::LOWEST)?;
        self.expect(TokenKind::RightParen)?;

        self.expect(TokenKind::LeftBlock)?;

        let mut then = vec![];
        while let Some(tok) = self.src.peek()
            && tok.kind != TokenKind::RightBlock
        {
            then.push(self.stmt()?);
        }

        self.expect(TokenKind::RightBlock)?;

        Ok(Statement::If(If { cond, then }))
    }

    fn expr(&mut self, min_prec: u8) -> ParseResult<Expression> {
        let mut lhs = self.primary()?;

        while let Some(tok) = self.src.peek() {
            let Ok(op) = BinaryOp::try_from(&tok.kind) else {
                break;
            };
            let info = op.op_info();

            if !info.binds_at(min_prec) {
                break;
            }

            // 代入先が識別子でない場合、構文エラー
            // e.g. "1 = 2"
            if matches!(op, BinaryOp::Assign) && !matches!(lhs, Expression::Var(_)) {
                // TODO: エラーメッセージにlhsを表示する
                return Err(SyntaxError::InvalidAssignmentTarget(tok.clone()));
            }

            // トークンを消費
            let _ = self.src.next();

            let next_prec = match info.assoc {
                Assoc::Left => info.prec + 1,
                Assoc::Right => info.prec,
            };
            let rhs = self.expr(next_prec)?;
            lhs = Expression::Binary {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn primary(&mut self) -> ParseResult<Expression> {
        let tok = self.src.next().ok_or(SyntaxError::UnexpectedEof)?;

        let primary = match tok.kind {
            TokenKind::Num(n) => Expression::Value(n),
            TokenKind::Minus => {
                let expr = self.expr(prec::UNARY)?;
                Expression::Unary {
                    op: UnaryOp::Minus,
                    expr: Box::new(expr),
                }
            }
            TokenKind::LeftParen => {
                let expr = self.expr(prec::LOWEST)?;
                if self.expect(TokenKind::RightParen).is_err() {
                    return Err(SyntaxError::UnmatchedLeftParen(tok));
                }
                expr
            }
            TokenKind::Ident(name) => Expression::Var(name),
            _ => return Err(SyntaxError::UnexpectedToken(tok)),
        };

        Ok(primary)
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), SyntaxError> {
        match self.src.next() {
            Some(tok) if tok.kind == expected => Ok(()),
            Some(tok) => Err(SyntaxError::UnexpectedToken(tok)),
            None => Err(SyntaxError::UnexpectedEof),
        }
    }

    fn is_eof(&mut self) -> bool {
        self.src.peek().is_none()
    }
}
