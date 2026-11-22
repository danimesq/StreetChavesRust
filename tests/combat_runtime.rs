//! Executable vertical-slice regression against the real Pascal tick-316 sample.

use std::io::Cursor;

use street_chaves::combat_runtime::{
    AiTickInputs, BodyBox, CombatState, FighterState, SIDE_LEFT, SIDE_RIGHT,
};
use street_chaves::pascal_differential::{first_divergence, load_tick_checkpoints};
use street_chaves::pascal_replay::PascalDrawReplay;

const PASCAL_TICK_316: &str = include_str!("fixtures/pascal-runtime-schema-v2-tick316.jsonl");

fn fixture_state() -> CombatState {
    let body = BodyBox {
        x1: -10.0,
        y1: -40.0,
        x2: 10.0,
        y2: 0.0,
    };
    CombatState {
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
                facing: SIDE_LEFT,
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
    }
}

#[test]
fn tick_316_replays_real_random_gate_and_matches_pascal_checkpoints() {
    let mut replay = PascalDrawReplay::from_reader(Cursor::new(PASCAL_TICK_316)).unwrap();
    let mut state = fixture_state();

    let result = state
        .tick_common_ai_walk(
            AiTickInputs {
                distance: 150,
                projectile_threat: false,
            },
            &mut replay,
        )
        .unwrap();

    assert_eq!(result.draws.len(), 1);
    assert_eq!((result.draws[0].range, result.draws[0].value), (13, 12));
    assert_eq!(state.fighters[1].action_id, 2);
    assert_eq!(state.fighters[1].action_frame, 1);
    assert_eq!(state.fighters[1].action_duration, 99);
    replay.finish().unwrap();

    let pascal = load_tick_checkpoints(Cursor::new(PASCAL_TICK_316)).unwrap();
    let rust = load_tick_checkpoints(Cursor::new(result.checkpoint_jsonl)).unwrap();
    assert_eq!(first_divergence(&pascal, &rust), None);
}

#[test]
fn unsupported_close_range_attack_chain_fails_closed() {
    let mut replay = PascalDrawReplay::from_reader(Cursor::new(PASCAL_TICK_316)).unwrap();
    let mut state = fixture_state();

    let result = state.tick_common_ai_walk(
        AiTickInputs {
            distance: 50,
            projectile_threat: false,
        },
        &mut replay,
    );
    assert!(matches!(
        result,
        Err(
            street_chaves::combat_runtime::RuntimeError::UnsupportedBranch(
                "Common.rot [IA]: close-range attack probability chain"
            )
        )
    ));
}


#[test]
fn replay_and_pascal_diff_clis_round_trip_tick_316() {
    use std::fs;
    use std::process::Command;

    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl");
    let replay = Command::new(env!("CARGO_BIN_EXE_pascal-replay-tick"))
        .arg(&fixture_path)
        .output()
        .expect("run pascal-replay-tick CLI");
    assert!(
        replay.status.success(),
        "replay CLI failed: {}",
        String::from_utf8_lossy(&replay.stderr)
    );

    let rust_path = std::env::temp_dir().join(format!(
        "street-chaves-rust-tick316-{}.jsonl",
        std::process::id()
    ));
    fs::write(&rust_path, &replay.stdout).expect("write Rust checkpoint JSONL");

    let diff = Command::new(env!("CARGO_BIN_EXE_pascal-diff"))
        .arg(&fixture_path)
        .arg(&rust_path)
        .output()
        .expect("run pascal-diff CLI");
    let _ = fs::remove_file(&rust_path);

    assert!(
        diff.status.success(),
        "pascal-diff CLI failed: {}",
        String::from_utf8_lossy(&diff.stdout)
    );
    assert!(
        String::from_utf8_lossy(&diff.stdout).contains("MATCH: 2 Pascal/Rust tick checkpoints compared"),
        "unexpected differential output: {}",
        String::from_utf8_lossy(&diff.stdout)
    );
}
