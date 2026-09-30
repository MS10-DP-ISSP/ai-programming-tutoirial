using ElasticCollisions
using CairoMakie

arena = Arena(10, 6)
balls = random_balls(15; arena, radius = 0.4, speed = 3.0, seed = 7)
world = World(balls, arena)
traj = simulate(world, 10.0; frame_dt = 1 / 30)
path = save_gif(joinpath(@__DIR__, "collisions.gif"), traj; fps = 30)
println("wrote ", path)
