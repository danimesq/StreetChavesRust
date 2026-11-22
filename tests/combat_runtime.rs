//! Executable Pascal-derived AI runtime regressions using captured RNG draw segments.

use std::io::Cursor;

use serde_json::Value;
use street_chaves::combat_runtime::{
    AiCommand, AiTickInputs, BodyBox, CharacterActionScript, CombatState, FighterState,
    RuntimeDrawWidth, RuntimeError, SIDE_LEFT, SIDE_RIGHT,
};
use street_chaves::pascal_differential::{first_divergence, load_tick_checkpoints};
use street_chaves::pascal_replay::{PascalDrawReplay, PascalRandomSource};

const PASCAL_TICK_316: &str = include_str!("fixtures/pascal-runtime-schema-v2-tick316.jsonl");
const PASCAL_TICK_317: &str = include_str!("fixtures/pascal-runtime-ai-target-tick317.jsonl");
const PASCAL_TICK_381: &str = include_str!("fixtures/pascal-runtime-ai-close-range-tick381.jsonl");

fn required_i32(value: &Value, key: &str) -> i32 {
    value[key]
        .as_i64()
        .unwrap_or_else(|| panic!("missing integer snapshot field {key}")) as i32
}

fn required_f64(value: &Value, key: &str) -> f64 {
    value[key]
        .as_f64()
        .unwrap_or_else(|| panic!("missing numeric snapshot field {key}"))
}

fn fighter_from_snapshot(value: &Value) -> FighterState {
    let body = value["body_box"]
        .as_array()
        .expect("snapshot body_box array");
    assert_eq!(body.len(), 4);

    FighterState {
        character_id: required_i32(value, "character_id") as u8,
        action_id: required_i32(value, "action_id") as u16,
        action_frame: required_i32(value, "action_frame") as u16,
        action_duration: required_i32(value, "action_duration") as u16,
        facing: required_i32(value, "facing") as u8,
        defending: value["defending"].as_bool().expect("snapshot defending"),
        vertical_velocity: required_i32(value, "velocity_y") as i8,
        in_air: value["in_air"].as_bool().expect("snapshot in_air"),
        x: required_f64(value, "x"),
        y: required_f64(value, "y"),
        speed: required_f64(value, "speed"),
        distance_ai: required_i32(value, "distance_ai"),
        health: required_i32(value, "health"),
        body: BodyBox {
            x1: body[0].as_f64().expect("body_box x1"),
            y1: body[1].as_f64().expect("body_box y1"),
            x2: body[2].as_f64().expect("body_box x2"),
            y2: body[3].as_f64().expect("body_box y2"),
        },
    }
}

/// Reconstruct the common IA entry state and all five values passed to the Pascal
/// .rot [IA] script directly from its captured script_start.ai_inputs object.
fn ai_scenario(trace: &str, tick_index: u64) -> (CombatState, AiTickInputs) {
    let mut game_state = None;
    let mut ai_inputs = None;

    for line in trace.lines() {
        let record: Value = serde_json::from_str(line).expect("valid trace JSONL");
        if record["tick_index"].as_u64() != Some(tick_index) {
            continue;
        }

        match record["record_type"].as_str() {
            Some("tick_start") => game_state = record["game_state"].as_u64().map(|n| n as u8),
            Some("script_start") if record["script_origin"].as_str() == Some("Comum.rot [IA]") => {
                ai_inputs = Some(record["ai_inputs"].clone());
            }
            _ => {}
        }
    }

    let ai = ai_inputs
        .unwrap_or_else(|| panic!("tick {tick_index} lacks captured Comum.rot [IA] inputs"));
    let fighter = fighter_from_snapshot(&ai["fighter"]);
    let opponent = fighter_from_snapshot(&ai["opponent"]);

    let state = CombatState {
        tick_index,
        game_state: game_state.unwrap_or(7),
        // The trace captures fighter positions and body boxes. Camera position is not
        // part of ai_inputs yet; these three paths remain away from camera clamp edges.
        camera_x: 0.0,
        clock_stopped: false,
        fighters: [opponent, fighter],
    };

    let inputs = AiTickInputs {
        distance: required_i32(&ai, "dist"),
        projectile_distance: required_i32(&ai, "dist_proj"),
        opponent_y: required_i32(&ai, "opp_y"),
        threat: required_i32(&ai, "threat"),
        difficulty_factor: required_f64(&ai, "difficulty_factor"),
    };

    (state, inputs)
}

fn replay(trace: &str) -> PascalDrawReplay {
    PascalDrawReplay::from_reader(Cursor::new(trace)).unwrap()
}

#[test]
fn tick_316_uses_captured_inputs_and_matches_pascal_checkpoints() {
    let mut replay = replay(PASCAL_TICK_316);
    let (mut state, inputs) = ai_scenario(PASCAL_TICK_316, 316);

    assert_eq!(inputs.distance, 132);
    assert_eq!(inputs.projectile_distance, 0);
    assert_eq!(inputs.opponent_y, 300);
    assert_eq!(inputs.threat, 0);
    assert_eq!(inputs.difficulty_factor, 3.6);
    assert_eq!(state.fighters[1].distance_ai, 0);
    assert_eq!(state.fighters[1].facing, SIDE_LEFT);
    assert_eq!(state.fighters[1].x, 366.0);
    assert_eq!(state.fighters[0].x, 234.0);

    let result = state.tick_common_ai(inputs, &mut replay).unwrap();

    assert_eq!(result.draws.len(), 1);
    assert_eq!(
        (
            result.draws[0].width,
            result.draws[0].range,
            result.draws[0].value
        ),
        (RuntimeDrawWidth::LongInt, 13, 12)
    );
    assert_eq!(result.commands, vec![AiCommand::WalkLeft]);
    assert_eq!(state.fighters[1].action_id, 2);
    assert_eq!(state.fighters[1].action_frame, 1);
    assert_eq!(state.fighters[1].action_duration, 99);
    assert_eq!(state.fighters[1].x, 358.0);
    replay.finish().unwrap();

    let pascal = load_tick_checkpoints(Cursor::new(PASCAL_TICK_316)).unwrap();
    let rust = load_tick_checkpoints(Cursor::new(result.checkpoint_jsonl)).unwrap();
    assert_eq!(first_divergence(&pascal, &rust), None);
}

#[test]
fn tick_317_replays_random_zero_and_random_six_with_captured_inputs() {
    let mut replay = replay(PASCAL_TICK_317);
    let (mut state, inputs) = ai_scenario(PASCAL_TICK_317, 317);

    assert_eq!(inputs.distance, 124);
    assert_eq!(state.fighters[1].distance_ai, 0);
    assert_eq!(state.fighters[1].facing, SIDE_LEFT);
    assert_eq!(state.fighters[1].x, 358.0);

    let result = state.tick_common_ai(inputs, &mut replay).unwrap();

    let actual: Vec<_> = result
        .draws
        .iter()
        .map(|draw| (draw.width, draw.range, draw.value))
        .collect();
    assert_eq!(
        actual,
        vec![
            (RuntimeDrawWidth::LongInt, 13, 0),
            (RuntimeDrawWidth::LongInt, 6, 2),
        ]
    );
    assert_eq!(state.fighters[1].distance_ai, 200);
    assert_eq!(result.commands, vec![AiCommand::WalkRight]);
    assert_eq!(state.fighters[1].action_id, 2);
    assert_eq!(state.fighters[1].action_frame, 2);
    assert_eq!(state.fighters[1].x, 366.0);
    replay.finish().unwrap();
}

#[test]
fn tick_381_replays_the_ordered_close_range_chain_with_captured_inputs() {
    let mut replay = replay(PASCAL_TICK_381);
    let (mut state, inputs) = ai_scenario(PASCAL_TICK_381, 381);

    assert_eq!(inputs.distance, 78);
    assert_eq!(inputs.difficulty_factor, 3.6);
    assert_eq!(state.fighters[1].distance_ai, 300);
    assert_eq!(state.fighters[1].facing, SIDE_LEFT);
    assert_eq!(state.fighters[1].x, 356.0);
    assert_eq!(state.fighters[0].action_id, 3);
    assert_eq!(state.fighters[0].action_frame, 1);

    let result = state.tick_common_ai(inputs, &mut replay).unwrap();

    let actual: Vec<_> = result
        .draws
        .iter()
        .map(|draw| (draw.width, draw.range, draw.value))
        .collect();
    assert_eq!(
        actual,
        vec![
            (RuntimeDrawWidth::LongInt, 13, 7),
            (RuntimeDrawWidth::Int64, 29, 25),
            (RuntimeDrawWidth::Int64, 29, 0),
            (RuntimeDrawWidth::Int64, 32, 1),
            (RuntimeDrawWidth::Int64, 32, 8),
            (RuntimeDrawWidth::Int64, 14, 0),
            (RuntimeDrawWidth::Int64, 7, 3),
            (RuntimeDrawWidth::Int64, 7, 3),
            (RuntimeDrawWidth::Int64, 11, 6),
            (RuntimeDrawWidth::Int64, 11, 5),
        ]
    );
    assert_eq!(
        result.commands,
        vec![AiCommand::WalkRight, AiCommand::WeakKick, AiCommand::Crouch]
    );
    assert_eq!(state.fighters[1].action_id, 4);
    assert_eq!(state.fighters[1].action_frame, 0);
    assert_eq!(state.fighters[1].action_duration, 6);
    assert_eq!(state.fighters[1].x, 364.0);
    replay.finish().unwrap();
}

/// Small deterministic RNG stream used to prove that a child executor receives the
/// exact parent cursor, rather than constructing or resetting its own RNG.
struct SequenceRng {
    expected: Vec<(RuntimeDrawWidth, i64, i64)>,
    cursor: usize,
}

impl SequenceRng {
    fn new(expected: Vec<(RuntimeDrawWidth, i64, i64)>) -> Self {
        Self {
            expected,
            cursor: 0,
        }
    }

    fn draw(&mut self, width: RuntimeDrawWidth, range: i64) -> Result<i64, String> {
        let Some((expected_width, expected_range, value)) = self.expected.get(self.cursor).copied()
        else {
            return Err(format!(
                "unexpected extra RNG request at cursor {}",
                self.cursor
            ));
        };
        if (width, range) != (expected_width, expected_range) {
            return Err(format!(
                "RNG mismatch at cursor {}: expected {expected_width:?}/{expected_range}, got {width:?}/{range}",
                self.cursor
            ));
        }
        self.cursor += 1;
        Ok(value)
    }
}

impl PascalRandomSource for SequenceRng {
    type Error = String;

    fn draw_longint(&mut self, range: i32) -> Result<i32, Self::Error> {
        i32::try_from(self.draw(RuntimeDrawWidth::LongInt, range as i64)?)
            .map_err(|error| error.to_string())
    }

    fn draw_int64(&mut self, range: i64) -> Result<i64, Self::Error> {
        self.draw(RuntimeDrawWidth::Int64, range)
    }
}

#[derive(Default)]
struct ChildThatConsumesOneDraw {
    values: Vec<i32>,
}

impl CharacterActionScript for ChildThatConsumesOneDraw {
    fn execute<R: PascalRandomSource>(
        &mut self,
        _state: &mut CombatState,
        _fighter_index: usize,
        _command: AiCommand,
        rng: &mut R,
    ) -> Result<(), RuntimeError<R::Error>> {
        let value = rng.draw_longint(7).map_err(RuntimeError::Rng)?;
        self.values.push(value);
        Ok(())
    }
}

#[test]
fn nested_character_action_consumes_the_parent_rng_cursor() {
    let mut rng = SequenceRng::new(vec![
        (RuntimeDrawWidth::LongInt, 13, 12),
        (RuntimeDrawWidth::LongInt, 7, 3),
    ]);
    let (mut state, inputs) = ai_scenario(PASCAL_TICK_316, 316);
    state.tick_index = 400;
    let mut child = ChildThatConsumesOneDraw::default();

    let result = state
        .tick_common_ai_with_child(inputs, &mut rng, &mut child)
        .unwrap();

    assert_eq!(rng.cursor, 2);
    assert_eq!(child.values, vec![3]);
    assert_eq!(result.commands, vec![AiCommand::WalkLeft]);
    assert_eq!(
        result
            .draws
            .iter()
            .map(|draw| (draw.width, draw.range, draw.value))
            .collect::<Vec<_>>(),
        vec![
            (RuntimeDrawWidth::LongInt, 13, 12),
            (RuntimeDrawWidth::LongInt, 7, 3),
        ]
    );
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
        String::from_utf8_lossy(&diff.stdout)
            .contains("MATCH: 2 Pascal/Rust tick checkpoints compared"),
        "unexpected differential output: {}",
        String::from_utf8_lossy(&diff.stdout)
    );
}
