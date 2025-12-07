use crate::lexer::LexicalError;
use crate::token::Token;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum CompilerError {
    Lexical(LexicalError),
    Syntax(SyntaxError),
}

#[derive(Debug, PartialEq)]
pub enum SyntaxError {
    UnmatchedLeftParen,
    UnexpectedToken(Token),
}

impl Error for CompilerError {}
impl Error for SyntaxError {}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::Lexical(e) => write!(f, "Lexical error: {}", e),
            CompilerError::Syntax(e) => write!(f, "Syntax error: {}", e),
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::UnmatchedLeftParen => write!(f, "Unmatched left parenthesis"),
            SyntaxError::UnexpectedToken(tok) => write!(f, "Unexpected token: {:?}", tok),
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
