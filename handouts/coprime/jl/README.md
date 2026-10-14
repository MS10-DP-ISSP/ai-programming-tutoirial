# 互いに素な確率による円周率の近似 (Julia 版)

1 以上 N 以下の整数 a, b をランダムに選んだとき、a と b が互いに素になる確率は N → ∞ で 6/π² に近づきます。
このディレクトリのプログラムは N × N 通りの組を数え上げ、`sqrt(6 / prob)` で π を近似します。

## 実行方法

```sh
julia naive.jl   # ユークリッドの互除法 (自作 mygcd)
julia stein.jl   # Julia 標準の gcd (Stein のアルゴリズム)
julia opt.jl     # Stein のアルゴリズムを Unsigned 向けに書き換えた mygcd
```
