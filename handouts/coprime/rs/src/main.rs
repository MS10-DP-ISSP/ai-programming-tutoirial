use rs::calc_pi;

// Main function to run the pi approximation
fn main() {
    let n = 10000u32;   // Number limit for coprimality checking
    let start = std::time::Instant::now();
    let pi = calc_pi(n); // Approximate pi
    let duration = start.elapsed();
    println!("calcPi: {:?}", duration);
    println!("N: {}", n);   // Output N
    println!("pi: {}", pi); // Output approximation of pi
}