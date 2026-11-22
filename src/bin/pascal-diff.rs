//! Compare Rust-emitted tick checkpoint JSONL against the Pascal schema-v2 trace.
//!
//! Usage: cargo run --bin pascal-diff -- pascal-rng.jsonl rust-checkpoints.jsonl

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process::ExitCode;
use street_chaves::pascal_differential::{first_divergence, load_tick_checkpoints};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: pascal-diff <pascal-trace.jsonl> <rust-checkpoints.jsonl>");
        return ExitCode::from(2);
    }

    let load = |path: &str| -> Result<_, String> {
        let file = File::open(path).map_err(|e| format!("{path}: {e}"))?;
        load_tick_checkpoints(BufReader::new(file)).map_err(|e| format!("{path}: {e}"))
    };

    let pascal = match load(&args[1]) {
        Ok(rows) => rows,
        Err(error) => { eprintln!("{error}"); return ExitCode::FAILURE; }
    };
    let rust = match load(&args[2]) {
        Ok(rows) => rows,
        Err(error) => { eprintln!("{error}"); return ExitCode::FAILURE; }
    };

    match first_divergence(&pascal, &rust) {
        None => {
            println!("MATCH: {} Pascal/Rust tick checkpoints compared", pascal.len());
            ExitCode::SUCCESS
        }
        Some(diff) => {
            println!(
                "FIRST DIVERGENCE: tick={} phase={:?} field={} Pascal={:?} Rust={:?}",
                diff.tick_index, diff.phase, diff.field, diff.pascal, diff.rust
            );
            ExitCode::FAILURE
        }
    }
}
