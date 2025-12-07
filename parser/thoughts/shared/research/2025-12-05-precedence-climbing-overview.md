---
date: 2025-12-05T23:43:30+09:00
researcher: noyan
git_commit: fedbef000ef577a41eb67984f2111befa4e1eb42
branch: main
repository: no-yan/rust-tmp
topic: "Precedence climbing理論と実装調査"
tags: [research, codebase, parsing, precedence-climbing, rust, calculator]
status: complete
last_updated: 2025-12-05
last_updated_by: noyan
---

# Research: Precedence climbing理論と実装調査

**Date**: 2025-12-05T23:43:30+09:00  
**Researcher**: noyan  
**Git Commit**: fedbef000ef577a41eb67984f2111befa4e1eb42  
**Branch**: main  
**Repository**: no-yan/rust-tmp

## Research Question
precedence climbing（演算子優先度再帰）アルゴリズムの理論と実装手法を信頼できる外部資料で整理し、現行の Shunting Yard ベース電卓（`src/main.rs`）へ適用する際の課題を明記する。

## Summary
- precedence climbing は「再帰的に min_precedence を引数に取り、左右オペランドを結合していく」トップダウン式の演算子優先度パーサーで、Pratt parser の実装スタイルの一種として位置付けられる citeturn0search12turn0search11。
- 演算子の `precedence` と `associativity` をテーブル化し、`while lookahead_prec >= min_prec`（右結合なら `>`）のループで左結合/右結合を自然に扱うのが核心 citeturn0search0turn0search2。
- Pratt parser との違いは「nud/led のメソッド分派」を使うかどうかであり、precedence climbing はより手続き的で実装が短い傾向がある citeturn0search1。
- 既存コードは Shunting Yard で RPN 化→評価を行っており、AST を持たないため、precedence climbing を導入する場合は「式を直接評価」か「AST 構築＋評価」のいずれかを選ぶ必要がある（現行コード参照: `src/main.rs:42-145`）。
- hack/spec_metadata.sh はリポジトリに存在せず実行失敗したため、メタデータは手動で収集した（`git rev-parse`, `git branch`）。

## Detailed Findings

### 理論: precedence climbing の流れ
- **基本アルゴリズム**: `parse_expr(min_prec)` が最初のプライマリ（数値/括弧式/前置演算子を処理）を読み、続いて `lookahead` が演算子かつその優先度 `prec` が `>= min_prec` の間、`rhs = parse_expr(prec + is_left_assoc)` を再帰呼び出しし、`lhs = combine(lhs, op, rhs)` を繰り返す。左結合なら閾値 `prec + 1`、右結合なら `prec` を渡すことで結合性を制御する citeturn0search0。
- **Pratt との関係**: Pratt parser は各トークン型に `nud`/`led` を紐づけるディスパッチ指向の実装で、precedence climbing は「演算子表を引いて while ループで畳み込む」実装パターンとして説明されることが多い citeturn0search1。
- **歴史的位置付け**: Dijkstra の shunting-yard（1950年代末）とは別系譜で、Pratt (1973) の「トップダウン演算子優先順位解析」が母体。現在も clang の C パーサーや多数の小型インタプリタで利用されている citeturn0search11。
- **利点**: 実装が短く、再帰下降スタイルのまま優先度テーブルを追加するだけで多くの演算子を扱える。パースしながら即時評価（インタプリタ）と AST 構築のどちらにも応用可能 citeturn0search12。

### 実装パターン（参考スニペット）
```rust
fn parse_expr(min_prec: u8, tokens: &mut Peekable<I>) -> Expr {
    let mut lhs = parse_prefix(tokens); // 数値/括弧/前置演算子
    while let Some(op) = tokens.peek().and_then(op_info) {
        let prec = op.prec;
        let right_assoc = op.assoc == Assoc::Right;
        if prec < min_prec { break; }
        tokens.next(); // 消費
        let next_min = if right_assoc { prec } else { prec + 1 };
        let rhs = parse_expr(next_min, tokens);
        lhs = Expr::Binary{ op: op.kind, lhs: Box::new(lhs), rhs: Box::new(rhs) };
    }
    lhs
}
```
（擬似コード。`op_info` はトークン→優先度/結合性の表引き。右結合なら `prec` を渡し、左結合なら `prec+1` で再帰深さを制御する。実装原理は Norvell 2017 の記事に準拠。） citeturn0search0

### 参考実装と補足事項
- **前置/後置演算子**: parse_prefix が前置演算子を処理し、後置演算子（`!`, `++` など）はメインループ内で「優先度が非常に高い右結合中置」として扱う実例が多い citeturn0search2。
- **エラーハンドリング**: Andy Chu は「演算子表に存在しないトークンを見たらループを抜ける」ことで、エラー復旧を簡潔に保つ実装を紹介している citeturn0search1。
- **パフォーマンス**: Shunting Yard との計算量は同等 O(n)。再帰を使うためスタック深さは演算子数に比例するが、実用上問題ない規模とされる citeturn0search12。

### 現行コードベースへの適用課題
- **評価方式の選択**: 現行は「RPN へ変換 → スタック評価」（`Calculator::infix_to_rpn`, `evaluate_rpn`）で AST 不要。precedence climbing を採用する場合、(a) 「パースしながら即時評価」か (b) 「AST を組んでから評価」のいずれかへ設計を変える必要がある (`src/main.rs:42-145`)。
- **トークン設計**: `Token` に結合性情報が無く、`precedence()` も整数のみ (`src/token.rs:15-25`)。右結合演算子（累乗など）や前置符号を導入するには「優先度 + 結合性 + 種別（前置/中置/後置）」を保持する構造体化が必要。
- **単項演算子の文脈判定**: レキサは記号をそのまま `Plus/Minus` にしており（`src/lexer.rs` 参照）、前置符号を扱うには「直前がオペランドか？」というコンテキスト判定を追加するか、`Token::UnaryPlus/UnaryMinus` を生成する分岐が必要。
- **括弧処理**: precedence climbing では括弧は `parse_prefix` 側で `(` を消費し、対応する `)` までを再帰で処理する。現行の括弧検証ロジック（スタックでチェック）が消えるため、パース段階でのエラー伝播を整備する必要がある。
- **テスト資産の移行**: 既存テストは RPN 出力を前提せず算出値だけ検証している（`src/main.rs:94-162`）。アルゴリズム変更後も再利用できるが、パニック条件（括弧不一致など）の扱いが変わるため、エラーメッセージや `Result` の形を揃える追加テストが必要。

## Code References
- `src/main.rs:42-145` — Shunting Yard による中置→RPN 変換と評価。AST を持たない二段階処理。
- `src/token.rs:15-25` — 演算子優先度テーブル（結合性情報なし）。
- `src/lexer.rs` — 記号を文脈なしで `Plus/Minus/Mul/Div` にトークン化。
- `thoughts/shared/research/2025-12-05-unary-operator-parsing.md` — 単項演算子対応策の既存調査（参考・歴史的文脈）。

## Architecture Documentation
- 現行アーキテクチャは「Lexer → Shunting Yard (中置→RPN) → スタック評価」というパイプライン。演算子優先度は `Token::precedence` の整数で管理し、結合性は固定で左結合として扱う。precedence climbing へ移行する場合、パーサ段階で優先度と結合性を使い分ける再帰下降スタイルに変わる。

## Historical Context (from thoughts/)
- `thoughts/shared/research/2025-12-05-unary-operator-parsing.md` — 単項演算子導入時の候補手法（Pratt/再帰下降/Shunting Yard 拡張）を整理しており、本件の背景として利用できる。
- `thoughts/shared/research/2025-12-05-comment-documentation-review.md` — コメント品質レビュー。precedence climbing 直接の議論は無し。

## Related Research
- 現在の thoughts/shared/research 配下に precedence climbing 特化の資料は本ドキュメントのみ。

## Open Questions
- AST を構築するか、直接評価に留めるかの方針。
- 前置符号・累乗など右結合演算子を追加する際のトークン表現（構造体 or 列挙＋付随テーブル）をどこで保持するか。
- エラー報告をパニックにするか `Result` に統一するか、既存テストとの整合性をどう取るか。

