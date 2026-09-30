using Test
using ElasticCollisions

@testset "simulate" begin
    arena = Arena(10, 6)

    @testset "frame times" begin
        w = World([Ball((5, 3), (1, 0), 1.0, 1.0)], arena)
        traj = simulate(w, 1.0; frame_dt = 0.25)
        @test traj isa Trajectory
        @test traj.arena == arena
        @test traj.times ≈ [0.0, 0.25, 0.5, 0.75, 1.0]
        @test length(traj.frames) == length(traj.times)
        @test traj.frames[1] == w.balls
        @test traj.frames[end][1].position[1] ≈ 6.0
        # T that is not a multiple of frame_dt: last frame is at or before T
        traj = simulate(w, 1.0; frame_dt = 0.3)
        @test traj.times ≈ [0.0, 0.3, 0.6, 0.9]
        # T = 0 yields just the initial frame
        @test simulate(w, 0.0; frame_dt = 0.1).times == [0.0]
    end

    @testset "a ball bouncing between the walls" begin
        w = World([Ball((5, 3), (2, 0), 1.0, 1.0)], arena)
        traj = simulate(w, 7.0; frame_dt = 1.0)
        xs = [f[1].position[1] for f in traj.frames]
        @test xs ≈ [5, 7, 9, 7, 5, 3, 1, 3]
        @test all(f[1].position[2] ≈ 3.0 for f in traj.frames)
        @test traj.frames[end][1].velocity == (2.0, 0.0)
        @test traj.frames[3][1].velocity[1] ≈ 2.0 || traj.frames[3][1].velocity[1] ≈ -2.0
    end

    @testset "head-on collision of equal masses" begin
        w = World([Ball((2, 3), (1, 0), 1.0, 1.0), Ball((8, 3), (-1, 0), 1.0, 1.0)], arena)
        traj = simulate(w, 3.0; frame_dt = 1.0)
        # they touch at t = 2 (centres at 4 and 6) and exchange velocities
        b1, b2 = traj.frames[end]
        @test b1.position[1] ≈ 3.0
        @test b2.position[1] ≈ 7.0
        @test b1.velocity[1] ≈ -1.0
        @test b2.velocity[1] ≈ 1.0
    end

    @testset "corner hit is handled one wall at a time" begin
        w = World([Ball((2, 2), (-1, -1), 1.0, 1.0)], arena)
        traj = simulate(w, 2.0; frame_dt = 1.0)   # hits both walls at t = 1
        b = traj.frames[end][1]
        @test b.velocity == (1.0, 1.0)
        @test b.position[1] ≈ 2.0
        @test b.position[2] ≈ 2.0
    end

    @testset "argument errors" begin
        w = World([Ball((5, 3), (1, 0), 1.0, 1.0)], arena)
        @test_throws ArgumentError simulate(w, -1.0; frame_dt = 0.1)
        @test_throws ArgumentError simulate(w, 1.0; frame_dt = 0.0)
        @test_throws ArgumentError simulate(w, 1.0; frame_dt = -0.1)
    end
end
