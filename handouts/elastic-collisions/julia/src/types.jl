"""
    Ball(position, velocity, radius, mass)

A rigid disc with the given `position` `(x, y)`, `velocity` `(vx, vy)`,
`radius > 0` and `mass > 0`.
"""
struct Ball
    position::NTuple{2,Float64}
    velocity::NTuple{2,Float64}
    radius::Float64
    mass::Float64

    function Ball(position, velocity, radius, mass)
        radius > 0 || throw(ArgumentError("radius must be positive, got $radius"))
        mass > 0 || throw(ArgumentError("mass must be positive, got $mass"))
        new(
            (Float64(position[1]), Float64(position[2])),
            (Float64(velocity[1]), Float64(velocity[2])),
            Float64(radius),
            Float64(mass),
        )
    end
end

"""
    Arena(width, height)

The rectangle `[0, width] x [0, height]` with reflecting walls.
"""
struct Arena
    width::Float64
    height::Float64

    function Arena(width, height)
        width > 0 || throw(ArgumentError("width must be positive, got $width"))
        height > 0 || throw(ArgumentError("height must be positive, got $height"))
        new(Float64(width), Float64(height))
    end
end

"""
    World(balls, arena)

Balls inside an arena. Throws `ArgumentError` if a ball sticks out of the arena
or two balls overlap.
"""
struct World
    balls::Vector{Ball}
    arena::Arena

    function World(balls, arena::Arena)
        balls = collect(Ball, balls)
        for (i, b) in enumerate(balls)
            inside(b, arena) ||
                throw(ArgumentError("ball $i is not inside the arena"))
        end
        for i in eachindex(balls), j in (i + 1):lastindex(balls)
            overlaps(balls[i], balls[j]) &&
                throw(ArgumentError("balls $i and $j overlap"))
        end
        new(balls, arena)
    end
end

function inside(b::Ball, arena::Arena)
    x, y = b.position
    r = b.radius
    return r <= x <= arena.width - r && r <= y <= arena.height - r
end

function overlaps(a::Ball, b::Ball)
    dx = b.position[1] - a.position[1]
    dy = b.position[2] - a.position[2]
    return hypot(dx, dy) < a.radius + b.radius
end

"""
    kinetic_energy(ball_or_balls_or_world)

Total kinetic energy ``\\sum m |v|^2 / 2``.
"""
kinetic_energy(b::Ball) = b.mass * (b.velocity[1]^2 + b.velocity[2]^2) / 2
kinetic_energy(balls::AbstractVector{Ball}) = sum(kinetic_energy, balls; init = 0.0)
kinetic_energy(w::World) = kinetic_energy(w.balls)
