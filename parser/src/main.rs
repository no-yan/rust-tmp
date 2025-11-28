use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;

    let mut lexer = Lexer::new(&buf);
    let tokens = lexer.lex()?;

    match process(tokens) {
        Ok(val) => println!("{}", val),
        Err(err) => eprintln!("{:?}", err),
    };

    Ok(())
}

fn process(input: Vec<Token>) -> Result<i32, Box<dyn Error>> {
    let [Token::Num(left), op, Token::Num(right)] = &input[..] else {
        unimplemented!();
    };

    let result = match op {
        Token::Plus => left + right,
        Token::Minus => left - right,
        Token::Mul => left * right,
        Token::Div => left / right,
        _ => return Err("unimplemented".into()),
    };

    Ok(result)
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Plus,
    Minus,
    Mul,
    Div,

    Num(i32),
    Eof, // レキサーの内部表現として使用する
}

struct Lexer<'a> {
    pos: usize,
    input: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { pos: 0, input }
    }

    // 入力全体をトークナイズし、Vec<Token> を返す
    // - 空白は無視する
    // - 連続する数字は一つのトークンとして扱う
    // - TODO: 小数点のサポート
    // - 不正な文字列があればErrを返す
    pub fn lex(&mut self) -> Result<Vec<Token>, Box<dyn Error>> {
        // 1. 結果用のベクタを用意
        // 2. self.peek() で次の文字列をみてループ
        //  - None なら終了
        //  - 空白ならself.bump()
        //  - +, -, *, / なら対応するtokenをpush
        //  - 数字なら、連続するdigitを一つのトークンとする
        //  - それ以外ならErrを返す

        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            match tok {
                Err(msg) => return Err(msg),
                Ok(Token::Eof) => break,
                Ok(t) => tokens.push(t)
            };
        }

        Ok(tokens)
    }

    // 現在位置から次の1トークンを読む
    // Tokenを読んだ分posを進める
    // 不正な文字に遭遇したらErrを返す
    pub fn next_token(&mut self) -> Result<Token, Box<dyn Error>> {
        use crate::Token::*;

        while let Some(c) = self.peek() {
          match c {
                c if c.is_whitespace() => {
                    self.bump();
                }
                '+' => {
                    self.bump();
                    return Ok(Plus);
                }
                '-' => {
                    self.bump();
                    return Ok(Minus);
                }
                '*' => {
                    self.bump();
                    return Ok(Mul);
                }
                '/' => {
                    self.bump();
                    return Ok(Div);
                }
                c if c.is_ascii_digit() => {
                    let num = self.next_number();
                    return Ok(Num(num));
                }
                c => return Err(format!("Invalid token: {}", c).into()),
            };
        };

        Ok(Eof)

    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    pub fn bump(&mut self) -> Option<char> {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, ch) = iter.next()?;

        if let Some((next_pos, _)) = iter.next() {
            self.pos += next_pos;
        } else {
            self.pos = self.input.len();
        }

        Some(ch)
    }

    pub fn next_number(&mut self) -> i32 {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.bump();
            } else {
                break;
            }
        }

        let num_str = &self.input[start..self.pos];
        // Safety: ascii_digitの文字列で構成されているため、安全にパースできます
        num_str.parse::<i32>().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Token::*;
    use super::*;

    #[test]
    fn sum() {
        let input = "1 + 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = process(tokens);

        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn tokenize_plus() {
        let input = "+";
        let mut lexer = Lexer::new(input);
        let result = lexer.lex().unwrap();

        assert_eq!(result, vec![Plus]);
    }

    #[test]
    fn tokenize_complex() {
        let input = "+ 123";
        let mut lexer = Lexer::new(input);
        let result = lexer.lex().unwrap();

        assert_eq!(result, vec![Plus, Num(123)]);
    }

    #[test]
    fn tokenize_number() {
        let input = "123";
        let mut lexer = Lexer::new(input);
        let result = lexer.lex().unwrap();

        assert_eq!(result, vec![Num(123)]);
    }

    #[test]
    fn tokenize_without_space() {
        let input = "1+2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = process(tokens).unwrap();

        assert_eq!(result, 3)
    }
}
