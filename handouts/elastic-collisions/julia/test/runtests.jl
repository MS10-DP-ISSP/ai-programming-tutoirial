using Test

@testset "ElasticCollisions" begin
    include("types.jl")
    include("events.jl")
    include("simulate.jl")
    include("random_balls.jl")
    include("long_run.jl")
    include("save_gif.jl")
    include("aqua.jl")
end
