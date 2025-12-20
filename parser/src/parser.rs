use std::{error::Error, fmt, iter::Peekable};

use crate::{
    ast::{Assoc, BinaryOp, Expression, For, If, Program, Statement, UnaryOp, While, prec},
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
            Self::UnmatchedLeftParen(_) => write!(f, "Unmatched left parenthesis"),
            Self::UnexpectedToken(tok) => write!(f, "Unexpected token: {:?}", tok.kind),
            Self::InvalidAssignmentTarget(tok) => {
                write!(f, "Invalid assignment target: {:?}", tok.kind)
            }
            Self::UnexpectedEof => write!(f, "Unexpected end of file"),
        }
    }
}

impl Spanned for SyntaxError {
    fn span(&self) -> Option<Span> {
        match self {
            Self::UnmatchedLeftParen(tok)
            | Self::UnexpectedToken(tok)
            | Self::InvalidAssignmentTarget(tok) => Some(tok.span.clone()),
            Self::UnexpectedEof => None,
        }
    }
}

pub type ParseResult<T> = Result<T, SyntaxError>;

/// 計算式を構文解析し、[`Expression`]を構築するパーサー。
///
/// ## 仕様
/// ### サポートする演算子
///
/// - 二項演算子: `+`, `-`, `*`, `/`, `^`, `>`, `<`, `>=`, `<=`, `=`
/// - 単項演算子: `-`
///
/// ### 優先順位
///
/// 下に行くほど優先度が高い
/// 1. `=`
/// 2. `<` `<=` `>` `>=`
/// 3. `+` `-`
/// 4. `*` `/`
/// 5. 単項`-`
/// 6. `^`
/// 7. `(` `)`
///
/// ### 結合性
///
/// - 右結合: `^` `=`
/// - 左結合: その他全て
///
/// ### 文法
///
/// Program -> Stmt { Stmt }
/// Stmt    -> If | While | For | E ";"
/// If      -> "if" "(" E ")" "{" { Stmt } "}"
/// While   -> "while" "(" E ")" "{" { Stmt } "}"
/// For     -> "for" "(" [ E ] ";" [ E ] ";" [ E ] ")" "{" { Stmt } "}"
///
/// E       -> Expr(0)
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
        // 文は再帰下降パーサで、式はPrecedence climbingパーサで解析する
        self.program()
    }

    fn program(&mut self) -> ParseResult<Program> {
        let mut body = vec![];

        body.push(self.stmt()?);
        while !self.is_eof() {
            body.push(self.stmt()?);
        }

        Ok(Program { body })
    }

    fn stmt(&mut self) -> ParseResult<Statement> {
        let tok = self.src.peek().ok_or(SyntaxError::UnexpectedEof)?;

        match tok.kind {
            TokenKind::If => Ok(self.r#if()?),
            TokenKind::While => Ok(self.r#while()?),
            TokenKind::For => Ok(self.r#for()?),
            TokenKind::LeftBlock => Ok(self.block_statement()?),
            _ => {
                let expr = self.expr(prec::LOWEST)?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Statement::ExpressionStatement(expr))
            }
        }
    }

    fn r#if(&mut self) -> ParseResult<Statement> {
        // If      -> "if" "(" E ")" "{" { Stmt } "}"
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

    fn r#while(&mut self) -> ParseResult<Statement> {
        // While   -> "while" "(" E ")" "{" { Stmt } "}"
        self.src.next();
        self.expect(TokenKind::LeftParen)?;
        let cond = self.expr(prec::LOWEST)?;
        self.expect(TokenKind::RightParen)?;

        self.expect(TokenKind::LeftBlock)?;

        let mut body = vec![];
        while let Some(tok) = self.src.peek()
            && tok.kind != TokenKind::RightBlock
        {
            body.push(self.stmt()?);
        }

        self.expect(TokenKind::RightBlock)?;

        Ok(Statement::While(While { cond, body }))
    }

    fn r#for(&mut self) -> ParseResult<Statement> {
        // For     -> "for" "(" [ E ] ";" [ E ] ";" [ E ] ")" "{" { Stmt } "}"
        self.src.next();
        self.expect(TokenKind::LeftParen)?;

        let init = match self.src.peek() {
            Some(tok) if tok.kind != TokenKind::Semicolon => Some(self.expr(prec::LOWEST)?),
            _ => None,
        };
        self.expect(TokenKind::Semicolon)?;

        let cond = match self.src.peek() {
            Some(tok) if tok.kind != TokenKind::Semicolon => Some(self.expr(prec::LOWEST)?),
            _ => None,
        };
        self.expect(TokenKind::Semicolon)?;

        let update = match self.src.peek() {
            Some(tok) if tok.kind != TokenKind::RightParen => Some(self.expr(prec::LOWEST)?),
            _ => None,
        };

        self.expect(TokenKind::RightParen)?;
        self.expect(TokenKind::LeftBlock)?;

        let mut body = vec![];
        while let Some(tok) = self.src.peek()
            && tok.kind != TokenKind::RightBlock
        {
            body.push(self.stmt()?);
        }

        self.expect(TokenKind::RightBlock)?;

        Ok(Statement::For(For {
            init,
            cond,
            update,
            body,
        }))
    }

    fn block_statement(&mut self) -> ParseResult<Statement> {
        self.expect(TokenKind::LeftBlock)?;

        let mut body = vec![];
        while let Some(tok) = self.src.peek()
            && tok.kind != TokenKind::RightBlock
        {
            body.push(self.stmt()?);
        }

        self.expect(TokenKind::RightBlock)?;

        Ok(Statement::BlockStatement(body))
    }

    fn expr(&mut self, min_prec: u8) -> ParseResult<Expression> {
        // Precedence climbing algorithmを使用してパースを行う。
        // see: https://www.engr.mun.ca/~theo/Misc/exp_parsing.htm#climbing

        let mut lhs = self.primary()?;

        while let Some(tok) = self.src.peek() {
            let Ok(op) = BinaryOp::try_from(&tok.kind) else {
                break;
            };
            let info = op.op_info();

            if !info.binds_at(min_prec) {
                break;
            }

            // 代入演算子の場合、左辺が変数であることを保証する。
            // 構文規則ではExprとしてパースされるが、L-valueである必要がある。
            if matches!(op, BinaryOp::Assign) && !matches!(lhs, Expression::Var(_)) {
                // 例: "1 = 2"

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

    /// 次のトークンが期待した`TokenKind`であることを確認し、消費する。
    /// 異なる種類、またはEoFの場合はエラーを返す。
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
