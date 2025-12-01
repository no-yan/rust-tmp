use crate::Hands;
use std::io::{self, Write};

pub enum DiscardAction {
    // スタンド（交換なし）
    Stand,
    // 指定位置を捨てる
    Discard(Vec<usize>),
}

pub fn prompt_discard(hands: &Hands) -> DiscardAction {
    // hands を表示して、入力に応じて Stand / Discard を返す
    println!("あなたの手札:\n{hands}");

    let input = prompt("交換したいカードの番号をスペース区切りで入力してください。\n交換しない場合は Enter を押してください。");

    if input == "\n" {
        println!("交換しませんでした。");
        DiscardAction::Stand
    } else {
        let picked: Vec<_> = input
            .split_whitespace()
            .filter_map(|s| s.parse::<usize>().ok())
            .filter(|&i| (1..13).contains(&i))
            .map(|i| i - 1) // 0-indexed
            .collect();

        let msg = picked
        .iter()
        .map(|i| format!("{}", hands[*i]))
        .collect::<Vec<_>>()
        .join(", ");
        println!("交換: {}を捨てました。\n", msg);

        DiscardAction::Discard(picked)
    }
}

fn prompt(ask: &str) -> String {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("{ask}");

    print!("> ");
    stdout.flush().unwrap();

    let mut input = String::new();
    if stdin.read_line(&mut input).is_err() {
        eprintln!("入力エラー");
    }

    input
}
