use crate::lexer::LexicalError;
use crate::parser::SyntaxError;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum CompilerError {
    Lexical(LexicalError),
    Syntax(SyntaxError),
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
