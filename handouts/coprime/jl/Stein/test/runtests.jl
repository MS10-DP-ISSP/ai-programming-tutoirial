using Test
using Stein

@testset "Stein" begin
    @testset "calcPi" begin
        @test calcPi(1) ≈ sqrt(6)  # Only (1, 1)
        @test calcPi(2) ≈ sqrt(8)  # (1, 1), (1, 2), (2, 1) -> prob = 3/4
        @test abs(calcPi(1000) - π) < 1e-2
    end
end
