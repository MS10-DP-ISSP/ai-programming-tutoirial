"""
    Trajectory

Result of [`simulate`](@ref): the `arena`, the frame `times`, and the balls at
each time in `frames`.
"""
struct Trajectory
    arena::Arena
    times::Vector{Float64}
    frames::Vector{Vector{Ball}}
end

# Earliest event among all walls and pairs: (dt, i, j, wall).
# `j == 0` means ball `i` hits `wall`; otherwise balls `i` and `j` collide.
function next_event(balls::Vector{Ball}, arena::Arena)
    best = (Inf, 0, 0, :none)
    for i in eachindex(balls)
        t, wall = time_to_wall(balls[i], arena)
        t < best[1] && (best = (t, i, 0, wall))
        for j in (i + 1):lastindex(balls)
            t = time_to_pair(balls[i], balls[j])
            t < best[1] && (best = (t, i, j, :none))
        end
    end
    return best
end

"""
    simulate(world, T; frame_dt) -> Trajectory

Evolve `world` for time `T`, event by event (no fixed time step), and record the
balls every `frame_dt`: at times `0, frame_dt, 2frame_dt, …` up to `T`.
"""
function simulate(world::World, T::Real; frame_dt::Real)
    T >= 0 || throw(ArgumentError("T must be non-negative, got $T"))
    frame_dt > 0 || throw(ArgumentError("frame_dt must be positive, got $frame_dt"))
    arena = world.arena
    nframes = floor(Int, T / frame_dt + 1e-9)
    times = [k * Float64(frame_dt) for k in 0:nframes]

    balls = copy(world.balls)
    frames = [copy(balls)]
    t = 0.0
    for target in @view times[2:end]
        while true
            dt, i, j, wall = next_event(balls, arena)
            t + dt <= target || break
            balls = [advance(b, dt) for b in balls]
            t += dt
            if j == 0
                balls[i] = collide_wall(balls[i], wall)
            else
                balls[i], balls[j] = collide_pair(balls[i], balls[j])
            end
        end
        dt = target - t
        balls = [advance(b, dt) for b in balls]
        t = target
        push!(frames, copy(balls))
    end
    return Trajectory(arena, times, frames)
end
