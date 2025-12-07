use std::error::Error;
use std::fmt;
use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Plus,
    Minus,
    Mul,
    Div,

    Num(i32),

    LeftParen,
    RightParen,
}

impl Token {
    /// Precedence for binary operators. Higher value == tighter binding.
    /// Returns None for non-binary tokens (numbers, parentheses, unary).
    pub fn precedence(&self) -> Option<u8> {
        use crate::Token::*;

        match self {
            Plus | Minus => Some(1),
            Mul | Div => Some(2),
            LeftParen | RightParen | Num(_) => None,
        }
    }
}

#[derive(Debug)]
pub enum LexicalError {
    InvalidToken(String),
    Eof,
}

impl Error for LexicalError {}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::LexicalError::*;

        match self {
            InvalidToken(s) => write!(f, "Invalid token: {}", s),
            Eof => write!(f, "End of File"),
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

    /// 入力全体をトークナイズし、Vec<Token> を返す
    /// - 空白は無視する
    /// - 連続する数字は一つのトークンとして扱う
    /// - TODO: 小数点のサポート
    /// - 不正な文字列があればErrを返す
    pub fn lex(&mut self) -> Result<Vec<Token>, Box<dyn Error>> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            match tok {
                Ok(t) => tokens.push(t),
                Err(LexicalError::Eof) => break,
                Err(e) => return Err(e.into()),
            };
        }

        Ok(tokens)
    }

    /// 現在位置から次の1トークンを読む
    /// 不正な文字に遭遇したらErrを返す
    pub fn next_token(&mut self) -> Result<Token, LexicalError> {
        use crate::Token::*;

        self.skip_whitespace();

        let char = match self.bump() {
            Some(c) => c,
            None => return Err(LexicalError::Eof),
        };

        let tok = match char {
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
            c => return Err(LexicalError::InvalidToken(c.to_string())),
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

    fn parse_expr(input: &str) -> Expression {
        let tokens = Lexer::new(input).lex().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

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

    #[test]
    fn parenthesis() {
        let expr = parse_expr("(1)");
        assert_eq!(expr, Expression::Num(1));
    }

    #[test]
    fn unary_minus() {
        let expr = parse_expr("-1");
        assert_eq!(
            expr,
            Expression::Unary {
                op: Token::Minus,
                expr: Box::new(Expression::Num(1))
            }
        );
    }

    #[test]
    fn parse_precedence() {
        let expr = parse_expr("1+2*3");
        // 1 + (2 * 3)
        assert_eq!(
            expr,
            Expression::Binary {
                lhs: Box::new(Expression::Num(1)),
                op: Token::Plus,
                rhs: Box::new(Expression::Binary {
                    lhs: Box::new(Expression::Num(2)),
                    op: Token::Mul,
                    rhs: Box::new(Expression::Num(3))
                })
            }
        );
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Binary {
        lhs: Box<Expression>,
        op: Token,
        rhs: Box<Expression>,
    },
    Unary {
        op: Token,
        expr: Box<Expression>,
    },
    Num(i32),
}

impl Expression {
    fn eval(&self) -> i32 {
        match self {
            Expression::Binary { lhs, op, rhs } => match op {
                Token::Plus => lhs.eval() + rhs.eval(),
                Token::Minus => lhs.eval() - rhs.eval(),
                Token::Mul => lhs.eval() * rhs.eval(),
                Token::Div => lhs.eval() / rhs.eval(),
                Token::Num(_) => unreachable!(""),
                Token::LeftParen => unreachable!(""),
                Token::RightParen => unreachable!(""),
            },
            Expression::Unary { op: _op, expr } => -expr.eval(),
            Expression::Num(v) => *v,
        }
    }
}

/// ```
/// E --> Exp(0)
/// Exp(p) --> P {B Exp(q)}
/// P --> U Exp(q) | "(" E ")" | v
/// B --> "+" | "-"  | "*" |"/" | "^" | "||" | "&&" | "="
/// U --> "-"
/// ```
pub struct Parser {
    src: Peekable<std::vec::IntoIter<Token>>,
}

impl Parser {
    fn new(src: Vec<Token>) -> Self {
        Self {
            src: src.into_iter().peekable(),
        }
    }

    fn parse(&mut self) -> Expression {
        let expr = self.expression(0);
        debug_assert!(self.src.next().is_none());

        expr
    }

    /// Pratt-style precedence climbing.
    fn expression(&mut self, min_prec: u8) -> Expression {
        let mut lhs = self.primary();

        loop {
            // Stop if next token is not a binary operator or has lower precedence.
            let op_token = match self.src.peek() {
                Some(t) if t.precedence().is_some() => t.clone(),
                _ => break,
            };

            let op_prec = op_token.precedence().unwrap();
            if op_prec < min_prec {
                break;
            }

            // consume operator
            self.src.next();

            // parse RHS with higher minimum precedence to enforce left associativity
            let rhs = self.expression(op_prec + 1);
            lhs = Expression::Binary {
                lhs: Box::new(lhs),
                op: op_token,
                rhs: Box::new(rhs),
            };
        }
        lhs
    }

    fn primary(&mut self) -> Expression {
        match self.src.next() {
            Some(Token::Num(v)) => Expression::Num(v),
            Some(Token::Minus) => {
                // unary minus binds tighter than any binary operator
                let expr = self.expression(3);
                Expression::Unary {
                    op: Token::Minus,
                    expr: Box::new(expr),
                }
            }
            Some(Token::LeftParen) => {
                let expr = self.expression(0);
                match self.src.next() {
                    Some(Token::RightParen) => expr,
                    _ => panic!("missing closing parenthesis"),
                }
            }
            other => panic!("unexpected token in primary: {:?}", other),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    let mut lexer = Lexer::new(&buffer);
    let tokens = lexer.lex()?;
    let mut parser = Parser::new(tokens);

    let v = parser.parse().eval();
    println!("{v}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Expression {
        let tokens = Lexer::new(input).lex().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn sum() {
        let result = parse("1 + 2");
        assert_eq!(
            result,
            Expression::Binary {
                lhs: Expression::Num(1).into(),
                op: Token::Plus,
                rhs: Expression::Num(2).into()
            }
        );
    }

    #[test]
    fn difference() {
        let result = parse("1 - 2 - 3");
        assert_eq!(
            result,
            Expression::Binary {
                lhs: Box::new(Expression::Binary {
                    lhs: Box::new(Expression::Num(1)),
                    op: Token::Minus,
                    rhs: Box::new(Expression::Num(2)),
                }),
                op: Token::Minus,
                rhs: Box::new(Expression::Num(3)),
            }
        );
    }

    #[test]
    fn sum_3_operand() {
        let result = parse("1 + 2 + 3");
        assert_eq!(
            result,
            Expression::Binary {
                lhs: Box::new(Expression::Binary {
                    lhs: Box::new(Expression::Num(1)),
                    op: Token::Plus,
                    rhs: Box::new(Expression::Num(2)),
                }),
                op: Token::Plus,
                rhs: Box::new(Expression::Num(3)),
            }
        );
    }

    #[test]
    fn prod_3_operand() {
        let result = parse("1*2*3");
        assert_eq!(
            result,
            Expression::Binary {
                lhs: Box::new(Expression::Binary {
                    lhs: Box::new(Expression::Num(1)),
                    op: Token::Mul,
                    rhs: Box::new(Expression::Num(2)),
                }),
                op: Token::Mul,
                rhs: Box::new(Expression::Num(3)),
            }
        );
    }

    #[test]
    fn process_with_priority() {
        let result = parse("1+2*3");
        assert_eq!(
            result,
            Expression::Binary {
                lhs: Box::new(Expression::Num(1)),
                op: Token::Plus,
                rhs: Box::new(Expression::Binary {
                    lhs: Box::new(Expression::Num(2)),
                    op: Token::Mul,
                    rhs: Box::new(Expression::Num(3)),
                })
            }
        );
    }

    #[test]
    fn without_space() {
        let result = parse("1+2");
        assert_eq!(
            result,
            Expression::Binary {
                lhs: Box::new(Expression::Num(1)),
                op: Token::Plus,
                rhs: Box::new(Expression::Num(2)),
            }
        );
    }

    #[test]
    fn with_paren() {
        let result = parse("(1+2)");
        assert_eq!(
            result,
            Expression::Binary {
                lhs: Box::new(Expression::Num(1)),
                op: Token::Plus,
                rhs: Box::new(Expression::Num(2)),
            }
        );
    }

    #[test]
    fn with_paren_precedence() {
        let result = parse("(1+2)*3");
        assert_eq!(
            result,
            Expression::Binary {
                lhs: Box::new(Expression::Binary {
                    lhs: Box::new(Expression::Num(1)),
                    op: Token::Plus,
                    rhs: Box::new(Expression::Num(2)),
                }),
                op: Token::Mul,
                rhs: Box::new(Expression::Num(3)),
            }
        );
    }

    #[test]
    #[should_panic]
    fn unmatched_left_paren() {
        let _ = parse("(1+2");
    }

    #[test]
    #[should_panic]
    fn unmatched_right_paren() {
        let _ = parse("1+2)");
    }

    #[test]
    fn unary_minus() {
        let result = parse("-1");
        assert_eq!(
            result,
            Expression::Unary {
                op: Token::Minus,
                expr: Box::new(Expression::Num(1))
            }
        );
    }
}

#[cfg(test)]
mod testss {
    use super::*;

    fn parse(src: Vec<Token>) -> i32 {
        let mut parser = Parser::new(src);
        parser.parse().eval()
    }

    #[test]
    fn sum() {
        let input = "1 + 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = parse(tokens);

        assert_eq!(result, 3);
    }

    #[test]
    fn difference() {
        let input = "1 - 2 - 3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = parse(tokens);

        assert_eq!(result, -4);
    }

    #[test]
    fn sum_3_operand() {
        let input = "1 + 2 + 3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = parse(tokens);

        assert_eq!(result, 6);
    }

    #[test]
    fn prod_3_operand() {
        let input = "1*2*3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = parse(tokens);

        assert_eq!(result, 6);
    }

    #[test]
    fn process_with_priority() {
        let input = "1+2*3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = parse(tokens);

        assert_eq!(result, 7);
    }

    #[test]
    fn without_space() {
        let input = "1+2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = parse(tokens);

        assert_eq!(result, 3)
    }

    #[test]
    fn with_paren() {
        let input = "(1+2)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = parse(tokens);

        assert_eq!(result, 3);
    }

    #[test]
    fn with_paren_precedence() {
        let input = "(1+2)*3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = parse(tokens);

        assert_eq!(result, 9);
    }

    #[test]
    #[should_panic]
    fn unmatched_left_paren() {
        let input = "(1+2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let _ = parse(tokens);
    }

    #[test]
    #[should_panic]
    fn unmatched_right_paren() {
        let input = "1+2)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let _ = parse(tokens);
    }

    #[test]
    fn unary_minus() {
        let input = "-1";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
        let result = parse(tokens);

        assert_eq!(result, -1);
    }
}
