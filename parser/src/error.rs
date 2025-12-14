use std::{error::Error, fmt};

use crate::{
    lexer::LexicalError,
    parser::SyntaxError,
    token::{Span, Spanned},
};

#[derive(Debug, PartialEq)]
pub enum CompilerError {
    Lexical(LexicalError),
    Syntax(SyntaxError),
}

impl Spanned for CompilerError {
    fn span(&self) -> Option<Span> {
        match self {
            Self::Lexical(e) => e.span(),
            Self::Syntax(e) => e.span(),
        }
    }
}

impl Error for CompilerError {}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::Lexical(e) => write!(f, "Lexical error: {}", e),
            CompilerError::Syntax(e) => write!(f, "Syntax error: {}", e),
        }
    }
}

impl From<LexicalError> for CompilerError {
    fn from(e: LexicalError) -> Self {
        CompilerError::Lexical(e)
    }
}

impl From<SyntaxError> for CompilerError {
    fn from(e: SyntaxError) -> Self {
        CompilerError::Syntax(e)
    }
}

/// エラーをソースコードとともに表示する
pub fn format_error<E: Spanned + fmt::Display>(e: &E, source: &str) -> String {
    if e.span().is_none() {
        return format!("{}\n{}", e, source);
    }

    // 表示形式:
    // エラー理由
    // ソース
    //    ^ エラー箇所
    //
    // 例:
    // Syntax error: Unexpected token: Plus
    // 1 + +
    //     ^
    let span = e.span().unwrap();
    let space = " ".repeat(span.start);
    let callet = "^".repeat(span.end - span.start);
    format!("{}\n{}\n{}{}", e, source, space, callet)
}
