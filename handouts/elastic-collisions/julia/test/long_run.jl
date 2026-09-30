using Test
using ElasticCollisions

@testset "long run: 20 balls of different masses, T = 200" begin
    arena = Arena(10, 6)
    balls = random_balls(20; arena, radius = 0.3, speed = 2.0, seed = 3)
    # give every ball its own mass
    balls = [Ball(b.position, b.velocity, b.radius, 0.5 + 0.25 * i) for (i, b) in enumerate(balls)]
    world = World(balls, arena)
    traj = simulate(world, 200.0; frame_dt = 0.5)

    e0 = kinetic_energy(world)
    @test length(traj.frames) == 401
    for (t, frame) in zip(traj.times, traj.frames)
        @test kinetic_energy(frame) ≈ e0 rtol = 1e-9
        for b in frame
            x, y = b.position
            @test b.radius - 1e-9 <= x <= arena.width - b.radius + 1e-9
            @test b.radius - 1e-9 <= y <= arena.height - b.radius + 1e-9
        end
        for i in eachindex(frame), j in (i + 1):lastindex(frame)
            d = hypot(frame[i].position[1] - frame[j].position[1], frame[i].position[2] - frame[j].position[2])
            @test d >= frame[i].radius + frame[j].radius - 1e-9
        end
    end
end
