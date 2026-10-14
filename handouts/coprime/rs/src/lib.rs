// Port of Julia's gcd functions from base/intfuncs.jl
// Greatest common divisor implementations using Euclidean and Binary GCD algorithms

use std::fmt::Display;
use std::ops::Rem;

/// Fixed-width integer types (Julia's `BitInteger`).
pub trait BitInteger: Copy + PartialEq + Display {
    const ZERO: Self;

    /// Absolute value; panics like Julia's `checked_abs` on `typemin` of signed types.
    fn checked_abs(self) -> Self;

    /// Binary GCD (aka Stein's) algorithm. Both arguments must be nonzero.
    fn binary_gcd(a: Self, b: Self) -> Self;
}

#[cold]
#[inline(never)]
fn throw_gcd_overflow<T: Display>(a: T, b: T) -> ! {
    panic!("gcd({}, {}) overflows", a, b)
}

/// Binary GCD on nonzero unsigned values.
///
/// Unlike Julia's `_gcd`, both operands are kept odd inside the loop and the loop
/// stops when they become equal. The wrapping difference has the same number of
/// trailing zeros as the absolute difference, so `trailing_zeros` does not have to
/// wait for `abs_diff`.
macro_rules! stein {
    ($a:expr, $b:expr) => {{
        let (a, b) = ($a, $b);
        let za = a.trailing_zeros();
        let zb = b.trailing_zeros();
        let k = za.min(zb);
        let mut a = a >> za;
        let mut b = b >> zb;
        while a != b {
            let d = a.wrapping_sub(b);
            // |a - b| selected from d: one `subs` + `cneg`. `abs_diff` becomes
            // `umax` + `umin` + `sub` when the CPU has `umax`/`umin` (e.g. -C target-cpu=native).
            let absd = if a > b { d } else { d.wrapping_neg() };
            b = a.min(b);
            a = absd >> d.trailing_zeros();
        }
        a << k
    }};
}

macro_rules! impl_unsigned {
    ($($t:ty),*) => {$(
        impl BitInteger for $t {
            const ZERO: Self = 0;
            fn checked_abs(self) -> Self {
                self
            }
            fn binary_gcd(a: Self, b: Self) -> Self {
                stein!(a, b)
            }
        }
    )*};
}

macro_rules! impl_signed {
    ($($t:ty),*) => {$(
        impl BitInteger for $t {
            const ZERO: Self = 0;
            fn checked_abs(self) -> Self {
                match <$t>::checked_abs(self) {
                    Some(x) => x,
                    None => panic!("abs({}) overflows", self),
                }
            }
            fn binary_gcd(a: Self, b: Self) -> Self {
                // Work on |a|, |b| as unsigned so that typemin is handled without overflow.
                // The result fits unless a == b == typemin (gcd = 2^(bits-1)).
                if a == <$t>::MIN && a == b {
                    throw_gcd_overflow(a, b);
                }
                stein!(a.unsigned_abs(), b.unsigned_abs()) as $t
            }
        }
    )*};
}

impl_unsigned!(u8, u16, u32, u64, u128);
impl_signed!(i8, i16, i32, i64, i128);

/// Greatest common (positive) divisor (or zero if all arguments are zero).
///
/// This is the general implementation for all integer types using the Euclidean algorithm.
pub fn gcd<T>(mut a: T, mut b: T) -> T
where
    T: BitInteger + Rem<Output = T>,
{
    while b != T::ZERO {
        let t = b;
        b = a % b;
        a = t;
    }
    a.checked_abs()
}

/// Greatest common divisor for fixed-width integer types using Binary GCD (Stein's) algorithm.
///
/// This is about 1.7x (2.1x) faster for random Int64s (Int128s) compared to Euclidean algorithm.
pub fn gcd_bitinteger<T: BitInteger>(a: T, b: T) -> T {
    if a == T::ZERO {
        return b.checked_abs();
    }
    if b == T::ZERO {
        return a.checked_abs();
    }
    T::binary_gcd(a, b)
}

macro_rules! typed_gcd {
    ($($t:ty => $gcd:ident, $bit:ident);* $(;)?) => {$(
        #[doc = concat!("GCD for ", stringify!($t))]
        pub fn $gcd(a: $t, b: $t) -> $t {
            $bit(a, b)
        }

        #[doc = concat!("Binary GCD for ", stringify!($t), " (Stein's algorithm matching Julia's implementation)")]
        pub fn $bit(a: $t, b: $t) -> $t {
            gcd_bitinteger(a, b)
        }
    )*};
}

typed_gcd! {
    u8 => gcd_u8, gcd_bitinteger_u8;
    u16 => gcd_u16, gcd_bitinteger_u16;
    u32 => gcd_u32, gcd_bitinteger_u32;
    u64 => gcd_u64, gcd_bitinteger_u64;
    u128 => gcd_u128, gcd_bitinteger_u128;
    i8 => gcd_i8, gcd_bitinteger_i8;
    i16 => gcd_i16, gcd_bitinteger_i16;
    i32 => gcd_i32, gcd_bitinteger_i32;
    i64 => gcd_i64, gcd_bitinteger_i64;
    i128 => gcd_i128, gcd_bitinteger_i128;
}

// Function to approximate pi using probability that two numbers are coprime
pub fn calc_pi(n: u64) -> f64 {
    let mut cnt = 0u64; // Counter for coprime pairs
    // Loop through all pairs (a, b) where 1 <= a, b <= N
    for a in 1..=n {
        for b in 1..=n {
            // Check if a and b are coprime using the fast binary GCD algorithm
            if gcd_u64(a, b) == 1 {
                cnt += 1;  // Increment counter if coprime
            }
        }
    }
    // Probability that two numbers are coprime
    let prob = cnt as f64 / (n as f64 * n as f64);
    // Approximate pi using the formula: pi ≈ sqrt(6 / prob)
    (6.0 / prob).sqrt()
}

