using Base.Cartesian: @nexprs

# Function to compute the greatest common divisor (GCD) using the binary GCD (Stein's) algorithm
# This is a variant of Julia's Base.gcd that keeps both operands odd inside the loop.
# Only unsigned integers are accepted, so abs (which overflows for typemin(Int)) is never needed.
function mygcd(a::T, b::T) where {T<:Unsigned}
    a == 0 && return b
    b == 0 && return a
    za = trailing_zeros(a)
    zb = trailing_zeros(b)
    k = min(za, zb) # The common power of two, 2^k, is a factor of the GCD
    # Remove all factors of two so that both a and b are odd
    a >>= za
    b >>= zb
    # Loop until both values are equal
    while a != b
        d = a - b                     # Wraps around, but has the same trailing zeros as |a - b|
        absd = ifelse(a > b, d, -d)   # |a - b|, which is even and nonzero
        b = min(a, b)                 # Keep the smaller value
        # Remove all factors of two from |a - b|.
        # d != 0, so the shift is less than the bit width; masking tells the compiler this.
        a = absd >> (trailing_zeros(d) & (8sizeof(T) - 1))
    end
    return a << k  # Restore the common power of two
end

# One binary GCD step on odd a and b. A finished lane (a == b) stays as it is.
@inline function gcdstep(a::T, b::T) where {T<:Unsigned}
    d = a - b
    absd = ifelse(a > b, d, -d)
    # d == 0 in a finished lane, so mask the shift and keep a as it is.
    next = absd >> (trailing_zeros(d) & (8sizeof(T) - 1))
    return ifelse(a != b, next, a), min(a, b)
end

# Count pairs (a, b) with 1 <= a, b <= N and gcd(a, b) == 1.
# For a fixed a, the GCDs with b, b+1, ..., b+15 are advanced one step at a time in lockstep.
# A single GCD is a chain of dependent instructions; running 16 independent chains
# side by side lets the CPU overlap them. @nexprs 16 expands the body into 16 copies
# with variables x_1, ..., x_16 so that every lane lives in a register.
#
# Every value of N up to typemax(T) is handled. A block of 16 starting at b0 fits when
# b0 <= N - 15, so b0 + 16 <= N + 1 never wraps: b0 is 1 mod 16 while typemax(T) - 15
# is 0 mod 16. The count is an Int, which holds N^2 for N <= typemax(UInt32).
function countcoprime(N::T) where {T<:Unsigned}
    cnt = 0    # Counter for coprime pairs
    lim = N >= T(15) ? N - T(15) : zero(T)  # Last b0 whose block of 16 fits in 1:N
    for a in one(T):N
        za = trailing_zeros(a)
        ao = a >> za  # Odd part of a
        b0 = one(T)
        while b0 <= lim
            @nexprs 16 i -> begin
                b_i = b0 + T(i - 1)
                zb_i = trailing_zeros(b_i)
                k_i = min(za, zb_i)  # The common power of two
                x_i = ao
                y_i = b_i >> zb_i    # Odd part of b
            end
            while true
                live = zero(T)
                @nexprs 16 i -> (live |= x_i ⊻ y_i)
                live == 0 && break  # Every lane has x == y
                @nexprs 16 i -> ((x_i, y_i) = gcdstep(x_i, y_i))
            end
            @nexprs 16 i -> (cnt += (x_i << k_i) == 1)
            b0 += T(16)
        end
        # Remaining b values that do not fill all lanes
        for b in b0:N
            cnt += mygcd(a, b) == 1
        end
    end
    return cnt
end

# Function to approximate pi using probability that two numbers are coprime
function calcPi(N::UInt32)
    cnt = countcoprime(N)
    # Probability that two numbers are coprime (Float64 avoids overflow in N * N)
    prob = cnt / Float64(N)^2
    # Approximate pi using the formula: pi ≈ sqrt(6 / prob)
    return sqrt(6 / prob)
end

# Main function to run the pi approximation
function main()
    N = UInt32(10000)    # Number limit for coprimality checking
    approx_pi = @time calcPi(N) # Approximate pi and time it
    println("N: $(Int(N))")   # Output N (a UInt32 would be printed in hexadecimal)
    println("pi: $(approx_pi)") # Output approximation of pi
end

main() # Call the main function
