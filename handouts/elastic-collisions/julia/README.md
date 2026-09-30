# ElasticCollisions.jl

2D 空間で複数のボールの弾性衝突をシミュレーションし、GIF アニメーションとして可視化する Julia パッケージです。

- ボールは位置・速度・半径・質量を持つ剛体の円盤（半径と質量はボールごとに異なってよい）。重力と摩擦はなし。
- 容器は反射壁を持つ矩形 `[0, width] x [0, height]`（`Arena`）。
- 時間発展はイベント駆動です。壁・ボール対の衝突時刻を厳密に計算し、最も早いイベントまで進めて衝突を解きます。固定時間刻みは使いません。
- コアパッケージの依存は標準ライブラリの `Random` だけです。GIF 出力は CairoMakie を読み込んだときだけ有効になるパッケージ拡張です。

## クイックスタート

GIF を作るには `app/` のスクリプトを実行します。

```sh
cd app
julia --project main.jl
```

`app/collisions.gif` が生成されます（アリーナ 10 x 6、半径 0.4 の 15 球、速さ 3、seed 7、30 fps で 10 秒）。
初回は依存パッケージの取得とプリコンパイルに時間がかかります。

## サンプルコード

```julia
using ElasticCollisions
using CairoMakie          # save_gif を使うときだけ必要

arena = Arena(10, 6)

# ボールを明示的に指定する: Ball((x, y), (vx, vy), radius, mass)
balls = [
    Ball((2, 3), (1, 0), 1.0, 1.0),
    Ball((8, 3), (-1, 0), 1.0, 2.0),
]

# あるいは、重ならないボールをランダムに生成する
balls = random_balls(15; arena, radius = 0.4, speed = 3.0, seed = 7)

world = World(balls, arena)             # はみ出し・重なりがあれば ArgumentError
traj  = simulate(world, 10.0; frame_dt = 1/30)
kinetic_energy(traj.frames[end])        # 運動エネルギー（保存される）
save_gif("collisions.gif", traj; fps = 30, size = (800, 500))
```

## API

| 名前 | 説明 |
| --- | --- |
| `Ball(pos, vel, radius, mass)` | ボール。`radius > 0`、`mass > 0`。 |
| `Arena(width, height)` | 反射壁を持つ矩形の容器。 |
| `World(balls, arena)` | ボールとアリーナの組。不正な初期配置は `ArgumentError`。 |
| `random_balls(n; arena, radius, speed, mass = 1.0, seed = nothing)` | 棄却サンプリングで重ならない `n` 個のボールを生成する。配置できなければ `ArgumentError`。 |
| `simulate(world, T; frame_dt)` | `T` まで進め、`frame_dt` ごとのフレームを持つ `Trajectory`（`arena`, `times`, `frames`）を返す。 |
| `kinetic_energy(x)` | ボール、ボールのベクトル、`World` の運動エネルギー。 |
| `save_gif(path, trajectory; fps = 30, size = (800, 500))` | GIF に書き出す（CairoMakie が必要）。 |

フレームは `World` ではなく単なる `Vector{Ball}` です。接触時の丸め誤差程度の重なりで検証エラーが出ないようにするためです。

内部ヘルパー（`time_to_wall`, `time_to_pair`, `collide_pair`, `collide_wall`, `advance`）は export していません。
必要なら `import ElasticCollisions: time_to_pair` のように明示的に import してください。

## テストの実行

```sh
julia --project -e 'using Pkg; Pkg.test()'
```

テストは workspace 方式の `test/Project.toml`（パッケージ本体、Test、CairoMakie、Aqua）で動きます。
手早く回すときは次のようにします。

```sh
julia --project=test test/runtests.jl
```

`weakdeps` や `extensions` を編集したら、`julia --project -e 'using Pkg; Pkg.resolve()'` を実行してください。
実行しないと拡張が読み込まれません。
