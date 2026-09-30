use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

/// 2D vector (x to the right, y downward).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn dot(self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, k: f64) -> Vec2 {
        Vec2::new(self.x * k, self.y * k)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, o: Vec2) {
        *self = *self + o;
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, o: Vec2) {
        *self = *self - o;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_sub() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, -1.0);
        assert_eq!(a + b, Vec2::new(4.0, 1.0));
        assert_eq!(a - b, Vec2::new(-2.0, 3.0));
    }

    #[test]
    fn scalar_mul_and_neg() {
        let a = Vec2::new(1.0, -2.0);
        assert_eq!(a * 2.0, Vec2::new(2.0, -4.0));
        assert_eq!(-a, Vec2::new(-1.0, 2.0));
    }

    #[test]
    fn dot_and_length() {
        let a = Vec2::new(3.0, 4.0);
        assert_eq!(a.dot(Vec2::new(1.0, 2.0)), 11.0);
        assert_eq!(a.length_squared(), 25.0);
        assert_eq!(a.length(), 5.0);
    }

    #[test]
    fn assign_ops() {
        let mut a = Vec2::new(1.0, 1.0);
        a += Vec2::new(2.0, 3.0);
        assert_eq!(a, Vec2::new(3.0, 4.0));
        a -= Vec2::new(1.0, 1.0);
        assert_eq!(a, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn is_finite_detects_nan_and_inf() {
        assert!(Vec2::new(1.0, 2.0).is_finite());
        assert!(!Vec2::new(f64::NAN, 2.0).is_finite());
        assert!(!Vec2::new(1.0, f64::INFINITY).is_finite());
    }
}
