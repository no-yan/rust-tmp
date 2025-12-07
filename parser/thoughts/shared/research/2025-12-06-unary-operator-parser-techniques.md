---
date: 2025-12-06T03:53:39Z
researcher: noyan
git_commit: fedbef000ef577a41eb67984f2111befa4e1eb42
branch: main
repository: rust-tmp/parser
topic: "単項演算子をサポートするパーサー技術の調査"
tags: [research, parser, unary-operators, pratt-parsing, recursive-descent, shunting-yard, precedence-climbing]
status: complete
last_updated: 2025-12-06
last_updated_by: noyan
last_updated_note: "Goのパーサーがハイブリッド（Recursive Descent + Precedence Climbing）であることを明確化"
---

# Research: 単項演算子をサポートするパーサー技術の調査

**Date**: 2025-12-06T03:53:39Z
**Researcher**: noyan
**Git Commit**: fedbef000ef577a41eb67984f2111befa4e1eb42
**Branch**: main
**Repository**: rust-tmp/parser

## Research Question

現在のShunting-yardアルゴリズムベースの計算機パーサーを単項演算子（-1, +1）に対応させたい。どのパーサー技術を採用すべきか判断するため、各アルゴリズムの詳細、使用している著名なコンパイラ、デメリットについて調査する。

## Summary

単項演算子をサポートするパーサーには主に4つのアプローチがある：

| アルゴリズム | 特徴 | 推奨度 |
|-------------|------|--------|
| Modified Shunting-yard | 既存実装への最小限の変更 | △ |
| Recursive Descent | 文法を直接コードに反映 | ○ |
| Precedence Climbing | シンプルで実用的 | ○ |
| Pratt Parsing | Precedence Climbingの一般化 | ◎ |

**結論**: Pratt ParsingまたはPrecedence Climbingを推奨。両者は本質的に同じアルゴリズムであり、Pratt Parsingはより拡張性が高い。

**重要な発見**: GCC、Clang、Goなど主要コンパイラは **ハイブリッドアプローチ** を採用している：
- 文・宣言 → Recursive Descent
- 式 → Precedence Climbing（≒ Pratt Parsing）

## Detailed Findings

---

### 1. Modified Shunting-yard Algorithm

#### 概要
現在使用中のShunting-yardアルゴリズムを拡張して単項演算子をサポートする方法。

#### アルゴリズム

単項演算子と二項演算子を区別するルール：
- **二項演算子として扱う場合**: オペランドまたは `)` の直後
- **単項演算子として扱う場合**: 演算子の直後、`(` の直後、または入力の先頭

```
前のトークンを確認:
  - オペランドまたは ')' → 二項演算子
  - 演算子または '(' または先頭 → 単項演算子
```

実装時の重要なルール：
> 単項演算子を処理する際、他の単項演算子のみをpop可能。二項演算子はpopしてはならない（優先順位に関わらず）

これは `a ^ -b` のような式を正しく処理するため。`^`（べき乗）が `-`（否定）より高い優先順位を持っていても、`-b` を先に評価する必要がある。

#### 使用しているプロジェクト

- 電卓アプリケーション
- シンプルな数式パーサー
- スプレッドシートの数式エンジン

#### デメリット

- 単項/二項の判定ロジックが複雑化する
- 出力（RPN）で単項と二項を区別する必要がある（例: `-` を `_` で表現）
- 連鎖した単項演算子（`--1`, `+-1`）の処理が難しい
- 後置単項演算子（`!`）を追加する場合、さらに複雑化
- アルゴリズムの本質から外れたハック的な対応になりがち

---

### 2. Recursive Descent Parsing

#### 概要
文法規則をそのまま再帰関数として実装する手法。各非終端記号に対応する関数を定義する。

#### アルゴリズム

単項演算子を含む文法例：
```
expression → term
term       → factor ( ( "-" | "+" ) factor )*
factor     → unary ( ( "/" | "*" ) unary )*
unary      → ( "!" | "-" ) unary | primary
primary    → NUMBER | "(" expression ")"
```

各優先順位レベルに対応する関数を定義：

```rust
fn unary(&mut self) -> Expr {
    if self.match_token(&[Token::Bang, Token::Minus]) {
        let operator = self.previous();
        let right = self.unary();  // 再帰呼び出しでネストに対応
        return Expr::Unary(operator, Box::new(right));
    }
    self.primary()
}
```

単項演算子は再帰的に自身を呼び出すため、`--1` や `!!true` のようなネストを自然に処理できる。

#### 使用している著名なコンパイラ

> **注意**: 多くの主要コンパイラは **ハイブリッドアプローチ** を採用している。
> - 文（statement）、宣言 → Recursive Descent
> - 式（expression） → Precedence Climbing / Pratt Parsing

| コンパイラ | 文の解析 | 式の解析 |
|-----------|---------|---------|
| **GCC** | Recursive Descent | Precedence Climbing |
| **Clang/LLVM** | Recursive Descent | Precedence Climbing |
| **Go** | Recursive Descent | **Precedence Climbing** |
| **V8 (JavaScript)** | 手書きパーサー | - |
| **SpiderMonkey** | 手書きパーサー | - |

**Goの式パーサーの例** (`go/parser/parser.go`):

```go
// これはPrecedence Climbingのパターン
for (p.tok == _Operator || p.tok == _Star) && p.prec > prec {
    t := new(Operation)
    t.Op = p.op
    tprec := p.prec
    p.next()
    t.X = x
    t.Y = p.binaryExpr(nil, tprec)  // 現在の優先順位で再帰
    x = t
}
```

この実装は典型的なPrecedence Climbingである：
- `p.prec > prec` で優先順位を比較
- `p.binaryExpr(nil, tprec)` で現在の優先順位を渡して再帰

#### デメリット

- 優先順位レベルごとに関数が必要（多くの優先順位レベル = 多くの関数）
- 左再帰の文法は直接書けない（変換が必要）
- Rustでは末尾呼び出し最適化がないため、深いネストでスタックオーバーフローの可能性
- 優先順位の追加・変更時にコード構造の変更が必要
- 優先順位がコード構造に暗黙的に埋め込まれる

---

### 3. Precedence Climbing

#### 概要
Eli Benderskyが「最もシンプル」と評する実用的なアルゴリズム。式を優先順位レベルでネストされた部分式として扱う。

#### アルゴリズム

核心となるアイデア：
> 次のアトムを消費し、後続の演算子を確認。演算子の優先順位が現在のステップで許容される最低値より低ければ、リターンする。

```rust
fn compute_expr(&mut self, min_prec: i32) -> i32 {
    let mut result = self.compute_atom();

    while let Some(op) = self.peek_operator() {
        if op.precedence() < min_prec {
            break;
        }
        self.advance();

        // 結合性の処理
        let next_min_prec = if op.is_left_associative() {
            op.precedence() + 1
        } else {
            op.precedence()
        };

        let rhs = self.compute_expr(next_min_prec);
        result = self.apply_op(op, result, rhs);
    }
    result
}
```

結合性の処理がエレガント：
- **左結合**: `min_prec + 1` で再帰（同じ優先順位の演算子を先に処理させない）
- **右結合**: `min_prec` のまま再帰（同じ優先順位の演算子も処理させる）

#### 単項演算子の扱い

Eli Benderskyの記事では単項演算子は「アトムとして扱うことで簡単に組み込める」と述べているが、詳細は省略されている。

基本的なアプローチ：
```rust
fn compute_atom(&mut self) -> i32 {
    if self.match_token(Token::Minus) {
        return -self.compute_atom();  // 単項マイナス
    }
    if self.match_token(Token::LeftParen) {
        let result = self.compute_expr(0);
        self.expect(Token::RightParen);
        return result;
    }
    self.consume_number()
}
```

#### 使用している著名なコンパイラ

| コンパイラ | 備考 |
|-----------|------|
| **Clang** | `Parser::ParseExpression` でPrecedence Climbingを使用 |
| **GCC** | 式の解析部分で演算子優先順位パーサーを使用 |
| **Go** | `binaryExpr` 関数でPrecedence Climbingを使用（上記コード参照） |

#### デメリット

- 単項演算子のサポートが基本アルゴリズムに含まれていない（拡張が必要）
- 二項演算子より高い優先順位を持つ単項演算子の処理が複雑
- 後置演算子の追加には追加のロジックが必要
- 拡張性がPratt Parsingより劣る

---

### 4. Pratt Parsing (Top-Down Operator Precedence)

#### 概要
1973年にVaughan Prattが提案。Precedence Climbingの一般化であり、コマンドパターンを用いてモジュラー性を高めた手法。

#### アルゴリズム

核心概念：
- **Null Denotation (nud)**: 前置位置での解析（左に何もない状態）
- **Left Denotation (led)**: 中置位置での解析（左に式がある状態）
- **Binding Power (bp)**: 演算子の結合力（優先順位に相当）

演算子のカテゴリ：
| カテゴリ | binding power | 例 |
|---------|--------------|-----|
| 前置（prefix） | 右bpのみ | `-x`, `!x` |
| 中置（infix） | 左右両方 | `+`, `*` |
| 後置（postfix） | 左bpのみ | `x!`, `x[]` |

```rust
fn expr(&mut self, min_bp: u8) -> Expr {
    // 1. nud: 前置演算子またはアトムを処理
    let mut lhs = match self.advance() {
        Token::Num(n) => Expr::Num(n),
        Token::Minus => {
            let rhs = self.expr(PREFIX_BP);  // 高いbpで再帰
            Expr::Unary(Op::Neg, Box::new(rhs))
        }
        Token::LeftParen => {
            let inner = self.expr(0);
            self.expect(Token::RightParen);
            inner
        }
        _ => panic!("unexpected token"),
    };

    // 2. led: 中置演算子を処理
    loop {
        let op = match self.peek() {
            Some(t) if is_operator(t) => t,
            _ => break,
        };

        let (l_bp, r_bp) = infix_binding_power(op);
        if l_bp < min_bp {
            break;
        }

        self.advance();
        let rhs = self.expr(r_bp);
        lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
    }

    lhs
}
```

結合性はbinding powerの非対称性で表現：
- **左結合**: `l_bp < r_bp` (例: `+` は (1, 2))
- **右結合**: `l_bp > r_bp` (例: `^` は (4, 3))

#### Precedence Climbingとの関係

> "Pratt parsing is a generalization of precedence climbing"
> — Theodore Norvell

両者は本質的に同じアルゴリズムだが、Pratt Parsingは：
- ルックアップテーブル（`nullComm`, `leftComm`）を使用
- 新しい演算子の追加がテーブルへのエントリ追加だけで完了
- コアのパース関数を変更せずに拡張可能

#### 使用している著名なプロジェクト

| プロジェクト | 備考 |
|-------------|------|
| **rust-analyzer** | Rustの言語サーバー。本番環境でPratt Parserを使用 |
| **Crafting Interpreters (Lox)** | 教育用言語だが業界で広く参照されている |
| **多くのJavaScript処理系** | 式の解析に使用 |

#### デメリット

- 概念の理解に時間がかかる（「メビウスの輪」と表現されることも）
- 再帰とループの混合で動作を追跡しづらい
- デバッグ時に状態の把握が難しい
- 文法規則が明示的でないため、文法の正確性を検証しづらい
- 小規模なパーサーにはオーバーエンジニアリングになる可能性

---

## Comparison Table

| 項目 | Modified Shunting-yard | Recursive Descent | Precedence Climbing | Pratt Parsing |
|------|------------------------|-------------------|---------------------|---------------|
| **実装の容易さ** | 中（既存改修） | 高（文法→コード） | 高（シンプル） | 中（概念理解必要） |
| **単項演算子** | 難 | 易 | 中 | 易 |
| **優先順位拡張** | 易 | 難 | 易 | 易 |
| **後置演算子** | 難 | 中 | 難 | 易 |
| **コード可読性** | 低 | 高 | 中 | 中 |
| **モジュラー性** | 低 | 低 | 中 | 高 |
| **本番使用実績** | 中 | 高 | 高 | 高 |

---

## Recommendation

現在の計算機パーサーには **Pratt Parsing** への移行を推奨する。

**理由**:

1. **概念的な近さ**: 現在のShunting-yardもbinding power（優先順位）の概念を使用しており、移行が自然
2. **拡張性**: 単項演算子、後置演算子、三項演算子などを同じフレームワークで追加可能
3. **実績**: rust-analyzerなど本番環境での使用実績あり
4. **学習リソース**: [Simple but Powerful Pratt Parsing](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html) がRustでの優れた実装ガイド

**代替案**:

| 選択肢 | 適したケース |
|--------|-------------|
| **Precedence Climbing** | Prattの概念が難しいと感じる場合。本質は同じ |
| **Recursive Descent** | 将来的に文（statement）の解析も必要な場合 |
| **Modified Shunting-yard** | 変更を最小限に抑えたい場合のみ（非推奨） |

---

## Code References

現在の実装:
- `src/main.rs:38-68` - Shunting-yard実装（`Calculator::calc`）
- `src/main.rs:70-115` - 中置→RPN変換（`infix_to_rpn`）
- `src/token.rs:14-26` - 優先順位定義（`Token::precedence`）

---

## Sources

### Pratt Parsing
- [Simple but Powerful Pratt Parsing](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html) - rust-analyzer作者による解説（Rust実装）
- [Pratt Parsers: Expression Parsing Made Easy](https://journal.stuffwithstuff.com/2011/03/19/pratt-parsers-expression-parsing-made-easy/) - Bob Nystrom（Java実装）
- [Top-Down operator precedence parsing - Eli Bendersky](https://eli.thegreenplace.net/2010/01/02/top-down-operator-precedence-parsing)

### Precedence Climbing
- [Parsing expressions by precedence climbing - Eli Bendersky](https://eli.thegreenplace.net/2012/08/02/parsing-expressions-by-precedence-climbing) - 「最もシンプル」と評価
- [From Precedence Climbing to Pratt Parsing](https://www.engr.mun.ca/~theo/Misc/pratt_parsing.htm) - 両者の関係を解説
- [Pratt Parsing and Precedence Climbing Are the Same Algorithm](https://www.oilshell.org/blog/2016/11/01.html)

### Recursive Descent
- [Parsing Expressions · Crafting Interpreters](https://craftinginterpreters.com/parsing-expressions.html)
- [Parsing Expressions by Recursive Descent](https://www.engr.mun.ca/~theo/Misc/exp_parsing.htm)

### Shunting-yard Modifications
- [The Shunting-Yard Algorithm – Nathan Reed](https://www.reedbeta.com/blog/the-shunting-yard-algorithm/)
- [Stack Overflow: How can I modify my Shunting-Yard Algorithm so it accepts unary operators?](https://stackoverflow.com/questions/1593080/how-can-i-modify-my-shunting-yard-algorithm-so-it-accepts-unary-operators)

### Compiler Implementations
- [Are GCC and Clang parsers really handwritten? - Stack Overflow](https://stackoverflow.com/questions/6319086/are-gcc-and-clang-parsers-really-handwritten)
- [Handwritten Parsers & Lexers in Go - Gopher Academy Blog](https://blog.gopheracademy.com/advent-2014/parsers-lexers/)
- [Go parser source code](https://go.dev/src/go/parser/parser.go) - `binaryExpr` 関数でPrecedence Climbingを確認可能
