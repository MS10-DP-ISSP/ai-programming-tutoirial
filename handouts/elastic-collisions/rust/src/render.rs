use crate::world::{World, HEIGHT, WIDTH};

/// Index 0 is the black background; balls use indices 1..=16.
pub const PALETTE: [[u8; 3]; 17] = [
    [0, 0, 0],
    [230, 25, 75],
    [60, 180, 75],
    [255, 225, 25],
    [0, 130, 200],
    [245, 130, 48],
    [145, 30, 180],
    [70, 240, 240],
    [240, 50, 230],
    [210, 245, 60],
    [250, 190, 212],
    [0, 128, 128],
    [220, 190, 255],
    [170, 110, 40],
    [255, 250, 200],
    [128, 0, 0],
    [170, 255, 195],
];

/// A palette-indexed image, row-major, origin at the top left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<u8>,
}

impl Frame {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.pixels[y * self.width as usize + x]
    }
}

/// Draws every ball as a filled disc (no anti-aliasing); later balls paint over earlier ones.
pub fn render(world: &World, scale: f64) -> Frame {
    let width = (WIDTH * scale).round() as usize;
    let height = (HEIGHT * scale).round() as usize;
    let mut pixels = vec![0u8; width * height];
    for (i, ball) in world.balls().iter().enumerate() {
        let color = (1 + i % 16) as u8;
        let (cx, cy, r) = (ball.pos.x * scale, ball.pos.y * scale, ball.radius * scale);
        let x0 = (cx - r).floor().max(0.0) as usize;
        let x1 = ((cx + r).ceil().max(0.0) as usize).min(width);
        let y0 = (cy - r).floor().max(0.0) as usize;
        let y1 = ((cy + r).ceil().max(0.0) as usize).min(height);
        for y in y0..y1 {
            for x in x0..x1 {
                let (dx, dy) = (x as f64 + 0.5 - cx, y as f64 + 0.5 - cy);
                if dx * dx + dy * dy <= r * r {
                    pixels[y * width + x] = color;
                }
            }
        }
    }
    Frame {
        width: width as u16,
        height: height as u16,
        pixels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ball::Ball;
    use crate::vec2::Vec2;
    use crate::world::World;

    fn world(balls: Vec<Ball>) -> World {
        World::new(balls).unwrap()
    }

    fn ball(x: f64, y: f64, r: f64) -> Ball {
        Ball::new(Vec2::new(x, y), Vec2::new(0.0, 0.0), r)
    }

    #[test]
    fn palette_is_black_plus_16_distinct_non_black_colors() {
        assert_eq!(PALETTE.len(), 17);
        assert_eq!(PALETTE[0], [0, 0, 0]);
        for (i, color) in PALETTE.iter().enumerate().skip(1) {
            assert_ne!(*color, [0, 0, 0], "color {i} is black");
            for (j, other) in PALETTE.iter().enumerate().skip(i + 1) {
                assert_ne!(color, other, "colors {i} and {j} are equal");
            }
        }
    }

    #[test]
    fn frame_size_follows_scale() {
        let f = render(&world(vec![]), 0.5);
        assert_eq!((f.width, f.height), (400, 300));
        assert_eq!(f.pixels.len(), 400 * 300);
    }

    #[test]
    fn empty_world_is_all_background() {
        let f = render(&world(vec![]), 0.5);
        assert!(f.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn ball_is_a_filled_disc_with_its_color() {
        // Scaled: centre (50, 50), radius 10. Pixel centres are at +0.5.
        let f = render(&world(vec![ball(100.0, 100.0, 20.0)]), 0.5);
        assert_eq!(f.get(50, 50), 1);
        assert_eq!(f.get(59, 50), 1);
        assert_eq!(f.get(60, 50), 0);
        assert_eq!(f.get(40, 50), 1);
        assert_eq!(f.get(39, 50), 0);
        assert_eq!(f.get(50, 59), 1);
        assert_eq!(f.get(50, 60), 0);
        assert_eq!(f.get(58, 58), 0); // corner of the bounding square
        let filled = f.pixels.iter().filter(|&&p| p != 0).count() as f64;
        let area = std::f64::consts::PI * 100.0;
        assert!((filled - area).abs() < 0.05 * area, "filled {filled}");
    }

    #[test]
    fn ball_colors_cycle_through_16() {
        let mut balls = Vec::new();
        for i in 0..17 {
            let (col, row) = (i % 6, i / 6);
            balls.push(ball(
                60.0 + 120.0 * col as f64,
                60.0 + 120.0 * row as f64,
                20.0,
            ));
        }
        let f = render(&world(balls), 0.5);
        for i in 0..17usize {
            let (col, row) = (i % 6, i / 6);
            let (x, y) = (30 + 60 * col, 30 + 60 * row);
            assert_eq!(f.get(x, y) as usize, 1 + i % 16, "ball {i}");
        }
    }

    #[test]
    fn ball_touching_the_edge_is_clipped_safely() {
        let f = render(&world(vec![ball(20.0, 20.0, 20.0)]), 0.5);
        assert_eq!(f.get(0, 0), 0);
        assert_eq!(f.get(10, 10), 1);
    }
}
