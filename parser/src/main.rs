#![feature(str_split_whitespace_remainder)]
fn main() {
    // 標準入力から数値を受け取り、それらの合計を標準出力に出す
    let stdin = std::io::read_to_string(std::io::stdin()).unwrap();
    let split = stdin.split_whitespace();

    let sum :i32= split.filter_map(|s| s.parse::<i32>().ok()).sum();
    println!("{}", sum);
}
