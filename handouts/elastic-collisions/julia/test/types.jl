using Test
using ElasticCollisions

@testset "types" begin
    @testset "Ball" begin
        b = Ball((1, 2), (3, 4), 0.5, 2)
        @test b.position == (1.0, 2.0)
        @test b.velocity == (3.0, 4.0)
        @test b.radius === 0.5
        @test b.mass === 2.0
        @test_throws ArgumentError Ball((0, 0), (0, 0), 0.0, 1.0)
        @test_throws ArgumentError Ball((0, 0), (0, 0), -1.0, 1.0)
        @test_throws ArgumentError Ball((0, 0), (0, 0), 1.0, 0.0)
        @test_throws ArgumentError Ball((0, 0), (0, 0), 1.0, -2.0)
    end

    @testset "Arena" begin
        a = Arena(10, 6)
        @test a.width === 10.0
        @test a.height === 6.0
        @test_throws ArgumentError Arena(0, 1)
        @test_throws ArgumentError Arena(1, -1)
    end

    @testset "World" begin
        arena = Arena(10, 6)
        b1 = Ball((2, 3), (1, 0), 1.0, 1.0)
        b2 = Ball((5, 3), (-1, 0), 1.0, 1.0)
        w = World([b1, b2], arena)
        @test w.balls == [b1, b2]
        @test w.arena == arena
        # touching is allowed (distance == r1 + r2)
        @test World([b1, Ball((4, 3), (0, 0), 1.0, 1.0)], arena) isa World
        # touching the wall is allowed
        @test World([Ball((1, 1), (0, 0), 1.0, 1.0)], arena) isa World
        # overlap
        @test_throws ArgumentError World([b1, Ball((3.5, 3), (0, 0), 1.0, 1.0)], arena)
        # outside the arena
        @test_throws ArgumentError World([Ball((0.5, 3), (0, 0), 1.0, 1.0)], arena)
        @test_throws ArgumentError World([Ball((9.5, 3), (0, 0), 1.0, 1.0)], arena)
        @test_throws ArgumentError World([Ball((5, 0.5), (0, 0), 1.0, 1.0)], arena)
        @test_throws ArgumentError World([Ball((5, 5.5), (0, 0), 1.0, 1.0)], arena)
    end

    @testset "kinetic_energy" begin
        b1 = Ball((0, 0), (3, 4), 1.0, 2.0)   # 0.5*2*25 = 25
        b2 = Ball((0, 0), (0, 2), 1.0, 1.0)   # 0.5*1*4 = 2
        @test kinetic_energy(b1) ≈ 25.0
        @test kinetic_energy([b1, b2]) ≈ 27.0
        @test kinetic_energy(World([Ball((2, 2), (3, 4), 1.0, 2.0)], Arena(10, 6))) ≈ 25.0
    end
end
