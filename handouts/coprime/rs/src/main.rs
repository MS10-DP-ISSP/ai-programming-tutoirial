use rs::calc_pi;
use std::num::NonZeroU32;

// Main function to run the pi approximation
fn main() {
    // Number limit for coprimality checking; NonZeroU32::new returns None for 0
    let n = NonZeroU32::new(10000).expect("N must be positive");
    let start = std::time::Instant::now();
    let pi = calc_pi(n); // Approximate pi
    let duration = start.elapsed();
    println!("calcPi: {:?}", duration);
    println!("N: {}", n);   // Output N
    println!("pi: {}", pi); // Output approximation of pi
}