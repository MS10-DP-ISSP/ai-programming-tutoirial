"""
    time_to_wall(ball, arena) -> (t, wall)

Time until `ball` touches a wall, and the wall's name (`:left`, `:right`,
`:bottom`, `:top`). `(Inf, :none)` if the ball never reaches a wall.
"""
function time_to_wall(b::Ball, arena::Arena)
    x, y = b.position
    vx, vy = b.velocity
    r = b.radius
    t, wall = Inf, :none
    if vx > 0
        t, wall = max((arena.width - r - x) / vx, 0.0), :right
    elseif vx < 0
        t, wall = max((x - r) / -vx, 0.0), :left
    end
    if vy > 0
        ty = max((arena.height - r - y) / vy, 0.0)
        ty < t && ((t, wall) = (ty, :top))
    elseif vy < 0
        ty = max((y - r) / -vy, 0.0)
        ty < t && ((t, wall) = (ty, :bottom))
    end
    return t, wall
end

"""
    time_to_pair(a, b) -> t

Time until balls `a` and `b` touch: the smaller root of
`|dx + dv t| = r_a + r_b`, only if they are approaching (`dx ⋅ dv < 0`).
`Inf` otherwise.
"""
function time_to_pair(a::Ball, b::Ball)
    dx = b.position[1] - a.position[1]
    dy = b.position[2] - a.position[2]
    dvx = b.velocity[1] - a.velocity[1]
    dvy = b.velocity[2] - a.velocity[2]
    p = dx * dvx + dy * dvy
    p < 0 || return Inf
    v2 = dvx^2 + dvy^2
    c = dx^2 + dy^2 - (a.radius + b.radius)^2
    disc = p^2 - v2 * c
    disc < 0 && return Inf
    # smaller root of v2 t^2 + 2 p t + c = 0, in the cancellation-free form
    return max(c / (-p + sqrt(disc)), 0.0)
end

"""
    advance(ball, dt) -> Ball

`ball` moved freely for time `dt`.
"""
function advance(b::Ball, dt)
    pos = (b.position[1] + b.velocity[1] * dt, b.position[2] + b.velocity[2] * dt)
    return Ball(pos, b.velocity, b.radius, b.mass)
end

"""
    collide_wall(ball, wall) -> Ball

Reflect `ball` off `wall` (`:left`, `:right`, `:bottom` or `:top`).
"""
function collide_wall(b::Ball, wall::Symbol)
    vx, vy = b.velocity
    if wall === :left || wall === :right
        vx = -vx
    elseif wall === :bottom || wall === :top
        vy = -vy
    else
        throw(ArgumentError("unknown wall: $wall"))
    end
    return Ball(b.position, (vx, vy), b.radius, b.mass)
end

"""
    collide_pair(a, b) -> (a′, b′)

Elastic collision: velocity components along the line of centres are
exchanged according to the masses, tangential components are unchanged.
"""
function collide_pair(a::Ball, b::Ball)
    dx = b.position[1] - a.position[1]
    dy = b.position[2] - a.position[2]
    d = hypot(dx, dy)
    nx, ny = dx / d, dy / d
    van = a.velocity[1] * nx + a.velocity[2] * ny
    vbn = b.velocity[1] * nx + b.velocity[2] * ny
    m = a.mass + b.mass
    van′ = ((a.mass - b.mass) * van + 2b.mass * vbn) / m
    vbn′ = ((b.mass - a.mass) * vbn + 2a.mass * van) / m
    a′ = Ball(a.position, (a.velocity[1] + (van′ - van) * nx, a.velocity[2] + (van′ - van) * ny), a.radius, a.mass)
    b′ = Ball(b.position, (b.velocity[1] + (vbn′ - vbn) * nx, b.velocity[2] + (vbn′ - vbn) * ny), b.radius, b.mass)
    return a′, b′
end
