use crate::Token;
use std::error::Error;

pub struct Lexer<'a> {
    pos: usize,
    input: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { pos: 0, input }
    }

    /// 入力全体をトークナイズし、Vec<Token> を返す
    /// - 空白は無視する
    /// - 連続する数字は一つのトークンとして扱う
    /// - TODO: 小数点のサポート
    /// - 不正な文字列があればErrを返す
    pub fn lex(&mut self) -> Result<Vec<Token>, Box<dyn Error>> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            match tok {
                Token::Eof => break,
                t => tokens.push(t),
            };
        }

        Ok(tokens)
    }

    /// 現在位置から次の1トークンを読む
    /// 不正な文字に遭遇したらErrを返す
    pub fn next_token(&mut self) -> Result<Token, Box<dyn Error>> {
        use crate::token::Token::*;

        self.skip_whitespace();

        let char = match self.bump() {
            Some(c) => c,
            None => return Ok(Token::Eof),
        };

        let tok = match char {
            '+' => Plus,
            '-' => Minus,
            '*' => Mul,
            '/' => Div,
            c if c.is_ascii_digit() => {
                let num = self.next_number();
                Num(num)
            }
            c => return Err(format!("Invalid token: {}", c).into()),
        };

        Ok(tok)
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
    use crate::Token::*;

    #[test]
    fn plus() {
        let input = "+";
        let mut lexer = Lexer::new(input);
        let result = lexer.lex().unwrap();

        assert_eq!(result, vec![Plus]);
    }

    #[test]
    fn complex() {
        let input = "+ 123";
        let mut lexer = Lexer::new(input);
        let result = lexer.lex().unwrap();

        assert_eq!(result, vec![Plus, Num(123)]);
    }

    #[test]
    fn number() {
        let input = "123";
        let mut lexer = Lexer::new(input);
        let result = lexer.lex().unwrap();

        assert_eq!(result, vec![Num(123)]);
    }
}
