#include <stdio.h>
#include <math.h>
#include <time.h>

// Function to compute the greatest common divisor (GCD) using the Euclidean algorithm
int mygcd(int a, int b) {
    // Loop until the remainder is zero
    while (b != 0) {
        int tmp = b;    // Store the value of b temporarily
        b = a % b;      // Update b to the remainder of a divided by b
        a = tmp;        // Set a to the previous value of b
    }
    return a;           // When b is zero, a is the GCD
}

// Function to approximate pi using probability that two numbers are coprime
double calcPi(int N) {
    int cnt = 0;    // Counter for coprime pairs
    // Loop through all pairs (a, b) where 1 <= a, b <= N
    for (int a = 1; a <= N; a++) {
        for (int b = 1; b <= N; b++) {
            // Check if a and b are coprime
            if (mygcd(a, b) == 1) {
                cnt += 1;  // Increment counter if coprime
            }
        }
    }
    // Probability that two numbers are coprime
    double prob = (double)cnt / (N * N);
    // Approximate pi using the formula: pi ≈ sqrt(6 / prob)
    return sqrt(6.0 / prob);
}

// Main function to run the pi approximation
int main() {
    int N = 10000;            // Number limit for coprimality checking
    clock_t start = clock();
    double pi = calcPi(N);    // Approximate pi
    clock_t end = clock();
    double cpu_time_used = ((double)(end - start)) / CLOCKS_PER_SEC;

    printf("calcPi: %f seconds\n", cpu_time_used);
    printf("N: %d\n", N);     // Output N
    printf("pi: %f\n", pi);   // Output approximation of pi

    return 0;
}
