using Test
using ElasticCollisions
import ElasticCollisions: time_to_wall, time_to_pair, collide_pair, collide_wall, advance

momentum(bs) = (sum(b.mass * b.velocity[1] for b in bs), sum(b.mass * b.velocity[2] for b in bs))

@testset "events" begin
    arena = Arena(10, 6)

    @testset "time_to_wall" begin
        t, wall = time_to_wall(Ball((5, 3), (2, 0), 1.0, 1.0), arena)
        @test t ≈ 2.0
        @test wall == :right
        t, wall = time_to_wall(Ball((5, 3), (-1, 0), 1.0, 1.0), arena)
        @test t ≈ 4.0
        @test wall == :left
        t, wall = time_to_wall(Ball((5, 3), (0, 1), 0.5, 1.0), arena)
        @test t ≈ 2.5
        @test wall == :top
        t, wall = time_to_wall(Ball((5, 3), (0, -2), 0.5, 1.0), arena)
        @test t ≈ 1.25
        @test wall == :bottom
        # the earlier of the two walls wins
        t, wall = time_to_wall(Ball((5, 3), (1, 3), 1.0, 1.0), arena)
        @test t ≈ 2 / 3
        @test wall == :top
        # a resting ball never hits a wall
        t, _ = time_to_wall(Ball((5, 3), (0, 0), 1.0, 1.0), arena)
        @test t == Inf
        # a ball touching the wall and moving away hits the opposite wall
        t, wall = time_to_wall(Ball((1, 3), (1, 0), 1.0, 1.0), arena)
        @test t ≈ 8.0
        @test wall == :right
        # a ball touching the wall and moving into it collides immediately
        t, wall = time_to_wall(Ball((1, 3), (-1, 0), 1.0, 1.0), arena)
        @test t == 0.0
        @test wall == :left
    end

    @testset "time_to_pair" begin
        a = Ball((2, 3), (1, 0), 1.0, 1.0)
        # approaching head-on: gap is 6 - 2 = 4, closing speed 2
        @test time_to_pair(a, Ball((8, 3), (-1, 0), 1.0, 1.0)) ≈ 2.0
        # separating
        @test time_to_pair(a, Ball((8, 3), (2, 0), 1.0, 1.0)) == Inf
        # passing by without contact
        @test time_to_pair(a, Ball((8, 5.5), (-1, 0), 1.0, 1.0)) == Inf
        # touching and approaching
        @test time_to_pair(a, Ball((4, 3), (-1, 0), 1.0, 1.0)) ≈ 0.0 atol = 1e-12
        # touching and separating
        @test time_to_pair(a, Ball((4, 3), (1.5, 0), 1.0, 1.0)) == Inf
        # different radii: sum of radii 1.5, gap 6 - 1.5 = 4.5, closing speed 3
        @test time_to_pair(Ball((2, 3), (1, 0), 1.0, 1.0), Ball((8, 3), (-2, 0), 0.5, 1.0)) ≈ 1.5
        # oblique glancing: exact root
        b1 = Ball((0, 0), (1, 0), 1.0, 1.0)
        b2 = Ball((4, 1), (0, 0), 1.0, 1.0)
        t = time_to_pair(b1, b2)
        @test hypot(b2.position[1] - (b1.position[1] + t), b2.position[2] - b1.position[2]) ≈ 2.0
        @test t ≈ 4 - sqrt(3)
    end

    @testset "advance" begin
        b = advance(Ball((1, 2), (3, -1), 0.5, 2.0), 0.5)
        @test b.position == (2.5, 1.5)
        @test b.velocity == (3.0, -1.0)
        @test b.radius == 0.5 && b.mass == 2.0
    end

    @testset "collide_wall" begin
        b = Ball((1, 3), (-2, 1), 1.0, 1.0)
        @test collide_wall(b, :left).velocity == (2.0, 1.0)
        @test collide_wall(b, :right).velocity == (2.0, 1.0)
        @test collide_wall(b, :bottom).velocity == (-2.0, -1.0)
        @test collide_wall(b, :top).velocity == (-2.0, -1.0)
        @test collide_wall(b, :left).position == b.position
        @test_throws ArgumentError collide_wall(b, :nowhere)
    end

    @testset "collide_pair" begin
        # equal masses, head-on: velocities are exchanged
        a = Ball((2, 3), (2, 0), 1.0, 1.0)
        b = Ball((4, 3), (-1, 0), 1.0, 1.0)
        a2, b2 = collide_pair(a, b)
        @test a2.velocity[1] ≈ -1.0
        @test a2.velocity[2] ≈ 0.0 atol = 1e-12
        @test b2.velocity[1] ≈ 2.0
        @test b2.velocity[2] ≈ 0.0 atol = 1e-12
        # tangential component is unchanged
        a = Ball((2, 3), (2, 1), 1.0, 1.0)
        b = Ball((4, 3), (-1, -3), 1.0, 1.0)
        a2, b2 = collide_pair(a, b)
        @test a2.velocity[2] ≈ 1.0
        @test b2.velocity[2] ≈ -3.0
        # unequal masses, oblique: momentum and energy are conserved
        a = Ball((2, 3), (2, 0.5), 1.0, 3.0)
        b = Ball((3.2, 4.5), (-1, -0.7), 0.75, 0.5)
        a2, b2 = collide_pair(a, b)
        @test all(momentum([a2, b2]) .≈ momentum([a, b]))
        @test kinetic_energy([a2, b2]) ≈ kinetic_energy([a, b])
        # heavy ball barely changes, light ball bounces back
        heavy = Ball((2, 3), (1, 0), 1.0, 1e9)
        light = Ball((4, 3), (0, 0), 1.0, 1.0)
        h2, l2 = collide_pair(heavy, light)
        @test h2.velocity[1] ≈ 1.0 atol = 1e-8
        @test l2.velocity[1] ≈ 2.0 atol = 1e-8
        # positions, radii and masses are untouched
        @test a2.position == a.position && a2.radius == a.radius && a2.mass == a.mass
    end
end
