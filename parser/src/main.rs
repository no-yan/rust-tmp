#![feature(str_split_whitespace_remainder)]
fn main() {
    // 標準入力から数値を受け取り、それらの合計を標準出力に出す
    let stdin = std::io::read_to_string(std::io::stdin()).unwrap();
    let split = stdin.split_whitespace();

    let sum = calc(split);
    println!("{}", sum);
}

fn calc<I, S>(input: I) -> i32
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    input
        .filter_map(|s: S| s.as_ref().parse::<i32>().ok())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum() {
        let input = "1 + 2".split_whitespace();
        let res = calc(input);

        assert_eq!(res, 3);
    }
}
