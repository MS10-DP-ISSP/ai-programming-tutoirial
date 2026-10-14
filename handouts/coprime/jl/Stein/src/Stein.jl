module Stein

export calcPi

function calcPi(N)
    cnt = 0    # Counter for coprime pairs
    # Loop through all pairs (a, b) where 1 <= a, b <= N
    for a in 1:N
        for b in 1:N
            # Check if a and b are coprime
            if gcd(a, b) == 1
                cnt += 1  # Increment counter if coprime
            end
        end
    end
    # Probability that two numbers are coprime
    prob = cnt / (N * N)
    # Approximate pi using the formula: pi ≈ sqrt(6 / prob)
    return sqrt(6 / prob)
end

# Entry point for `julia -m Stein`
function (@main)(args)
    N = 10000            # Number limit for coprimality checking
    approx_pi = @time calcPi(N) # Approximate pi and time it
    println("N: $(N)")   # Output N
    println("pi: $(approx_pi)") # Output approximation of pi
end

end # module Stein
