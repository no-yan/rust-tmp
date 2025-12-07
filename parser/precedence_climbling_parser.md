# Precedence climbing parser

https://www.engr.mun.ca/~theo/Misc/exp_parsing.htm#climbing

Precedence climbing algorithmとは、〜を特徴とするパーズアルゴリズムである。
このアルゴリズムは他のアルゴリズムと比較して、~という利点がある。


## EBPF and Precedence

Exp(p) は、pより優先される演算子を持たない式を認識する。

```
E --> Exp(0) 
Exp(p) --> P {B Exp(q)} 
P --> U Exp(q) | "(" E ")" | v
B --> "+" | "-"  | "*" |"/" | "^" | "||" | "&&" | "="
U --> "-"
```

``` Exp(p)
var t : Tree
t = P
while 次が演算子で p 以上なら:
    op を読む
    t1 = Exp(q)
    t = mkNode(t, op, t1)
return t
```

qの求め方:

1. bin_op(p)
左: q = p + 1
右結合: q = p

2. unary_op
q = p

https://www.oilshell.org/blog/2016/11/02.html
https://www.oilshell.org/blog/2017/03/31.html
