//! Executable vertical-slice regression against the real Pascal tick-316 sample.

use std::io::Cursor;

use street_chaves::combat_runtime::{
    AiTickInputs, BodyBox, CombatState, FighterState, SIDE_LEFT, SIDE_RIGHT,
};
use street_chaves::pascal_differential::{first_divergence, load_tick_checkpoints};
use street_chaves::pascal_replay::PascalDrawReplay;

const PASCAL_TICK_316: &str =
    include_str!("fixtures/pascal-runtime-schema-v2-tick316.jsonl");

fn fixture_state() -> CombatState {
    let body = BodyBox { x1: -10.0, y1: -40.0, x2: 10.0, y2: 0.0 };
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
            AiTickInputs { distance: 150, projectile_threat: false },
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
        AiTickInputs { distance: 50, projectile_threat: false },
        &mut replay,
    );
    assert!(matches!(
        result,
        Err(street_chaves::combat_runtime::RuntimeError::UnsupportedBranch(
            "Common.rot [IA]: close-range attack probability chain"
        ))
    ));
}
