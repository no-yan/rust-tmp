use std::error::Error;
fn main() {
    let stdin = std::io::read_to_string(std::io::stdin()).unwrap();
    let input = stdin.split_whitespace();

    match process(input) {
        Ok(val) => println!("{}", val),
        Err(err) => eprintln!("{:?}", err),
    }
}

fn process<I, S>(mut input: I) -> Result<i32, Box<dyn Error>>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let left = input.next().ok_or("error")?;
    let op = input.next().ok_or("error")?;
    let right = input.next().ok_or("error")?;

    let left = left.as_ref().parse::<i32>()?;
    let right = right.as_ref().parse::<i32>()?;

    let result = match op.as_ref() {
        "+" => left + right,
        "-" => left - right,
        "*" => left * right,
        "/" => left / right,
        _ => return Err("unimplemented".into()),
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum() {
        let input = "1 + 2".split_whitespace();
        let result = process(input);

        assert_eq!(result.unwrap(), 3);
    }
}
