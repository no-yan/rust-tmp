use std::{error::Error, fmt};

use crate::token::{Span, Spanned, Token};

pub type LexResult<T> = Result<T, LexicalError>;

#[derive(Debug, PartialEq)]
pub enum LexicalError {
    InvalidToken(String, Span),
    Eof, // センチネルエラー
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

    /// 入力全体をトークナイズし、トークン列を返す。
    /// 文字列をトークン化できない場合、エラーを返す。
    ///
    /// - 空白は読み飛ばす
    /// - 返却するトークン列に`Eof`は含めない
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

    /// 現在位置から1トークン読み進め、トークンを返す。
    /// EoFに到達した場合は、`LexicalError::Eof`を返す。
    /// トークナイズできない場合、`LexicalError::InvalidToken`を返す。
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
            '^' => Pow,
            '(' => LeftParen,
            ')' => RightParen,
            ';' => Semicolon,
            '{' => LeftBlock,
            '}' => RightBlock,

            '=' => {
                if self.eat('=') {
                    Eq
                } else {
                    Assign
                }
            }
            '!' if self.eat('=') => Neq,
            '<' => {
                if self.eat('=') {
                    LtEq
                } else {
                    Lt
                }
            }
            '>' => {
                if self.eat('=') {
                    GtEq
                } else {
                    Gt
                }
            }

            c if c.is_ascii_digit() => {
                let num = self.next_number();
                Num(num)
            }
            c if c.is_alphabetic() => {
                let ident = self.next_ident();
                match ident {
                    "if" => If,
                    "while" => While,
                    "for" => For,
                    _ => Ident(ident.to_string()),
                }
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

    /// 現在の読み取り位置にある文字を返す。
    /// `Lexer::bump`と異なり、この関数はポインタを移動しない。
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// 現在の読み取り位置にある文字を返し、ポインタを次の文字へ進める。
    pub fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;

        // 多バイト文字を考慮してutf8に変換
        self.pos += ch.len_utf8();
        Some(ch)
    }

    /// 現在の読み取り位置にある文字が`expected`と一致するか判定する。
    /// 一致した場合、ポインタを次の文字に進め、`true`を返す。
    /// 一致しない場合、ポインタを進めずに、`false`を返す。
    fn eat(&mut self, expected: char) -> bool {
        let ch = self.peek();
        if ch != Some(expected) {
            return false;
        }

        self.pos += ch.unwrap().len_utf8();
        true
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

    pub fn next_ident(&mut self) -> &str {
        // この関数に渡ってくる段階ですでに１文字目が読まれている
        let start = self.pos - 1;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() {
                self.bump();
            } else {
                break;
            }
        }

        &self.input[start..self.pos]
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Write;

    use super::*;

    fn format_lexer_test(name: &str, source: &str) -> String {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();

        let mut output = format!("=== {} ===\nsource: {}\n\n", name, source);
        for token in tokens {
            writeln!(
                output,
                "[{}..{}]\t{:?}",
                token.span.start, token.span.end, token.kind
            )
            .unwrap();
        }
        output.push('\n');
        output
    }

    #[test]
    fn lexer() {
        #[rustfmt::skip]
        const TESTS: &[(&str, &str)] = &[
            ("plus_operator",        "+"),
            ("number_literal",       "123"),
            ("plus_and_number",      "+ 123"),
            ("parenthesized_expr",   "(1)"),
            ("power_operator",       "^"),
            ("comparison_operators", "== != < <= > >="),
            ("assignment_statement", "x=1; x"),
            ("if_keyword",           "if"),
            ("if_statement",         "if (1>=0) {x=2;}"),
            ("while_loop",           "while(){}"),
            ("for_loop",             "for(i=0;i<1;i=i+1) {}"),
        ];

        let output = TESTS
            .iter()
            .map(|(name, source)| format_lexer_test(name, source))
            .collect::<String>();

        insta::assert_snapshot!(output);
    }
}
