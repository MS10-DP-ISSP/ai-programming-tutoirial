use rs::*;
use std::num::NonZeroU32;

#[test]
fn test_gcd_u64() {
    assert_eq!(gcd_u64(6, 9), 3);
    assert_eq!(gcd_u64(6, 0), 6);
    assert_eq!(gcd_u64(0, 0), 0);
    assert_eq!(gcd_u64(48, 18), 6);
    assert_eq!(gcd_u64(17, 13), 1);
}

#[test]
fn test_gcd_i64() {
    assert_eq!(gcd_i64(6, 9), 3);
    assert_eq!(gcd_i64(6, -9), 3);
    assert_eq!(gcd_i64(-6, 9), 3);
    assert_eq!(gcd_i64(-6, -9), 3);
    assert_eq!(gcd_i64(6, 0), 6);
    assert_eq!(gcd_i64(0, 0), 0);
}

#[test]
fn test_gcd_u32() {
    assert_eq!(gcd_u32(48, 18), 6);
    assert_eq!(gcd_u32(100, 25), 25);
}

#[test]
fn test_gcd_i32() {
    assert_eq!(gcd_i32(48, -18), 6);
    assert_eq!(gcd_i32(-100, 25), 25);
}

/// Reference implementation (Euclidean algorithm on i128) used to cross-check.
fn ref_gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

#[test]
fn test_gcd_u8_exhaustive() {
    for a in 0..=u8::MAX {
        for b in 0..=u8::MAX {
            assert_eq!(gcd_u8(a, b) as i128, ref_gcd(a as i128, b as i128), "gcd_u8({a}, {b})");
        }
    }
}

#[test]
fn test_gcd_i8_exhaustive() {
    for a in i8::MIN..=i8::MAX {
        for b in i8::MIN..=i8::MAX {
            let expected = ref_gcd(a as i128, b as i128);
            // gcd(MIN, 0), gcd(0, MIN), gcd(MIN, MIN) = 128 is not representable in i8
            if expected > i8::MAX as i128 {
                continue;
            }
            assert_eq!(gcd_i8(a, b) as i128, expected, "gcd_i8({a}, {b})");
        }
    }
}

#[test]
fn test_gcd_u16_against_reference() {
    for a in (0..=u16::MAX).step_by(97) {
        for b in (0..=u16::MAX).step_by(89) {
            assert_eq!(gcd_u16(a, b) as i128, ref_gcd(a as i128, b as i128), "gcd_u16({a}, {b})");
        }
    }
}

#[test]
fn test_gcd_i16_against_reference() {
    for a in (-1000i16..=1000).step_by(7) {
        for b in (-1000i16..=1000).step_by(11) {
            assert_eq!(gcd_i16(a, b) as i128, ref_gcd(a as i128, b as i128), "gcd_i16({a}, {b})");
        }
    }
}

#[test]
fn test_gcd_small_range_all_types() {
    for a in 0..=200u32 {
        for b in 0..=200u32 {
            let expected = ref_gcd(a as i128, b as i128);
            assert_eq!(gcd_u32(a, b) as i128, expected);
            assert_eq!(gcd_u64(a as u64, b as u64) as i128, expected);
            assert_eq!(gcd_u128(a as u128, b as u128) as i128, expected);
            let (sa, sb) = (a as i64, b as i64);
            for (x, y) in [(sa, sb), (-sa, sb), (sa, -sb), (-sa, -sb)] {
                assert_eq!(gcd_i32(x as i32, y as i32) as i128, expected);
                assert_eq!(gcd_i64(x, y) as i128, expected);
                assert_eq!(gcd_i128(x as i128, y as i128), expected);
            }
        }
    }
}

#[test]
fn test_gcd_is_commutative() {
    let vals = [0u64, 1, 2, 3, 12, 18, 1 << 40, 3 << 20, 1_000_000_007, u64::MAX];
    for &a in &vals {
        for &b in &vals {
            assert_eq!(gcd_u64(a, b), gcd_u64(b, a));
        }
    }
}

#[test]
fn test_gcd_powers_of_two() {
    assert_eq!(gcd_u64(1 << 10, 1 << 20), 1 << 10);
    assert_eq!(gcd_u64(3 << 5, 5 << 7), 1 << 5);
    assert_eq!(gcd_u128(1 << 100, 3 << 64), 1 << 64);
    assert_eq!(gcd_i64(-(1 << 30), 6 << 10), 1 << 11);
}

#[test]
fn test_gcd_extreme_values() {
    assert_eq!(gcd_u8(u8::MAX, u8::MAX), u8::MAX);
    assert_eq!(gcd_u16(u16::MAX, 0), u16::MAX);
    assert_eq!(gcd_u32(u32::MAX, u32::MAX - 1), 1);
    assert_eq!(gcd_u64(u64::MAX, u64::MAX), u64::MAX);
    assert_eq!(gcd_u64(u64::MAX, 3), 3); // 2^64 - 1 is divisible by 3
    assert_eq!(gcd_u128(u128::MAX, 0), u128::MAX);
    assert_eq!(gcd_u128(u128::MAX, 5), 5); // 2^128 - 1 is divisible by 5

    assert_eq!(gcd_i32(i32::MAX, i32::MAX), i32::MAX);
    assert_eq!(gcd_i64(i64::MIN, 2), 2);
    assert_eq!(gcd_i64(i64::MIN, i64::MAX), 1);
    assert_eq!(gcd_i64(i64::MIN, 1 << 62), 1 << 62);
    assert_eq!(gcd_i128(i128::MIN, 6), 2);
}

#[test]
fn test_gcd_large_primes() {
    let p = 1_000_000_007u64;
    let q = 998_244_353u64;
    assert_eq!(gcd_u64(p, q), 1);
    assert_eq!(gcd_u64(p * 6, p * 4), p * 2);
    assert_eq!(gcd_i64(-((p * 6) as i64), (q * 6) as i64), 6);
    assert_eq!(gcd_u128(p as u128 * q as u128, q as u128 * 7), q as u128);
}

#[test]
#[should_panic(expected = "overflows")]
fn test_gcd_i8_min_min_panics() {
    gcd_i8(i8::MIN, i8::MIN);
}

#[test]
#[should_panic(expected = "overflows")]
fn test_gcd_i64_min_min_panics() {
    gcd_i64(i64::MIN, i64::MIN);
}

#[test]
fn test_gcd_wrappers_match_bitinteger() {
    assert_eq!(gcd_u8(12, 18), gcd_bitinteger_u8(12, 18));
    assert_eq!(gcd_u16(12, 18), gcd_bitinteger_u16(12, 18));
    assert_eq!(gcd_u32(12, 18), gcd_bitinteger_u32(12, 18));
    assert_eq!(gcd_u64(12, 18), gcd_bitinteger_u64(12, 18));
    assert_eq!(gcd_u128(12, 18), gcd_bitinteger_u128(12, 18));
    assert_eq!(gcd_i8(-12, 18), gcd_bitinteger_i8(-12, 18));
    assert_eq!(gcd_i16(-12, 18), gcd_bitinteger_i16(-12, 18));
    assert_eq!(gcd_i32(-12, 18), gcd_bitinteger_i32(-12, 18));
    assert_eq!(gcd_i64(-12, 18), gcd_bitinteger_i64(-12, 18));
    assert_eq!(gcd_i128(-12, 18), gcd_bitinteger_i128(-12, 18));
}

#[test]
fn test_generic_gcd_euclid() {
    for a in 0..=100u64 {
        for b in 0..=100u64 {
            assert_eq!(gcd(a, b) as i128, ref_gcd(a as i128, b as i128));
        }
    }
    assert_eq!(gcd(48u32, 18u32), 6);
}

#[test]
fn test_generic_gcd_bitinteger_unsigned() {
    for a in 0..=100u32 {
        for b in 0..=100u32 {
            assert_eq!(gcd_bitinteger(a, b) as i128, ref_gcd(a as i128, b as i128));
        }
    }
}

#[test]
fn test_generic_gcd_euclid_signed_is_positive() {
    assert_eq!(gcd(6i32, -9), 3);
    assert_eq!(gcd(-6i32, 9), 3);
    assert_eq!(gcd(-6i64, -9), 3);
    assert_eq!(gcd(0i64, -5), 5);
}

#[test]
fn test_generic_gcd_bitinteger_signed() {
    for a in -100..=100i32 {
        for b in -100..=100i32 {
            assert_eq!(gcd_bitinteger(a, b) as i128, ref_gcd(a as i128, b as i128), "gcd_bitinteger({a}, {b})");
        }
    }
    assert_eq!(gcd_bitinteger(i64::MIN, 6), 2);
}

#[test]
fn test_generic_gcd_bitinteger_large() {
    assert_eq!(gcd_bitinteger(u64::MAX, 1), 1);
    assert_eq!(gcd_bitinteger(u128::MAX, 5), 5);
    assert_eq!(gcd_bitinteger(1u64 << 63, 3 << 10), 1 << 10);
}

#[test]
#[should_panic(expected = "overflows")]
fn test_gcd_i64_min_zero_panics() {
    gcd_i64(i64::MIN, 0);
}

#[test]
#[should_panic(expected = "overflows")]
fn test_gcd_i8_zero_min_panics() {
    gcd_i8(0, i8::MIN);
}

#[test]
#[should_panic(expected = "overflows")]
fn test_generic_gcd_euclid_min_zero_panics() {
    gcd(i32::MIN, 0);
}

#[test]
fn test_calc_pi_small() {
    // Coprime pairs in [1,2]^2: (1,1), (1,2), (2,1) -> prob = 3/4
    assert!((calc_pi(NonZeroU32::new(2).unwrap()) - 8.0f64.sqrt()).abs() < 1e-12);
    // Only pair (1,1) -> prob = 1
    assert!((calc_pi(NonZeroU32::new(1).unwrap()) - 6.0f64.sqrt()).abs() < 1e-12);
}

#[test]
fn test_calc_pi_approximates_pi() {
    let pi = calc_pi(NonZeroU32::new(1000).unwrap());
    assert!((pi - std::f64::consts::PI).abs() < 1e-2, "calc_pi(1000) = {pi}");
}

#[test]
fn test_count_coprime_pairs_matches_reference() {
    // Covers n smaller than, equal to, and not a multiple of the lane count.
    for n in (0..=40).chain([100, 257]) {
        let mut expected = 0u64;
        for a in 1..=n {
            for b in 1..=n {
                expected += (gcd(a, b) == 1) as u64;
            }
        }
        assert_eq!(count_coprime_pairs(n), expected, "n = {n}");
    }
}

#[test]
fn test_zero_is_rejected_before_calc_pi() {
    // calc_pi takes NonZeroU32, so N = 0 is turned into an error when the argument is built.
    assert!(NonZeroU32::new(0).is_none());
    assert!(NonZeroU32::try_from(0u32).is_err());
}
