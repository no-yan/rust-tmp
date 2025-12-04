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

// TOOD:
// ゴール: 優先順位をつけた演算をサポートする
// 1. 演算子の計算順序を書き下す
// 2. 演算子の処理の実装方法を9ccで確認する
// 3. goの実装手順を確認する
// 4. +だけの演算をサポートするShunting yard algorithmを実装する
// 5. -をサポート
// 6. トークンの優先順位をサポートし、演算規則をサポート
// 7. カッコをサポートする
//
// Stackの実装手順
// 実装方針の調査
//
// Shunting yard algorithm
//
// 演算子がオペランドの間におかれる構文を解析するアルゴリズム。得られる出力は逆ポーランド記法になる。
// 以下の手順で解析する:
// 出力用のベクタと、演算子を一時的に保管するStackを用意する。
// 入力からトークンをpopする
// 1. 数値: 出力にpush
// 2. 演算子:
//      Stackのtopがより高い優先順位を持つ場合:
//          stackをpop, 出力にpush
//      Stackにpush
// 3. 入力が空になったら:
//      Stackを空になるまでpopし出力にpushする
//
//  四則演算を行うには、演算子を評価するときに、stackから2つpopして、それらを計算することで結果を得られる
fn process(input: Vec<Token>) -> Result<i32, Box<dyn Error>> {
    // 1. 計算の順序
    //
    // (*, /) → (+, -)
    //
    // 2. EBNF
    //
    // Expr      = UnaryExpr
    //           | Expr BinaryOp Expr
    // BinaryOp  = AddOp | MulOp
    // AddOp     = "+" | "-"
    // MulOp     = "*" | "/"
    // UnaryExpr = Num

    let rpn = infix_to_rpn(input);
    evaluate_rpn(rpn)
}

fn apply_op(tok: Token, lhs: i32, rhs: i32) -> i32 {
    match tok {
        Token::Plus => lhs + rhs,
        Token::Minus => lhs - rhs,
        Token::Mul => lhs * rhs,
        Token::Div => lhs / rhs,
        _ => unimplemented!(),
    }
}

fn evaluate_rpn(rpn: Vec<Token>) -> Result<i32, Box<dyn Error>> {
    let mut stack: Vec<i32> = Vec::new();

    for tok in rpn.into_iter() {
        match tok {
            Token::Num(n) => stack.push(n),
            op @ Token::Plus | op @ Token::Minus | op @ Token::Mul | op @ Token::Div => {
                let rhs = stack.pop().ok_or("stack underflow (rhs)")?;
                let lhs = stack.pop().ok_or("stack underflow (lhs)")?;
                let val = apply_op(op, lhs, rhs);
                stack.push(val);
            }
            _ => unreachable!(""),
        }
    }

    assert!(stack.len() == 1);
    Ok(stack[0])
}

fn infix_to_rpn(input: Vec<Token>) -> Vec<Token> {
    let mut output = vec![];
    let mut ops_stack: Vec<Token> = vec![];

    // 入力が空になるまで、次のことを続ける
    // 1. トークンを一つ読み出す
    // 2. トークン種別に応じて次のことを行う
    //    a. 数値: output.push
    //    b. 演算子: ops_stack.push
    //
    // 空になったら、出力を評価する
    // 出力に入った演算子は動かないため、これは次のように最適化できる
    // - 演算子をoutputにpushしようとするたび、その代わりにoutputを2回popし、演算を適用する
    // - e.g.
    //      Output: [3, 3], Op: "+"　→ Output: [6]
    //

    let input = input.into_iter();
    for tok in input {
        match tok {
            Token::Num(_) => output.push(tok),
            Token::Plus | Token::Minus | Token::Mul | Token::Div => {
                while let Some(op) = ops_stack.last()
                    && op.precedence() >= tok.precedence()
                {
                    let op = ops_stack.pop().unwrap();
                    output.push(op);
                }
                ops_stack.push(tok);
            }
            _ => todo!(),
        }
    }
    while let Some(op) = ops_stack.pop() {
        output.push(op);
    }

    output
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

impl Token {
    // The higher precedes the lower.
    fn precedence(&self) -> i32 {
        use crate::Token::*;

        match self {
            Plus | Minus => 1,
            Mul | Div => 2,
            _ => 999,
        }
    }
}

struct Lexer<'a> {
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
        use crate::Token::*;

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
    fn difference() {
        let input = "1 - 2 - 3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = process(tokens);

        assert_eq!(result.unwrap(), -4);
    }

    #[test]
    fn sum_3_operand() {
        let input = "1 + 2 + 3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = process(tokens);

        assert_eq!(result.unwrap(), 6);
    }

    #[test]
    fn prod_3_operand() {
        let input = "1*2*3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = process(tokens);

        assert_eq!(result.unwrap(), 6);
    }

    #[test]
    fn process_with_priority() {
        let input = "1+2*3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = process(tokens);

        assert_eq!(result.unwrap(), 7);
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
