// approx_pi_gcd.js

// Function to compute the greatest common divisor (GCD) using the Euclidean algorithm
function mygcd(a, b) {
    // Loop until the remainder is zero
    while (b !== 0) {
        let tmp = b;    // Store the value of b temporarily
        b = a % b;      // Update b to the remainder of a divided by b
        a = tmp;        // Set a to the previous value of b
    }
    return a;           // When b is zero, a is the GCD
}

// Function to approximate pi using probability that two numbers are coprime
function calcPi(N) {
    let cnt = 0;    // Counter for coprime pairs
    // Loop through all pairs (a, b) where 1 <= a, b <= N
    for (let a = 1; a <= N; a++) {
        for (let b = 1; b <= N; b++) {
            // Check if a and b are coprime
            if (mygcd(a, b) === 1) {
                cnt += 1;  // Increment counter if coprime
            }
        }
    }
    // Probability that two numbers are coprime
    let prob = cnt / (N * N);
    // Approximate pi using the formula: pi ≈ sqrt(6 / prob)
    return Math.sqrt(6 / prob);
}

// Main function to run the pi approximation
function main() {
    let N = 10000;            // Number limit for coprimality checking
    let pi = calcPi(N);       // Approximate pi
    console.log(`N: ${N}`);   // Output N
    console.log(`pi: ${pi}`); // Output approximation of pi
}

main(); // Call the main function

