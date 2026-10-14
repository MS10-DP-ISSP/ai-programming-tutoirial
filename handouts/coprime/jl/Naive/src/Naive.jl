module Naive

export calcPi

# Function to compute the greatest common divisor (GCD) using the Euclidean algorithm
function mygcd(a, b)
    # Loop until the remainder is zero
    while b != 0
        tmp = b    # Store the value of b temporarily
        b = a % b  # Update b to the remainder of a divided by b
        a = tmp    # Set a to the previous value of b
    end
    return a       # When b is zero, a is the GCD
end

# Function to approximate pi using probability that two numbers are coprime
function calcPi(N)
    cnt = 0    # Counter for coprime pairs
    # Loop through all pairs (a, b) where 1 <= a, b <= N
    for a in 1:N
        for b in 1:N
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

# Entry point for `julia -m Naive`
function (@main)(args)
    N = 10000            # Number limit for coprimality checking
    approx_pi = @time calcPi(N) # Approximate pi and time it
    println("N: $(N)")   # Output N
    println("pi: $(approx_pi)") # Output approximation of pi
end

end # module Naive
