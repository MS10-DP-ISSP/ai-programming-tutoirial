use crate::render::{render, Frame};
use crate::world::World;

pub const FPS: u32 = 25;
pub const SUBSTEPS: usize = 4;
/// Fixed physics time step in seconds.
pub const DT: f64 = 0.01;
/// Output pixels per world unit (800x600 -> 400x300).
pub const SCALE: f64 = 0.5;

/// Number of frames for a clip of `seconds` at `FPS`.
pub fn frame_count(seconds: f64) -> usize {
    (seconds * FPS as f64).round() as usize
}

/// Renders `frames` frames; each is drawn first, then the world advances `SUBSTEPS` steps.
pub fn animate(world: &mut World, frames: usize) -> Vec<Frame> {
    (0..frames)
        .map(|_| {
            let frame = render(world, SCALE);
            for _ in 0..SUBSTEPS {
                world.step(DT);
            }
            frame
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::render;
    use crate::world::World;

    #[test]
    fn constants_match_the_spec() {
        assert_eq!(FPS, 25);
        assert_eq!(SUBSTEPS, 4);
        assert_eq!(DT, 0.01);
        assert_eq!(SCALE, 0.5);
    }

    #[test]
    fn frame_count_rounds_seconds_times_fps() {
        assert_eq!(frame_count(10.0), 250);
        assert_eq!(frame_count(0.02), 1);
        assert_eq!(frame_count(0.019), 0);
        assert_eq!(frame_count(1.5), 38);
    }

    #[test]
    fn first_frame_is_the_initial_state_and_each_frame_follows_four_substeps() {
        let initial = World::random(10, 3).unwrap();
        let mut reference = initial.clone();
        let mut world = initial.clone();
        let frames = animate(&mut world, 5);
        assert_eq!(frames.len(), 5);
        for frame in &frames {
            assert_eq!(*frame, render(&reference, SCALE));
            for _ in 0..4 {
                reference.step(DT);
            }
        }
        assert_eq!(world.balls(), reference.balls());
    }

    #[test]
    fn frames_have_output_size() {
        let mut world = World::random(3, 1).unwrap();
        let f = &animate(&mut world, 1)[0];
        assert_eq!((f.width, f.height), (400, 300));
    }

    #[test]
    fn zero_frames_leaves_the_world_untouched() {
        let mut world = World::random(3, 1).unwrap();
        let before = world.clone();
        assert!(animate(&mut world, 0).is_empty());
        assert_eq!(world.balls(), before.balls());
    }
}
