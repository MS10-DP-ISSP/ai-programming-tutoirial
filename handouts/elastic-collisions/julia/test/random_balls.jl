using Test
using ElasticCollisions

@testset "random_balls" begin
    arena = Arena(10, 6)

    @testset "validity" begin
        balls = random_balls(15; arena, radius = 0.4, speed = 3.0, seed = 1)
        @test length(balls) == 15
        @test all(b -> b.radius == 0.4, balls)
        @test all(b -> b.mass == 1.0, balls)
        @test all(b -> hypot(b.velocity...) ≈ 3.0, balls)
        @test World(balls, arena) isa World   # inside the arena, no overlaps
        # directions are not all the same
        @test length(unique(atan(b.velocity[2], b.velocity[1]) for b in balls)) > 1
    end

    @testset "reproducibility" begin
        a = random_balls(8; arena, radius = 0.4, speed = 3.0, seed = 42)
        b = random_balls(8; arena, radius = 0.4, speed = 3.0, seed = 42)
        c = random_balls(8; arena, radius = 0.4, speed = 3.0, seed = 43)
        @test a == b
        @test a != c
        # no seed still works
        @test length(random_balls(3; arena, radius = 0.4, speed = 1.0)) == 3
    end

    @testset "mass keyword" begin
        balls = random_balls(4; arena, radius = 0.4, speed = 1.0, mass = 2.5, seed = 1)
        @test all(b -> b.mass == 2.5, balls)
    end

    @testset "zero balls" begin
        @test random_balls(0; arena, radius = 0.4, speed = 1.0, seed = 1) == Ball[]
    end

    @testset "impossible placements" begin
        # too many balls for the area
        @test_throws ArgumentError random_balls(100; arena = Arena(2, 2), radius = 0.5, speed = 1.0, seed = 1)
        # a single ball that does not fit
        @test_throws ArgumentError random_balls(1; arena = Arena(2, 2), radius = 1.5, speed = 1.0, seed = 1)
    end
end
