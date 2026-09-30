use elastic_collisions::world::{HEIGHT, WIDTH};
use elastic_collisions::World;

const STEPS: usize = 5000;
const DT: f64 = 0.01;
const SEEDS: [u64; 4] = [0, 1, 42, 12345];
const COUNTS: [usize; 4] = [1, 5, 30, 80];

#[test]
fn kinetic_energy_is_conserved_over_long_runs() {
    for &seed in &SEEDS {
        for &n in &COUNTS {
            let mut world = World::random(n, seed).unwrap();
            let e0 = world.kinetic_energy();
            for _ in 0..STEPS {
                world.step(DT);
            }
            let rel = (world.kinetic_energy() - e0).abs() / e0;
            assert!(rel < 1e-9, "seed {seed}, n {n}: relative error {rel}");
        }
    }
}

#[test]
fn balls_stay_inside_the_box_at_every_step() {
    for &seed in &SEEDS {
        for &n in &COUNTS {
            let mut world = World::random(n, seed).unwrap();
            for step in 0..STEPS {
                world.step(DT);
                for b in world.balls() {
                    assert!(
                        b.pos.x - b.radius >= 0.0
                            && b.pos.x + b.radius <= WIDTH
                            && b.pos.y - b.radius >= 0.0
                            && b.pos.y + b.radius <= HEIGHT,
                        "seed {seed}, n {n}, step {step}: ball escaped at {:?}",
                        b.pos
                    );
                }
            }
        }
    }
}

#[test]
fn same_seed_gives_identical_trajectories() {
    let run = |seed| {
        let mut world = World::random(30, seed).unwrap();
        for _ in 0..STEPS {
            world.step(DT);
        }
        world.balls().to_vec()
    };
    assert_eq!(run(42), run(42));
    assert_ne!(run(42), run(43));
}
