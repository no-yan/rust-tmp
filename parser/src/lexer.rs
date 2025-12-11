use crate::error::Spanned;
use crate::token::{Span, Token};
use std::error::Error;
use std::fmt;

pub type LexResult<T> = Result<T, LexicalError>;

#[derive(Debug, PartialEq)]
pub enum LexicalError {
    InvalidToken(String, Span),
    Eof,
}

impl Error for LexicalError {}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::lexer::LexicalError::*;

        match self {
            InvalidToken(s, _) => write!(f, "Invalid token: {}", s),
            Eof => write!(f, "End of File"),
        }
    }
}

impl Spanned for LexicalError {
    fn span(&self) -> Option<Span> {
        match self {
            Self::InvalidToken(_, span) => Some(span.clone()),
            _ => None,
        }
    }
}

pub struct Lexer<'a> {
    pos: usize,
    input: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { pos: 0, input }
    }

    /// 入力全体をトークナイズし、トークン列 を返す
    /// - 空白は無視する
    /// - 連続する数字は一つのトークンとして扱う
    /// - TODO: 小数点のサポート
    /// - 不正な文字列があればErrを返す
    pub fn lex(&mut self) -> LexResult<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            match tok {
                Ok(t) => tokens.push(t),
                Err(LexicalError::Eof) => break,
                Err(e) => return Err(e),
            };
        }

        Ok(tokens)
    }

    /// 現在位置から次の1トークンを読む
    /// 不正な文字に遭遇したらErrを返す
    pub fn next_token(&mut self) -> Result<Token, LexicalError> {
        use crate::token::TokenKind::*;

        self.skip_whitespace();

        let start = self.pos;
        let char = match self.bump() {
            Some(c) => c,
            None => return Err(LexicalError::Eof),
        };

        let kind = match char {
            '+' => Plus,
            '-' => Minus,
            '*' => Mul,
            '/' => Div,
            '(' => LeftParen,
            ')' => RightParen,
            c if c.is_ascii_digit() => {
                let num = self.next_number();
                Num(num)
            }
            c => {
                return Err(LexicalError::InvalidToken(
                    c.to_string(),
                    Span {
                        start,
                        end: start + c.len_utf8(),
                    },
                ));
            }
        };
        let end = self.pos;

        Ok(Token {
            span: Span { start, end },
            kind,
        })
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.bump();
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// 1トークン読み進め、posを更新する
    pub fn bump(&mut self) -> Option<char> {
        let mut iter = self.input[self.pos..].chars();
        let ch = iter.next()?;

        // 多バイト文字を考慮してutf8に変換
        self.pos += ch.len_utf8();

        Some(ch)
    }

    pub fn next_number(&mut self) -> i32 {
        // この関数に渡ってくる段階ですでに１文字目が読まれている
        let start = self.pos - 1;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.bump();
            } else {
                break;
            }
        }

        let num_str = &self.input[start..self.pos];
        // Safety: ascii_digitの文字列で構成されているため、安全にパースできる
        num_str.parse::<i32>().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tok;
    use crate::token::TokenKind::*;

    #[test]
    fn plus() {
        let input = "+";
        let mut lexer = Lexer::new(input);
        let result = lexer.lex().unwrap();

        assert_eq!(result, vec![tok!(Plus, 0, 1),]);
    }

    #[test]
    fn complex() {
        let input = "+ 123";
        let mut lexer = Lexer::new(input);
        let result = lexer.lex().unwrap();

        assert_eq!(result, vec![tok!(Plus, 0, 1), tok!(Num(123), 2, 5),]);
    }

    #[test]
    fn number() {
        let input = "123";
        let mut lexer = Lexer::new(input);
        let result = lexer.lex().unwrap();

        assert_eq!(result, vec![tok!(Num(123), 0, 3),]);
    }

    #[test]
    fn parenthesis() {
        let input = "(1)";
        let mut lexer = Lexer::new(input);
        let result = lexer.lex().unwrap();

        assert_eq!(
            result,
            vec![
                tok!(LeftParen, 0, 1),
                tok!(Num(1), 1, 2),
                tok!(RightParen, 2, 3),
            ]
        );
    }
}
