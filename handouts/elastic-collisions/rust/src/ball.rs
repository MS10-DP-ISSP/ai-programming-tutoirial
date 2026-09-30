use crate::vec2::Vec2;

/// A disc with mass proportional to its area (mass = pi r^2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ball {
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f64,
}

impl Ball {
    pub fn new(pos: Vec2, vel: Vec2, radius: f64) -> Self {
        Self { pos, vel, radius }
    }

    pub fn mass(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    pub fn kinetic_energy(&self) -> f64 {
        0.5 * self.mass() * self.vel.length_squared()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec2::Vec2;
    use std::f64::consts::PI;

    #[test]
    fn mass_is_pi_r_squared() {
        let b = Ball::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0), 2.0);
        assert!((b.mass() - 4.0 * PI).abs() < 1e-12);
    }

    #[test]
    fn kinetic_energy_is_half_m_v_squared() {
        let b = Ball::new(Vec2::new(0.0, 0.0), Vec2::new(3.0, 4.0), 1.0);
        assert!((b.kinetic_energy() - 0.5 * PI * 25.0).abs() < 1e-12);
    }

    #[test]
    fn fields_are_stored() {
        let b = Ball::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0), 5.0);
        assert_eq!(b.pos, Vec2::new(1.0, 2.0));
        assert_eq!(b.vel, Vec2::new(3.0, 4.0));
        assert_eq!(b.radius, 5.0);
    }
}
