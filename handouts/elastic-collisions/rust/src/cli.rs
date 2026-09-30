use crate::animation::{animate, frame_count};
use crate::error::Error;
use crate::gif_export::save_gif;
use crate::world::World;
use std::fmt;
use std::path::PathBuf;

/// Parsed command-line options.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub out: PathBuf,
    pub balls: usize,
    pub seed: u64,
    pub seconds: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            out: PathBuf::from("collisions.gif"),
            balls: 30,
            seed: 42,
            seconds: 10.0,
        }
    }
}

/// What `run` produced, for the final report.
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub out: PathBuf,
    pub frames: usize,
    pub initial_energy: f64,
    pub final_energy: f64,
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "output: {}", self.out.display())?;
        writeln!(f, "frames: {}", self.frames)?;
        writeln!(f, "initial kinetic energy: {:.6}", self.initial_energy)?;
        write!(f, "final kinetic energy:   {:.6}", self.final_energy)
    }
}

pub fn usage() -> String {
    "usage: elastic-collisions [--out collisions.gif] [--balls 30] [--seed 42] [--seconds 10]"
        .to_string()
}

fn invalid(msg: impl Into<String>) -> Error {
    Error::InvalidInput(msg.into())
}

/// Hand-written parser for `--out`, `--balls`, `--seed` and `--seconds` (each takes one value).
pub fn parse_args<I, S>(args: I) -> Result<Config, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut config = Config::default();
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        let flag = flag.as_ref();
        if !matches!(flag, "--out" | "--balls" | "--seed" | "--seconds") {
            return Err(invalid(format!("unknown argument: {flag}")));
        }
        let value = match args.next() {
            Some(v) if !v.as_ref().starts_with("--") => v,
            _ => return Err(invalid(format!("missing value for {flag}"))),
        };
        let value = value.as_ref();
        match flag {
            "--out" => config.out = PathBuf::from(value),
            "--balls" => {
                config.balls = match value.parse::<i64>() {
                    Ok(n) if n > 0 => n as usize,
                    Ok(_) => return Err(invalid("--balls must be a positive integer")),
                    Err(_) => return Err(invalid(format!("--balls: not an integer: {value:?}"))),
                }
            }
            "--seed" => {
                config.seed = value.parse::<u64>().map_err(|_| {
                    invalid(format!("--seed: not a non-negative integer: {value:?}"))
                })?
            }
            _ => {
                config.seconds = match value.parse::<f64>() {
                    Ok(s) if s.is_finite() && s > 0.0 => s,
                    _ => {
                        return Err(invalid(format!(
                            "--seconds: not a positive finite number: {value:?}"
                        )))
                    }
                };
                if frame_count(config.seconds) == 0 {
                    return Err(invalid(format!(
                        "--seconds {value}: too short to make a single frame at 25 fps"
                    )));
                }
            }
        }
    }
    Ok(config)
}

/// Simulates, renders and writes the GIF.
pub fn run(config: &Config) -> Result<Summary, Error> {
    let mut world = World::random(config.balls, config.seed)?;
    let initial_energy = world.kinetic_energy();
    let frames = animate(&mut world, frame_count(config.seconds));
    let (width, height) = frames
        .first()
        .map(|f| (f.width, f.height))
        .ok_or(Error::NoFrames)?;
    save_gif(&config.out, width, height, &frames)?;
    Ok(Summary {
        out: config.out.clone(),
        frames: frames.len(),
        initial_energy,
        final_energy: world.kinetic_energy(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn parse(args: &[&str]) -> Result<Config, Error> {
        parse_args(args.iter().copied())
    }

    fn rejected(args: &[&str]) -> bool {
        matches!(parse(args), Err(Error::InvalidInput(_)))
    }

    #[test]
    fn defaults() {
        let c = parse(&[]).unwrap();
        assert_eq!(c.out, PathBuf::from("collisions.gif"));
        assert_eq!(c.balls, 30);
        assert_eq!(c.seed, 42);
        assert_eq!(c.seconds, 10.0);
    }

    #[test]
    fn parses_all_options() {
        let c = parse(&[
            "--out",
            "a.gif",
            "--balls",
            "5",
            "--seed",
            "7",
            "--seconds",
            "1.5",
        ])
        .unwrap();
        assert_eq!(c.out, PathBuf::from("a.gif"));
        assert_eq!(c.balls, 5);
        assert_eq!(c.seed, 7);
        assert_eq!(c.seconds, 1.5);
    }

    #[test]
    fn options_may_come_in_any_order_and_partially() {
        let c = parse(&["--seed", "0", "--balls", "1"]).unwrap();
        assert_eq!((c.balls, c.seed, c.seconds), (1, 0, 10.0));
    }

    #[test]
    fn rejects_unknown_arguments() {
        assert!(rejected(&["--bogus"]));
        assert!(rejected(&["--balls", "3", "extra"]));
        assert!(rejected(&["--help"]));
    }

    #[test]
    fn rejects_missing_values() {
        assert!(rejected(&["--out"]));
        assert!(rejected(&["--balls"]));
        assert!(rejected(&["--seed", "--balls", "3"]));
        assert!(rejected(&["--out", "--seed", "1"]));
        assert!(rejected(&["--seconds", "--balls"]));
    }

    #[test]
    fn rejects_bad_balls() {
        for v in ["0", "-1", "1.5", "abc", "", "99999999999999999999"] {
            assert!(rejected(&["--balls", v]), "--balls {v:?}");
        }
    }

    #[test]
    fn rejects_bad_seed() {
        for v in ["-1", "1.5", "abc", ""] {
            assert!(rejected(&["--seed", v]), "--seed {v:?}");
        }
    }

    #[test]
    fn rejects_bad_seconds() {
        for v in ["0", "-1", "nan", "inf", "-inf", "abc", "", "0.019"] {
            assert!(rejected(&["--seconds", v]), "--seconds {v:?}");
        }
    }

    #[test]
    fn accepts_shortest_clip_of_one_frame() {
        assert!(parse(&["--seconds", "0.02"]).is_ok());
    }

    #[test]
    fn usage_mentions_every_option() {
        let u = usage();
        for opt in ["--out", "--balls", "--seed", "--seconds"] {
            assert!(u.contains(opt), "usage lacks {opt}");
        }
    }

    #[test]
    fn run_writes_a_gif_and_reports_energies() {
        let out =
            std::env::temp_dir().join(format!("elastic-collisions-cli-{}.gif", std::process::id()));
        let config = Config {
            out: out.clone(),
            balls: 5,
            seed: 1,
            seconds: 0.2,
        };
        let summary = run(&config).unwrap();
        let exists = out.exists();
        let _ = std::fs::remove_file(&out);
        assert!(exists);
        assert_eq!(summary.frames, 5);
        assert_eq!(summary.out, out);
        assert!(summary.initial_energy > 0.0);
        assert!(
            (summary.final_energy - summary.initial_energy).abs() / summary.initial_energy < 1e-9
        );
    }

    #[test]
    fn run_reports_placement_failure() {
        let config = Config {
            out: PathBuf::from("unused.gif"),
            balls: 5000,
            seed: 1,
            seconds: 1.0,
        };
        assert!(matches!(run(&config), Err(Error::PlacementFailed)));
    }

    #[test]
    fn summary_display_lists_path_frames_and_energies() {
        let s = Summary {
            out: PathBuf::from("x.gif"),
            frames: 250,
            initial_energy: 1.0,
            final_energy: 2.0,
        };
        let text = s.to_string();
        assert!(text.contains("x.gif") && text.contains("250"));
        assert!(text.contains("1.0") && text.contains("2.0"));
    }
}
