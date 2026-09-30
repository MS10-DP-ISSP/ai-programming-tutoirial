module ElasticCollisionsCairoMakieExt

using CairoMakie
using ElasticCollisions: Trajectory, save_gif
import ElasticCollisions

function ElasticCollisions.save_gif(path::AbstractString, traj::Trajectory; fps = 30, size = (800, 500))
    arena = traj.arena
    frames = traj.frames
    n = length(first(frames))
    colors = cgrad(:turbo, max(n, 2); categorical = true)

    frame = Observable(1)
    title = @lift "t = $(round(traj.times[$frame]; digits = 2))"

    fig = Figure(; size)
    ax = Axis(fig[1, 1]; aspect = DataAspect(), title)
    lines!(ax, [0, arena.width, arena.width, 0, 0], [0, 0, arena.height, arena.height, 0]; color = :black, linewidth = 2)
    for i in 1:n
        disc = @lift begin
            b = frames[$frame][i]
            Circle(Point2f(b.position...), Float32(b.radius))
        end
        poly!(ax, disc; color = colors[i], strokecolor = :black, strokewidth = 1)
    end
    margin = 0.02 * max(arena.width, arena.height)
    limits!(ax, -margin, arena.width + margin, -margin, arena.height + margin)

    record(fig, path, eachindex(frames); framerate = fps) do k
        frame[] = k
    end
    return path
end

end # module ElasticCollisionsCairoMakieExt
