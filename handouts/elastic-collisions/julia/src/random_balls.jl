"""
    random_balls(n; arena, radius, speed, mass = 1.0, seed = nothing) -> Vector{Ball}

`n` non-overlapping balls placed by rejection sampling, with random directions and
the given `speed`. `seed` makes the result reproducible. Throws `ArgumentError` if
the balls cannot be placed.
"""
function random_balls(
    n::Integer;
    arena::Arena,
    radius::Real,
    speed::Real,
    mass::Real = 1.0,
    seed = nothing,
    max_attempts::Integer = 10_000,
)
    rng = seed === nothing ? Xoshiro() : Xoshiro(seed)
    xmax, ymax = arena.width - radius, arena.height - radius
    (radius < xmax && radius < ymax) ||
        throw(ArgumentError("a ball of radius $radius does not fit in the arena"))
    balls = Ball[]
    for k in 1:n
        placed = false
        for _ in 1:max_attempts
            x = radius + (xmax - radius) * rand(rng)
            y = radius + (ymax - radius) * rand(rng)
            all(b -> hypot(b.position[1] - x, b.position[2] - y) > b.radius + radius, balls) ||
                continue
            θ = 2π * rand(rng)
            push!(balls, Ball((x, y), (speed * cos(θ), speed * sin(θ)), radius, mass))
            placed = true
            break
        end
        placed || throw(ArgumentError("could not place ball $k of $n in $max_attempts attempts"))
    end
    return balls
end
