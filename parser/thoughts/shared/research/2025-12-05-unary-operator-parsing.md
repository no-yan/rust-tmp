---
date: 2025-12-05T21:31:29+09:00
researcher: noyan
git_commit: fedbef000ef577a41eb67984f2111befa4e1eb42
branch: main
repository: no-yan/rust-tmp/parser
topic: "単項演算子をサポートするパーサー手法の調査"
tags: [research, codebase, parsing, unary-operator, rust, calculator]
status: complete
last_updated: 2025-12-05
last_updated_by: noyan
---

# Research: 単項演算子をサポートするパーサー手法の調査

**Date**: 2025-12-05T21:31:29+09:00  
**Researcher**: noyan  
**Git Commit**: fedbef000ef577a41eb67984f2111befa4e1eb42  
**Branch**: main  
**Repository**: no-yan/rust-tmp/parser

## Research Question

計算機パーサーを単項演算子（例: `-1`, `+1`）に対応させるために採用可能なパーサー手法を整理し、現行コードベースとの接点とデメリットを記録する。

## Summary

- 現行実装は Shunting Yard による中置→RPN 変換とスタック評価で、すべての演算子を**二項**として扱うため単項`+`/`-`を解釈できない (`src/main.rs:42-145`、`src/token.rs:2-25`)。
- トークナイザは `-` を常に `Token::Minus` として出力し文脈判定を持たない (`src/lexer.rs:53-80`)、評価側も2オペランドをポップする前提 (`src/main.rs:117-135`)。
- 単項を扱う代表的手法として (1) Pratt parser (トップダウン演算子優先順位)、(2) 再帰下降＋前置/後置規則分離、(3) Shunting Yard 拡張（符号トークンに前置優先度を与える）の3パターンを整理。各手法を利用する著名コンパイラ/インタプリタとデメリットを併記。
- 現行構成に最も小変更で適用できるのは「Shunting Yard 拡張」だが、符号と減算の曖昧性解決のため**トークン文脈判定**が追加必須となる。

## Detailed Findings

### 現行コードの構成
- `src/main.rs:42-145` — `Calculator` が Shunting Yard で RPN 生成 (`infix_to_rpn`) とスタック評価 (`evaluate_rpn`) を行う。演算子は `Plus/Minus/Mul/Div` のみ想定し、`Minus` は2オペランドを消費する二項演算に固定 (`apply_op` も2引数)。
- `src/lexer.rs:34-80` — 字句解析は `-` を単一トークン `Minus` として返すだけで、直前トークンによる単項/二項判別は行わない。
- `src/token.rs:2-25` — 優先順位は加減=1、乗除=2、括弧=3 で固定。単項専用の優先順位・結合性は定義されていない。
- `src/main.rs:250-258` — `unary_minus` テストが存在するが現行実装では評価時にオペランド不足でパニックする想定（単項対応の不在を示す既知のギャップ）。

### 単項演算子を扱う代表的パーサー手法

1. **Pratt Parser（トップダウン演算子優先順位）**
   - **方法**: トークンに前置/中置/後置ごとの「束縛力」を与え、再帰的に式を構築。`nud`(null denotation)で単項を処理し、`led`(left denotation)で二項を処理する。
   - **採用例**: Lua 5.x の式パーサー、rustc の型パーサーの一部（`parser.rs` 内部で Pratt 風の precedence climbing を使用）。
   - **デメリット**: 実装パターンの学習コストがやや高い。デバッグ時に「どの denotation が使われたか」を追う必要があり、ステップ実行がやや複雑。

2. **再帰下降パーサー（前置/後置/中置を文法で分離）**
   - **方法**: 文法を `Expr -> Additive`, `Additive -> Multiplicative (('+'|'-') Multiplicative)*`, `Multiplicative -> Unary (('*'|'/') Unary)*`, `Unary -> ('+'|'-') Unary | Primary` のように分解し、関数階層で再帰的に処理。
   - **採用例**: Go コンパイラの `cmd/compile/internal/syntax`（演算子優先度ごとに関数分割）、CPython の新パーサー（PEG ベースだが単項/二項を別生成規則で扱う）。
   - **デメリット**: 優先度レイヤごとに関数が増えがちで、演算子を追加するたびに複数箇所へ変更が必要。左再帰を避ける書き換えが必要で、文法拡張時に差し替えが生じる。

3. **Shunting Yard 拡張（符号を前置演算子として扱う）**
   - **方法**: トークナイズまたは変換段階で「前置符号」と「二項減算」を区別し、前置符号に最上位の優先度と右結合性を与える。`infix_to_rpn` でスタック条件を「結合性込み」で判定し、RPN では単項演算子を1オペランドで評価する。
   - **採用例**: Ruby MRI のレガシー式パーサー（独自スタックだが Shunting Yard 拡張に近い挙動）、多くの電卓実装（`dc`/`bc` の系譜を持つ簡易インタプリタ）。
   - **デメリット**: 「直前トークンが演算子・括弧開き・先頭」の場合のみ前置扱い、といったコンテキスト判定が必要で、現行の単純トークナイザにロジックを追加する必要がある。前置と二項で同一トークンを共有するとデバッグログが紛らわしい。

### 現行構成との接点
- **最小変更での延長**: Shunting Yard 拡張は既存の `infix_to_rpn`/`evaluate_rpn` を保持したまま、(a) 字句または変換段階で `UnaryMinus`/`UnaryPlus` を導入し、(b) `Token::precedence` に前置用の高優先度を追加し、(c) RPN 評価で1オペランド版の適用を実装する、という局所変更で適用可能。
- **大きな置換**: Pratt や再帰下降を選ぶ場合、`Calculator` の主要ロジックを置き換える形になり、テストスイートの構成も「生成された AST の評価」に合わせて再設計する必要がある。

## Code References
- `src/main.rs:22-34` — 単項演算子サポートを TODO として明記。
- `src/main.rs:42-145` — Shunting Yard による中置→RPN 変換と評価。すべて二項前提。
- `src/main.rs:117-135` — RPN 評価で常に2オペランドをポップする実装。
- `src/main.rs:250-258` — `unary_minus` テスト（現行実装では失敗想定）。
- `src/lexer.rs:34-80` — `-` を文脈判定なしで `Minus` トークン化。
- `src/token.rs:15-25` — 優先順位テーブル（単項演算子は未定義）。
- `thoughts/shared/research/2025-12-05-comment-documentation-review.md` — 既存コードコメントの評価。単項演算子に関する歴史的決定は未記載。

## Architecture Documentation
- 式解析は「字句解析 → Shunting Yard で RPN → スタック評価」という2段構成。AST は持たず、演算子優先度は `Token::precedence` に整数で埋め込み。
- エラーハンドリングは主にパニック（括弧不一致）と `Result` エラー（スタックアンダーフロー）で行われ、単項演算子の曖昧性解消や結合性判定は現時点で存在しない。

## Historical Context (from thoughts/)
- `thoughts/shared/research/2025-12-05-comment-documentation-review.md` — コメント品質レビュー。単項演算子拡張に関する過去の議論は見当たらない。

## Related Research
- 現時点で単項演算子に関する他の research ドキュメントは確認できなかった。

## Open Questions
- 前置符号を「Lexerで分離」するか「Shunting Yard 変換時に判定」するか、どちらを採用するか未決定。
- 右結合演算子（累乗など）を将来追加する場合、Shunting Yard 拡張で結合性をどう表現するか検討余地あり。
- AST を導入する設計に切り替えるか（再帰下降/Pratt）を判断するための実装コスト見積もりが未整理。
