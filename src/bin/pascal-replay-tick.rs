//! Replay the tick-316 Pascal AI sample through the first Rust runtime slice.
//!
//! Usage:
//! cargo run --bin pascal-replay-tick -- tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl
//!
//! The source trace does not export Dist/DistanciaIA/body geometry. The bounded
//! scenario below supplies admissible values that select the observed left-walk
//! branch while preserving the captured Random(13) request/result. It is not a
//! claim that these hidden input values were the original tick-316 values.

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process::ExitCode;

use street_chaves::combat_runtime::{
    AiTickInputs, BodyBox, CombatState, FighterState, SIDE_LEFT, SIDE_RIGHT,
};
use street_chaves::pascal_replay::PascalDrawReplay;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: pascal-replay-tick <pascal-tick-trace.jsonl>");
        return ExitCode::from(2);
    }

    let file = match File::open(&args[1]) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("{}: {error}", args[1]);
            return ExitCode::FAILURE;
        }
    };
    let mut replay = match PascalDrawReplay::from_reader(BufReader::new(file)) {
        Ok(replay) => replay,
        Err(error) => {
            eprintln!("invalid Pascal replay input: {error}");
            return ExitCode::FAILURE;
        }
    };

    let body = BodyBox {
        x1: -10.0,
        y1: -40.0,
        x2: 10.0,
        y2: 0.0,
    };
    let mut state = CombatState {
        tick_index: 316,
        game_state: 7,
        camera_x: 0.0,
        clock_stopped: false,
        fighters: [
            FighterState {
                character_id: 2,
                action_id: 1,
                action_frame: 0,
                action_duration: 99,
                facing: SIDE_RIGHT,
                defending: false,
                x: 100.0,
                y: 0.0,
                speed: 3.0,
                distance_ai: 250,
                health: 100,
                body,
            },
            FighterState {
                character_id: 1,
                action_id: 1,
                action_frame: 0,
                action_duration: 99,
                facing: SIDE_RIGHT,
                defending: false,
                x: 250.0,
                y: 0.0,
                speed: 3.0,
                distance_ai: 250,
                health: 100,
                body,
            },
        ],
    };
    // Make the initial values explicit: fighter 1's facing is not used by this AI pass.
    state.fighters[0].facing = SIDE_LEFT;

    let tick = match state.tick_common_ai_walk(
        AiTickInputs {
            distance: 150,
            projectile_threat: false,
        },
        &mut replay,
    ) {
        Ok(tick) => tick,
        Err(error) => {
            eprintln!("runtime stopped at an unsupported or divergent branch: {error}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(error) = replay.finish() {
        eprintln!("Pascal draw stream was not fully consumed: {error}");
        return ExitCode::FAILURE;
    }

    for draw in &tick.draws {
        eprintln!("RNG LongInt range={} -> {}", draw.range, draw.value);
    }
    print!("{}", tick.checkpoint_jsonl);
    ExitCode::SUCCESS
}
