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

# Function to approximate pi using probability that two numbers are coprime
function calcPi(N)
    cnt = 0    # Counter for coprime pairs
    # Loop through all pairs (a, b) where 1 <= a, b <= N
    for a in one(UInt):N
        for b in one(UInt):N
            # Check if a and b are coprime
            if mygcd(a, b) == 1
                cnt += 1  # Increment counter if coprime
            end
        end
    end
    # Probability that two numbers are coprime
    prob = cnt / (N * N)
    # Approximate pi using the formula: pi ≈ sqrt(6 / prob)
    return sqrt(6 / prob)
end

# Main function to run the pi approximation
function main()
    N = 10000            # Number limit for coprimality checking
    approx_pi = @time calcPi(N) # Approximate pi and time it
    println("N: $(N)")   # Output N
    println("pi: $(approx_pi)") # Output approximation of pi
end

main() # Call the main function
