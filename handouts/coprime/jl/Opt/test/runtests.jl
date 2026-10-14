using Test
using Opt
using Opt: PositiveUInt32, countcoprime, mygcd

# Number of coprime pairs (a, b) with 1 <= a, b <= n, counted with Base.gcd
refcount(n) = count(gcd(a, b) == 1 for a in 1:n, b in 1:n)

@testset "Opt" begin
    @testset "mygcd matches Base.gcd" begin
        for T in (UInt8, UInt32, UInt64), a in 0:100, b in 0:100
            @test mygcd(T(a), T(b)) == gcd(a, b)
        end
    end

    @testset "countcoprime" begin
        # Every N of UInt8, including typemax(UInt8), checks the block loop at the type's limit
        for n in 0:255
            @test countcoprime(UInt8(n)) == refcount(n)
        end
        for n in (100, 257, 1000)
            @test countcoprime(UInt32(n)) == refcount(n)
        end
    end

    @testset "PositiveUInt32" begin
        @test PositiveUInt32(1).value == 1
        @test PositiveUInt32(typemax(UInt32)).value == typemax(UInt32)
        @test_throws ArgumentError PositiveUInt32(0)
        @test_throws ArgumentError PositiveUInt32(-1)
        @test_throws ArgumentError PositiveUInt32(Int64(typemax(UInt32)) + 1)
    end

    @testset "calcPi" begin
        @test calcPi(PositiveUInt32(1)) ≈ sqrt(6)  # Only (1, 1)
        @test calcPi(PositiveUInt32(2)) ≈ sqrt(8)  # (1, 1), (1, 2), (2, 1) -> prob = 3/4
        @test abs(calcPi(PositiveUInt32(1000)) - π) < 1e-2
    end

    @testset "calcPi rejects plain integers" begin
        @test_throws MethodError calcPi(10000)
        @test_throws MethodError calcPi(UInt32(10000))
    end
end
