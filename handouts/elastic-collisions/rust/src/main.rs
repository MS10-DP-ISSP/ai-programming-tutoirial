use elastic_collisions::cli::{parse_args, run, usage};
use std::process::ExitCode;

fn main() -> ExitCode {
    let config = match parse_args(std::env::args().skip(1)) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("error: {e}\n{}", usage());
            return ExitCode::FAILURE;
        }
    };
    match run(&config) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
