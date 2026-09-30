module ElasticCollisions

using Random: Xoshiro

export Ball, Arena, World, Trajectory, kinetic_energy, simulate, random_balls, save_gif

include("types.jl")
include("events.jl")
include("simulate.jl")
include("random_balls.jl")

"""
    save_gif(path, trajectory; fps = 30, size = (800, 500)) -> path

Render `trajectory` as an animated GIF. Requires `CairoMakie` to be loaded.
"""
function save_gif end

end # module ElasticCollisions
