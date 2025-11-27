use std::error::Error;

fn main() {
    // 標準入力から数値を受け取り、それらの合計を標準出力に出す
    let stdin = std::io::read_to_string(std::io::stdin()).unwrap();
    let split = stdin.split_whitespace();

    let Ok(sum) = calc(split) else {
        return;
    };

    println!("{}", sum);
}

fn calc<I, S>(mut input: I) -> Result<i32, Box<dyn Error>>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let left = input.next().unwrap();
    let op = input.next().unwrap();
    let right = input.next().unwrap();

    let left = left.as_ref().parse::<i32>()?;
    let right = right.as_ref().parse::<i32>()?;

    let result = match op.as_ref() {
        "+" => left + right,
        "-" => left - right,
        "*" => left * right,
        "/" => left / right,
        _ => 0,
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum() {
        let input = "1 + 2".split_whitespace();
        let res = calc(input).unwrap();

        assert_eq!(res, 3);
    }
}
