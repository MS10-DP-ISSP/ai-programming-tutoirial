use crate::ball::Ball;
use crate::error::Error;
use crate::vec2::Vec2;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const MAX_PLACEMENT_ATTEMPTS: usize = 1000;

/// Box width (x extent).
pub const WIDTH: f64 = 800.0;
/// Box height (y extent, downward).
pub const HEIGHT: f64 = 600.0;

/// A set of balls inside the 800x600 box.
#[derive(Debug, Clone)]
pub struct World {
    balls: Vec<Ball>,
}

impl World {
    /// Validates the balls: positive finite radius, finite state, inside the box, no overlap.
    pub fn new(balls: Vec<Ball>) -> Result<World, Error> {
        for (i, b) in balls.iter().enumerate() {
            if !(b.radius.is_finite() && b.radius > 0.0) {
                return Err(Error::InvalidInput(format!(
                    "ball {i}: radius must be positive and finite"
                )));
            }
            if !b.pos.is_finite() || !b.vel.is_finite() {
                return Err(Error::InvalidInput(format!(
                    "ball {i}: position and velocity must be finite"
                )));
            }
            if b.pos.x - b.radius < 0.0
                || b.pos.x + b.radius > WIDTH
                || b.pos.y - b.radius < 0.0
                || b.pos.y + b.radius > HEIGHT
            {
                return Err(Error::InvalidInput(format!("ball {i}: outside the box")));
            }
        }
        for i in 0..balls.len() {
            for j in (i + 1)..balls.len() {
                let d = balls[j].pos - balls[i].pos;
                let rr = balls[i].radius + balls[j].radius;
                if d.length_squared() < rr * rr {
                    return Err(Error::InvalidInput(format!("balls {i} and {j} overlap")));
                }
            }
        }
        Ok(World { balls })
    }

    /// Reproducible, non-overlapping random world (radius 10..30, speed 20..200, uniform direction).
    pub fn random(n: usize, seed: u64) -> Result<World, Error> {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut balls: Vec<Ball> = Vec::with_capacity(n);
        for _ in 0..n {
            let ball = (0..MAX_PLACEMENT_ATTEMPTS)
                .map(|_| {
                    let radius = rng.gen_range(10.0..=30.0);
                    let pos = Vec2::new(
                        rng.gen_range(radius..=WIDTH - radius),
                        rng.gen_range(radius..=HEIGHT - radius),
                    );
                    let speed = rng.gen_range(20.0..=200.0);
                    let angle = rng.gen_range(0.0..std::f64::consts::TAU);
                    Ball::new(pos, Vec2::new(angle.cos(), angle.sin()) * speed, radius)
                })
                .find(|c| {
                    balls.iter().all(|b| {
                        let rr = b.radius + c.radius;
                        (b.pos - c.pos).length_squared() >= rr * rr
                    })
                })
                .ok_or(Error::PlacementFailed)?;
            balls.push(ball);
        }
        World::new(balls)
    }

    pub fn balls(&self) -> &[Ball] {
        &self.balls
    }

    pub fn kinetic_energy(&self) -> f64 {
        self.balls.iter().map(Ball::kinetic_energy).sum()
    }

    pub fn total_momentum(&self) -> Vec2 {
        self.balls
            .iter()
            .fold(Vec2::default(), |acc, b| acc + b.vel * b.mass())
    }

    /// Advances by `dt`: move, then ball-ball collisions, then walls.
    pub fn step(&mut self, dt: f64) {
        for b in &mut self.balls {
            b.pos += b.vel * dt;
        }
        self.resolve_collisions();
        self.resolve_walls();
    }

    fn resolve_collisions(&mut self) {
        let n = self.balls.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let d = self.balls[j].pos - self.balls[i].pos;
                let rr = self.balls[i].radius + self.balls[j].radius;
                let dist_sq = d.length_squared();
                if dist_sq >= rr * rr {
                    continue;
                }
                let dist = dist_sq.sqrt();
                let normal = if dist > 0.0 {
                    d * (1.0 / dist)
                } else {
                    Vec2::new(1.0, 0.0)
                };
                let half = (rr - dist) * 0.5;
                self.balls[i].pos -= normal * half;
                self.balls[j].pos += normal * half;

                let approach = (self.balls[i].vel - self.balls[j].vel).dot(normal);
                if approach <= 0.0 {
                    continue;
                }
                let (ma, mb) = (self.balls[i].mass(), self.balls[j].mass());
                let j_mag = 2.0 * approach / (1.0 / ma + 1.0 / mb);
                self.balls[i].vel -= normal * (j_mag / ma);
                self.balls[j].vel += normal * (j_mag / mb);
            }
        }
    }

    fn resolve_walls(&mut self) {
        for b in &mut self.balls {
            let r = b.radius;
            if b.pos.x < r {
                b.pos.x = r;
                b.vel.x = b.vel.x.abs();
            } else if b.pos.x > WIDTH - r {
                b.pos.x = WIDTH - r;
                b.vel.x = -b.vel.x.abs();
            }
            if b.pos.y < r {
                b.pos.y = r;
                b.vel.y = b.vel.y.abs();
            } else if b.pos.y > HEIGHT - r {
                b.pos.y = HEIGHT - r;
                b.vel.y = -b.vel.y.abs();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ball(x: f64, y: f64, vx: f64, vy: f64, r: f64) -> Ball {
        Ball::new(Vec2::new(x, y), Vec2::new(vx, vy), r)
    }

    fn invalid(balls: Vec<Ball>) -> bool {
        matches!(World::new(balls), Err(Error::InvalidInput(_)))
    }

    #[test]
    fn box_is_800_by_600() {
        assert_eq!(WIDTH, 800.0);
        assert_eq!(HEIGHT, 600.0);
    }

    #[test]
    fn accepts_valid_balls() {
        let w = World::new(vec![
            ball(100.0, 100.0, 1.0, 2.0, 10.0),
            ball(300.0, 300.0, 0.0, 0.0, 20.0),
        ])
        .unwrap();
        assert_eq!(w.balls().len(), 2);
    }

    #[test]
    fn accepts_touching_wall_and_touching_balls() {
        assert!(World::new(vec![
            ball(10.0, 10.0, 0.0, 0.0, 10.0),
            ball(790.0, 590.0, 0.0, 0.0, 10.0)
        ])
        .is_ok());
        assert!(World::new(vec![
            ball(100.0, 100.0, 0.0, 0.0, 10.0),
            ball(120.0, 100.0, 0.0, 0.0, 10.0)
        ])
        .is_ok());
    }

    #[test]
    fn rejects_bad_radius() {
        assert!(invalid(vec![ball(100.0, 100.0, 0.0, 0.0, 0.0)]));
        assert!(invalid(vec![ball(100.0, 100.0, 0.0, 0.0, -1.0)]));
        assert!(invalid(vec![ball(100.0, 100.0, 0.0, 0.0, f64::NAN)]));
        assert!(invalid(vec![ball(100.0, 100.0, 0.0, 0.0, f64::INFINITY)]));
    }

    #[test]
    fn rejects_non_finite_position_or_velocity() {
        assert!(invalid(vec![ball(f64::NAN, 100.0, 0.0, 0.0, 10.0)]));
        assert!(invalid(vec![ball(100.0, f64::INFINITY, 0.0, 0.0, 10.0)]));
        assert!(invalid(vec![ball(100.0, 100.0, f64::NAN, 0.0, 10.0)]));
        assert!(invalid(vec![ball(
            100.0,
            100.0,
            0.0,
            f64::NEG_INFINITY,
            10.0
        )]));
    }

    #[test]
    fn rejects_balls_outside_box() {
        assert!(invalid(vec![ball(9.0, 100.0, 0.0, 0.0, 10.0)]));
        assert!(invalid(vec![ball(791.0, 100.0, 0.0, 0.0, 10.0)]));
        assert!(invalid(vec![ball(100.0, 9.0, 0.0, 0.0, 10.0)]));
        assert!(invalid(vec![ball(100.0, 591.0, 0.0, 0.0, 10.0)]));
    }

    #[test]
    fn rejects_overlapping_balls() {
        assert!(invalid(vec![
            ball(100.0, 100.0, 0.0, 0.0, 10.0),
            ball(119.0, 100.0, 0.0, 0.0, 10.0)
        ]));
    }

    #[test]
    fn free_motion_moves_by_velocity_times_dt() {
        let mut w = World::new(vec![ball(100.0, 100.0, 10.0, -20.0, 10.0)]).unwrap();
        w.step(0.5);
        let b = w.balls()[0];
        assert_eq!(b.pos, Vec2::new(105.0, 90.0));
        assert_eq!(b.vel, Vec2::new(10.0, -20.0));
    }

    #[test]
    fn reflects_off_each_wall() {
        // (start, vel, expected pos after 0.1s, expected vel)
        let cases = [
            (
                ball(15.0, 300.0, -100.0, 5.0, 10.0),
                Vec2::new(10.0, 300.5),
                Vec2::new(100.0, 5.0),
            ),
            (
                ball(785.0, 300.0, 100.0, 5.0, 10.0),
                Vec2::new(790.0, 300.5),
                Vec2::new(-100.0, 5.0),
            ),
            (
                ball(400.0, 15.0, 5.0, -100.0, 10.0),
                Vec2::new(400.5, 10.0),
                Vec2::new(5.0, 100.0),
            ),
            (
                ball(400.0, 585.0, 5.0, 100.0, 10.0),
                Vec2::new(400.5, 590.0),
                Vec2::new(5.0, -100.0),
            ),
        ];
        for (start, pos, vel) in cases {
            let mut w = World::new(vec![start]).unwrap();
            w.step(0.1);
            let b = w.balls()[0];
            assert!(
                (b.pos - pos).length() < 1e-9,
                "pos {:?} vs {:?}",
                b.pos,
                pos
            );
            assert!(
                (b.vel - vel).length() < 1e-9,
                "vel {:?} vs {:?}",
                b.vel,
                vel
            );
        }
    }

    #[test]
    fn wall_does_not_flip_velocity_that_already_points_inward() {
        // Placed outside the left wall but already moving inward.
        let mut w = World {
            balls: vec![ball(5.0, 300.0, 50.0, 0.0, 10.0)],
        };
        w.step(0.0);
        let b = w.balls()[0];
        assert_eq!(b.pos.x, 10.0);
        assert_eq!(b.vel.x, 50.0);
    }

    #[test]
    fn stays_inside_box_even_with_huge_step() {
        let mut w = World::new(vec![ball(400.0, 300.0, 1e4, -1e4, 10.0)]).unwrap();
        w.step(1.0);
        let b = w.balls()[0];
        assert!(b.pos.x - 10.0 >= 0.0 && b.pos.x + 10.0 <= WIDTH);
        assert!(b.pos.y - 10.0 >= 0.0 && b.pos.y + 10.0 <= HEIGHT);
    }

    fn overlapping(balls: Vec<Ball>) -> World {
        World { balls }
    }

    fn close(a: Vec2, b: Vec2) -> bool {
        (a - b).length() < 1e-9
    }

    #[test]
    fn overlapping_balls_are_separated_half_each() {
        let mut w = overlapping(vec![
            ball(100.0, 100.0, 0.0, 0.0, 10.0),
            ball(110.0, 100.0, 0.0, 0.0, 10.0),
        ]);
        w.step(0.0);
        assert!(close(w.balls()[0].pos, Vec2::new(95.0, 100.0)));
        assert!(close(w.balls()[1].pos, Vec2::new(115.0, 100.0)));
    }

    #[test]
    fn separation_splits_overlap_evenly_for_unequal_radii() {
        let mut w = overlapping(vec![
            ball(100.0, 100.0, 0.0, 0.0, 10.0),
            ball(100.0, 120.0, 0.0, 0.0, 20.0),
        ]);
        w.step(0.0);
        // overlap = 30 - 20 = 10, normal = +y
        assert!(close(w.balls()[0].pos, Vec2::new(100.0, 95.0)));
        assert!(close(w.balls()[1].pos, Vec2::new(100.0, 125.0)));
    }

    #[test]
    fn coincident_centers_use_x_axis_as_normal() {
        let mut w = overlapping(vec![
            ball(100.0, 100.0, 0.0, 0.0, 10.0),
            ball(100.0, 100.0, 0.0, 0.0, 10.0),
        ]);
        w.step(0.0);
        assert!(close(w.balls()[0].pos, Vec2::new(90.0, 100.0)));
        assert!(close(w.balls()[1].pos, Vec2::new(110.0, 100.0)));
    }

    #[test]
    fn touching_balls_are_left_alone() {
        let mut w = World::new(vec![
            ball(100.0, 100.0, 0.0, 0.0, 10.0),
            ball(120.0, 100.0, 0.0, 0.0, 10.0),
        ])
        .unwrap();
        w.step(0.0);
        assert_eq!(w.balls()[0].pos, Vec2::new(100.0, 100.0));
        assert_eq!(w.balls()[1].pos, Vec2::new(120.0, 100.0));
    }

    #[test]
    fn head_on_equal_balls_swap_velocities() {
        let mut w = World::new(vec![
            ball(100.0, 100.0, 100.0, 0.0, 10.0),
            ball(120.0, 100.0, 0.0, 0.0, 10.0),
        ])
        .unwrap();
        w.step(0.01);
        assert!(close(w.balls()[0].vel, Vec2::new(0.0, 0.0)));
        assert!(close(w.balls()[1].vel, Vec2::new(100.0, 0.0)));
    }

    #[test]
    fn head_on_unequal_masses_follow_1d_elastic_formula() {
        let mut w = overlapping(vec![
            ball(100.0, 100.0, 50.0, 0.0, 10.0),
            ball(129.0, 100.0, 0.0, 0.0, 20.0),
        ]);
        w.step(0.0);
        let (ma, mb) = (w.balls()[0].mass(), w.balls()[1].mass());
        assert!(close(
            w.balls()[0].vel,
            Vec2::new((ma - mb) / (ma + mb) * 50.0, 0.0)
        ));
        assert!(close(
            w.balls()[1].vel,
            Vec2::new(2.0 * ma / (ma + mb) * 50.0, 0.0)
        ));
    }

    #[test]
    fn tangential_velocity_is_preserved() {
        let mut w = overlapping(vec![
            ball(100.0, 100.0, 100.0, 30.0, 10.0),
            ball(119.0, 100.0, 0.0, -5.0, 10.0),
        ]);
        w.step(0.0);
        assert!(close(w.balls()[0].vel, Vec2::new(0.0, 30.0)));
        assert!(close(w.balls()[1].vel, Vec2::new(100.0, -5.0)));
    }

    #[test]
    fn separating_overlapping_balls_keep_their_velocities() {
        let mut w = overlapping(vec![
            ball(100.0, 100.0, -10.0, 3.0, 10.0),
            ball(110.0, 100.0, 10.0, -3.0, 10.0),
        ]);
        w.step(0.0);
        assert_eq!(w.balls()[0].vel, Vec2::new(-10.0, 3.0));
        assert_eq!(w.balls()[1].vel, Vec2::new(10.0, -3.0));
        assert!(close(w.balls()[0].pos, Vec2::new(95.0, 100.0)));
    }

    #[test]
    fn overlapping_balls_with_equal_velocity_are_separated_without_impulse() {
        let mut w = overlapping(vec![
            ball(100.0, 100.0, 5.0, 5.0, 10.0),
            ball(110.0, 100.0, 5.0, 5.0, 10.0),
        ]);
        w.step(0.0);
        assert_eq!(w.balls()[0].vel, Vec2::new(5.0, 5.0));
        assert_eq!(w.balls()[1].vel, Vec2::new(5.0, 5.0));
    }

    #[test]
    fn kinetic_energy_and_momentum_sum_over_balls() {
        let w = World::new(vec![
            ball(100.0, 100.0, 3.0, 4.0, 1.0),
            ball(300.0, 300.0, -3.0, 0.0, 2.0),
        ])
        .unwrap();
        let (m1, m2) = (w.balls()[0].mass(), w.balls()[1].mass());
        assert!((w.kinetic_energy() - (0.5 * m1 * 25.0 + 0.5 * m2 * 9.0)).abs() < 1e-9);
        assert!(close(
            w.total_momentum(),
            Vec2::new(3.0 * m1 - 3.0 * m2, 4.0 * m1)
        ));
    }

    #[test]
    fn collision_conserves_momentum_and_energy() {
        let mut w = overlapping(vec![
            ball(100.0, 100.0, 80.0, 20.0, 12.0),
            ball(115.0, 108.0, -30.0, 10.0, 25.0),
        ]);
        let (p0, e0) = (w.total_momentum(), w.kinetic_energy());
        w.step(0.0);
        assert!(close(w.total_momentum(), p0));
        assert!((w.kinetic_energy() - e0).abs() / e0 < 1e-12);
    }

    #[test]
    fn random_is_reproducible_for_same_seed() {
        let a = World::random(20, 7).unwrap();
        let b = World::random(20, 7).unwrap();
        assert_eq!(a.balls(), b.balls());
        let c = World::random(20, 8).unwrap();
        assert_ne!(a.balls(), c.balls());
    }

    #[test]
    fn random_respects_ranges_and_validity() {
        let w = World::random(30, 42).unwrap();
        assert_eq!(w.balls().len(), 30);
        for b in w.balls() {
            assert!((10.0..=30.0).contains(&b.radius));
            let speed = b.vel.length();
            assert!(
                (20.0 - 1e-9..=200.0 + 1e-9).contains(&speed),
                "speed {speed}"
            );
        }
        // Re-validating must succeed: inside the box, no overlap.
        assert!(World::new(w.balls().to_vec()).is_ok());
    }

    #[test]
    fn random_directions_are_spread_over_all_quadrants() {
        let w = World::random(60, 1).unwrap();
        let mut seen = [false; 4];
        for b in w.balls() {
            seen[(b.vel.x >= 0.0) as usize * 2 + (b.vel.y >= 0.0) as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn random_zero_balls_is_empty() {
        assert!(World::random(0, 1).unwrap().balls().is_empty());
    }

    #[test]
    fn random_fails_when_box_cannot_hold_all_balls() {
        assert!(matches!(
            World::random(5000, 1),
            Err(Error::PlacementFailed)
        ));
    }
}
