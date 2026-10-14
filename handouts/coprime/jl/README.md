# 互いに素な確率による円周率の近似 (Julia 版)

1 以上 N 以下の整数 a, b をランダムに選んだとき、a と b が互いに素になる確率は N → ∞ で 6/π² に近づきます。
このディレクトリのプログラムは N × N 通りの組を数え上げ、`sqrt(6 / prob)` で π を近似します。

## 実行方法

```sh
julia naive.jl   # ユークリッドの互除法 (自作 mygcd)
julia stein.jl   # Julia 標準の gcd (Stein のアルゴリズム)
julia opt.jl     # Stein のアルゴリズムを Unsigned 向けに書き換えた mygcd
```

| ファイル | GCD の実装 | 計測例 (N = 10000, Julia 1.13.1) |
| --- | --- | --- |
| `naive.jl` | ユークリッドの互除法 (`a % b` の繰り返し) | 約 1.90 秒 |
| `stein.jl` | `Base.gcd` (Stein のアルゴリズム) | 約 1.22 秒 |
| `opt.jl` | `Base._gcd` の変形版 | 約 1.07 秒 |

時間は `@time` の出力で、マシンや実行ごとに変わります。

## Stein のアルゴリズム (binary GCD) とは

ユークリッドの互除法は剰余 `a % b` を繰り返し計算します。
整数の除算は CPU にとって重い命令で、加減算やシフトの数十倍の時間がかかることがあります。
Stein のアルゴリズムは除算を使わず、次の 4 つの性質だけで GCD を求めます。

1. `gcd(a, 0) = a`
2. a, b がともに偶数なら `gcd(a, b) = 2 * gcd(a/2, b/2)`
3. a が偶数で b が奇数なら `gcd(a, b) = gcd(a/2, b)` (2 は公約数にならない)
4. a, b がともに奇数なら `gcd(a, b) = gcd(|a - b|, min(a, b))`。このとき `|a - b|` は偶数になる

2 で割る操作は右シフト `>>` で済み、「2 で何回割れるか」は `trailing_zeros` (二進表現の末尾の 0 の数) で一度に求まります。
現代の CPU には `trailing_zeros` に対応する命令 (x86 の `tzcnt`、ARM の `rbit` + `clz`) があるので、ループ 1 回はシフト・減算・比較だけで終わります。

計算の流れは次のとおりです。

1. a と b の末尾の 0 の数の小さいほうを k とする。2^k が共通に含まれる 2 のべき
2. a と b から 2 の因子をすべて取り除き、両方を奇数にする
3. 大きいほうから小さいほうを引く。差は偶数なので 2 の因子を取り除いて奇数に戻す
4. 差が 0 になるまで (あるいは 2 数が等しくなるまで) 3 を繰り返す
5. 残った奇数に 2^k を掛け戻す (`<< k`)

## Julia 標準ライブラリの実装

Julia の `gcd` は `base/intfuncs.jl` にあります (以下は 1.14.0-DEV 時点のソースからの抜粋)。
`Int64` や `UInt32` などの固定長整数 (`BitInteger`) では、入口の関数で 0 と `typemin` を処理してから `_gcd` を呼びます。

```julia
function gcd(a::T, b::T) where T<:BitInteger
    a == 0 && return Base.checked_abs(b)
    b == 0 && return Base.checked_abs(a)
    if a isa Signed && a == typemin(T)
        if a == b
            Base.__throw_gcd_overflow(a, b)
        else
            a, b = b, a
        end
    end
    return _gcd(a, b)
end
```

- どちらかが 0 なら、もう一方の絶対値が答えになる (性質 1)
- 符号付き整数の `typemin` (例: `typemin(Int64) = -2^63`) は `abs` を取ると正の値で表せずオーバーフローする。`gcd(typemin, typemin)` は答え 2^63 が表現できないので `OverflowError` を投げる。それ以外は a と b を入れ替えて、`_gcd` の中で `typemin` に `abs` を適用しないようにする

本体の `_gcd` は次のとおりです。

```julia
function absdiff(x::T,y::T) where {T<:Unsigned}
    d = max(x,y) - min(x,y)
    d, d
end
function absdiff(x::T,y::T) where {T<:Signed}
    d = x - y
    abs(d), d
end
# binary GCD (aka Stein's) algorithm
# about 1.7x (2.1x) faster for random Int64s (Int128s)
@assume_effects :terminates_locally function _gcd(ain::T, bin::T) where T<:BitInteger
    zb = trailing_zeros(bin)
    za = trailing_zeros(ain)
    a = abs(ain)
    b = abs(bin >> zb)
    k = min(za, zb)
    while a != 0
        a >>= za
        absd, diff = absdiff(a, b)
        za = trailing_zeros(diff)
        b = min(a, b)
        a = absd
    end
    r = b << k
    return r % T
end
```

各行の役割は次のとおりです。

- `k = min(za, zb)`: 共通の 2 のべき 2^k を記録する (性質 2)
- `b = abs(bin >> zb)`: b はループに入る前に奇数にしておく。以後 b は常に奇数
- ループ先頭の `a >>= za`: a から 2 の因子を取り除いて奇数にする (性質 3)。初回は元の a の末尾の 0 を、2 回目以降は前回の差 `diff` の末尾の 0 を使う
- `absdiff(a, b)`: 奇数どうしの差 `|a - b|` を求める (性質 4)。差は偶数で、次の周回の先頭で 2 の因子が取り除かれる
- `b = min(a, b)`: 小さいほうを残す。a, b ともに奇数なので b は奇数のまま
- `while a != 0`: 差が 0 になったら a = b なので、b が奇数部分の GCD
- `r = b << k`: 2^k を掛け戻す
- `r % T`: 型 T に戻す。符号付きの場合でも結果は非負

`absdiff` が値を 2 つ返すのは、符号付き整数では `diff = x - y` (符号付きの差) から `trailing_zeros` を取れば、`abs` の計算を待たずに済むからです。
2 の補数表現では `-d` と `d` の末尾の 0 の数が等しいので、`trailing_zeros(diff)` と `trailing_zeros(abs(diff))` は同じ値になります。

`@assume_effects :terminates_locally` はコンパイラに「このループは必ず終了する」と伝える注釈です。
`gcd` は `Rational` のコンストラクタで使われるので、定数引数のときにコンパイル時に計算 (定数畳み込み) できるようにしています。
ソースのコメントによると、ユークリッドの互除法に比べて乱数の `Int64` で約 1.7 倍、`Int128` で約 2.1 倍速くなります。

### 実行例: `gcd(48, 18)`

48 = `0b110000`、18 = `0b10010` です。

| 段階 | a | b | za | 備考 |
| --- | --- | --- | --- | --- |
| 初期化 | 48 | 18 >> 1 = 9 | 4 | k = min(4, 1) = 1 |
| 1 周目 | 48 >> 4 = 3 → 差 \|3 − 9\| = 6 | min(3, 9) = 3 | trailing_zeros(6) = 1 | |
| 2 周目 | 6 >> 1 = 3 → 差 \|3 − 3\| = 0 | 3 | trailing_zeros(0) | a = 0 でループ終了 |
| 終了 | | 3 << 1 = **6** | | |

`trailing_zeros(0)` はビット幅 (Int64 なら 64) を返しますが、そのときはループを抜けるのでシフトには使われません。

## `opt.jl` での変更点

`opt.jl` の `mygcd` は `Base._gcd` をもとに、`Unsigned` 専用に書き換えたものです (`../rs/src/lib.rs` の Rust 版も同じ方針です)。

```julia
function mygcd(a::T, b::T) where {T<:Unsigned}
    a == 0 && return b
    b == 0 && return a
    za = trailing_zeros(a)
    zb = trailing_zeros(b)
    k = min(za, zb)
    a >>= za
    b >>= zb
    while a != b
        d = a - b
        absd = max(a, b) - min(a, b)
        b = min(a, b)
        a = absd >> trailing_zeros(d)
    end
    return a << k
end
```

`Base._gcd` との違いは 3 点です。

1. 引数を `Unsigned` に限定した。`abs` も `typemin` の処理も不要になる
2. ループの条件を `a != 0` から `a != b` にした。2 数が等しくなった時点で終わるので、差が 0 になるのを確かめる最後の 1 周が要らない
3. `trailing_zeros` を `a - b` (符号なしなので負になると桁あふれして大きな値になる) から求める。桁あふれした値は 2^n − |a − b| で、末尾の 0 の数は |a − b| と等しい。`Base` の `Unsigned` 版 `absdiff` は `max - min` の結果から `trailing_zeros` を取るので、`max`・`min`・減算が終わるまで待つ必要がある。`a - b` なら減算 1 回で `trailing_zeros` に進めるので、ループ 1 周の依存チェーンが短くなる

`calcPi` では `one(UInt):N` でループし、`mygcd` に `UInt` を渡しています。

## 参考

- Julia のソース: `base/intfuncs.jl` (https://github.com/JuliaLang/julia/blob/master/base/intfuncs.jl)
- J. Stein, "Computational problems associated with Racah algebra", *Journal of Computational Physics* 1 (3), 397–405 (1967)
- D. E. Knuth, *The Art of Computer Programming*, Vol. 2, §4.5.2
