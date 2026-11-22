//! Executable Pascal-derived AI runtime regressions using captured RNG draw segments.

use std::io::Cursor;

use street_chaves::combat_runtime::{
    AiCommand, AiTickInputs, BodyBox, CharacterActionScript, CombatState, FighterState,
    RuntimeDrawWidth, RuntimeError, SIDE_LEFT, SIDE_RIGHT,
};
use street_chaves::pascal_differential::{first_divergence, load_tick_checkpoints};
use street_chaves::pascal_replay::{PascalDrawReplay, PascalRandomSource};

const PASCAL_TICK_316: &str = include_str!("fixtures/pascal-runtime-schema-v2-tick316.jsonl");
const PASCAL_TICK_317: &str = include_str!("fixtures/pascal-runtime-ai-target-tick317.jsonl");
const PASCAL_TICK_381: &str = include_str!("fixtures/pascal-runtime-ai-close-range-tick381.jsonl");

fn fixture_state(tick_index: u64) -> CombatState {
    let body = BodyBox {
        x1: -10.0,
        y1: -40.0,
        x2: 10.0,
        y2: 0.0,
    };
    CombatState {
        tick_index,
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
                vertical_velocity: 0,
                in_air: false,
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
                vertical_velocity: 0,
                in_air: false,
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

fn replay(trace: &str) -> PascalDrawReplay {
    PascalDrawReplay::from_reader(Cursor::new(trace)).unwrap()
}

#[test]
fn tick_316_replays_real_random_gate_and_matches_pascal_checkpoints() {
    let mut replay = replay(PASCAL_TICK_316);
    let mut state = fixture_state(316);

    let result = state
        .tick_common_ai(
            AiTickInputs {
                distance: 150,
                projectile_threat: false,
                difficulty_factor: 3.6,
            },
            &mut replay,
        )
        .unwrap();

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
    replay.finish().unwrap();

    let pascal = load_tick_checkpoints(Cursor::new(PASCAL_TICK_316)).unwrap();
    let rust = load_tick_checkpoints(Cursor::new(result.checkpoint_jsonl)).unwrap();
    assert_eq!(first_divergence(&pascal, &rust), None);
}

#[test]
fn tick_317_replays_random_zero_and_random_six_target_selection() {
    let mut replay = replay(PASCAL_TICK_317);
    let mut state = fixture_state(317);
    state.fighters[1].action_id = 2;
    state.fighters[1].action_frame = 1;

    let result = state
        .tick_common_ai(
            AiTickInputs {
                distance: 100,
                projectile_threat: false,
                difficulty_factor: 3.6,
            },
            &mut replay,
        )
        .unwrap();

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
    assert_eq!(result.commands, vec![AiCommand::WalkLeft]);
    assert_eq!(state.fighters[1].action_id, 2);
    assert_eq!(state.fighters[1].action_frame, 2);
    replay.finish().unwrap();
}

#[test]
fn tick_381_replays_the_ordered_close_range_attack_probability_chain() {
    let mut replay = replay(PASCAL_TICK_381);
    let mut state = fixture_state(381);
    state.fighters[0].x = 200.0;
    state.fighters[1].action_id = 2;
    state.fighters[1].action_frame = 13;
    // The original trace does not export P.DistanciaIA or Dist. These are explicit
    // scenario inputs chosen to exercise the captured close-range path.
    state.fighters[1].distance_ai = 200;

    let result = state
        .tick_common_ai(
            AiTickInputs {
                distance: 50,
                projectile_threat: false,
                // Inferred from the captured Random ranges, not directly exported.
                difficulty_factor: 3.6,
            },
            &mut replay,
        )
        .unwrap();

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
    assert_eq!(result.commands, vec![AiCommand::WalkLeft, AiCommand::WeakKick, AiCommand::Crouch]);
    assert_eq!(state.fighters[1].action_id, 4);
    assert_eq!(state.fighters[1].action_frame, 0);
    assert_eq!(state.fighters[1].action_duration, 6);
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
        let Some((expected_width, expected_range, value)) =
            self.expected.get(self.cursor).copied()
        else {
            return Err(format!("unexpected extra RNG request at cursor {}", self.cursor));
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
    let mut state = fixture_state(400);
    let mut child = ChildThatConsumesOneDraw::default();

    let result = state
        .tick_common_ai_with_child(
            AiTickInputs {
                distance: 150,
                projectile_threat: false,
                difficulty_factor: 3.6,
            },
            &mut rng,
            &mut child,
        )
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
