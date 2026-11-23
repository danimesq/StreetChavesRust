//! Executable Pascal-derived AI runtime regressions using captured RNG draw segments.

use std::io::Cursor;

use serde_json::Value;
use street_chaves::combat_runtime::{
    AiCommand, AiTickInputs, BodyBox, CharacterActionScript, CombatState, FighterState,
    RuntimeDrawWidth, RuntimeError, VictoryAnimationContext, ACTION_CROUCHING, ACTION_IDLE,
    ACTION_POSE, ACTION_SPECIAL1, ACTION_SPECIAL2, ACTION_SPECIAL3, ACTION_SPECIAL4,
    ACTION_SPECIAL_FINAL, ACTION_VICTORY, ACTION_WALKING, FORCE_STRONG, FORCE_WEAK, SIDE_LEFT,
    SIDE_RIGHT,
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
        sprite_frame: 0,
        character_id: required_i32(value, "character_id") as u8,
        action_id: required_i32(value, "action_id") as u16,
        action_frame: required_i32(value, "action_frame") as u16,
        action_duration: required_i32(value, "action_duration") as u16,
        facing: required_i32(value, "facing") as u8,
        force_strength: 0,
        defending: value["defending"].as_bool().expect("snapshot defending"),
        invulnerable: value["invulnerable"].as_bool().unwrap_or(false),
        hit_count: value["hits"].as_u64().unwrap_or(0) as u8,
        is_kick: false,
        recoil: 0,
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
        effects: Default::default(),
        pending_sound: None,
        screen_shake: 0,
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

const NESTED_CHILD_REPLAY: &str = concat!(
    "{\"record_type\":\"session\",\"schema_version\":2,\"session_id\":\"nested-child-test\",\"rng_scope\":\"SortearValor\",\"rand_seed_initial\":1}\n",
    "{\"record_type\":\"draw\",\"schema_version\":2,\"session_id\":\"nested-child-test\",\"api_call_index\":0,\"first_mt_word_index\":0,\"raw_mt_words\":1,\"width\":\"LongInt\",\"range\":13,\"result\":12,\"source_kind\":\"rot_expression\",\"source_origin\":\"Comum.rot [IA]\",\"source_line\":293,\"source_column\":4,\"script_execution_id\":900,\"parent_script_execution_id\":null}\n",
    "{\"record_type\":\"draw\",\"schema_version\":2,\"session_id\":\"nested-child-test\",\"api_call_index\":1,\"first_mt_word_index\":1,\"raw_mt_words\":1,\"width\":\"LongInt\",\"range\":7,\"result\":3,\"source_kind\":\"rot_expression\",\"source_origin\":\"Character.rot [Acao]\",\"source_line\":42,\"source_column\":8,\"script_execution_id\":901,\"parent_script_execution_id\":900}\n",
);

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
fn nested_character_action_consumes_the_parent_replay_cursor() {
    // The second captured-style draw belongs to a child script. Its parent execution
    // ID points back to the Common IA script; the runtime must consume it on the same
    // PascalDrawReplay cursor after the parent's Random(13) request.
    let mut replay = PascalDrawReplay::from_reader(Cursor::new(NESTED_CHILD_REPLAY)).unwrap();
    let (mut state, inputs) = ai_scenario(PASCAL_TICK_316, 316);
    state.tick_index = 400;
    let mut child = ChildThatConsumesOneDraw::default();

    let result = state
        .tick_common_ai_with_child(inputs, &mut replay, &mut child)
        .unwrap();

    assert_eq!(replay.draws_consumed(), 2);
    assert_eq!(replay.draws_remaining(), 0);
    replay.finish().unwrap();
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

#[derive(Default)]
struct ScriptedRng {
    values: Vec<i32>,
    cursor: usize,
}

impl ScriptedRng {
    fn with_values(values: &[i32]) -> Self {
        Self {
            values: values.to_vec(),
            cursor: 0,
        }
    }
}

impl PascalRandomSource for ScriptedRng {
    type Error = String;

    fn draw_longint(&mut self, range: i32) -> Result<i32, Self::Error> {
        let value = *self
            .values
            .get(self.cursor)
            .ok_or_else(|| format!("missing scripted draw at {}", self.cursor))?;
        self.cursor += 1;
        if value < 0 || value >= range {
            return Err(format!("scripted value {value} outside range {range}"));
        }
        Ok(value)
    }

    fn draw_int64(&mut self, range: i64) -> Result<i64, Self::Error> {
        self.draw_longint(range as i32).map(i64::from)
    }
}

fn victory_context() -> VictoryAnimationContext {
    VictoryAnimationContext {
        sprite_offset: 1000,
        ground_y: 300.0,
        camera_x: 0.0,
        camera_left_limit: 100.0,
        camera_right_limit: 300.0,
    }
}

fn victory_state(character_id: u8, action_frame: u16, sprite_frame: i32) -> CombatState {
    let (mut state, _) = ai_scenario(PASCAL_TICK_316, 316);
    let fighter = &mut state.fighters[1];
    fighter.character_id = character_id;
    fighter.action_id = ACTION_VICTORY;
    fighter.action_frame = action_frame;
    fighter.sprite_frame = sprite_frame;
    fighter.x = 200.0;
    fighter.y = 290.0;
    fighter.facing = SIDE_RIGHT;
    state
}

fn c01_frame_state(action_id: u16, action_frame: u16, sprite_frame: i32) -> CombatState {
    let mut state = victory_state(1, action_frame, sprite_frame);
    state.fighters[0].x = 40.0;
    state.fighters[1].action_id = action_id;
    state.fighters[1].force_strength = FORCE_WEAK;
    state
}

#[test]
fn frame_only_victory_sequences_match_audited_boundaries() {
    let cases = [
        (1, 0, 1033),
        (1, 7, 1034),
        (1, 14, 1035),
        (1, 21, 1034),
        (1, 28, 1033),
        (3, 0, 1033),
        (3, 5, 1034),
        (3, 10, 1035),
        (3, 15, 1034),
        (3, 20, 1033),
        (3, 25, 1034),
        (3, 30, 1035),
        (3, 35, 1034),
        (4, 12, 1033),
        (4, 13, 1034),
        (5, 4, 1033),
        (5, 9, 1034),
        (5, 13, 1033),
        (5, 41, 1035),
        (7, 0, 1033),
        (7, 1, 1034),
        (10, 11, 1033),
        (10, 12, 1034),
        (11, 200, 1033),
    ];

    for (character_id, frame, expected) in cases {
        let mut state = victory_state(character_id, frame, 0);
        let mut rng = ScriptedRng::default();
        state
            .update_victory_animation(1, victory_context(), &mut rng)
            .unwrap();
        assert_eq!(
            state.fighters[1].sprite_frame, expected,
            "C{character_id:02} at action frame {frame}"
        );
        assert_eq!(rng.cursor, 0);
        if character_id == 1 && frame == 28 {
            assert_eq!(state.fighters[1].action_frame, 0);
        } else {
            assert_eq!(state.fighters[1].action_frame, frame);
        }
    }
}

#[test]
fn victory_toggle_characters_read_both_prior_sprite_frames() {
    for (character_id, boundary) in [(6, 4), (9, 3), (14, 3)] {
        for previous in [1033, 1034] {
            let mut state = victory_state(character_id, boundary, previous);
            let mut rng = ScriptedRng::default();
            let result = state
                .update_victory_animation(1, victory_context(), &mut rng)
                .unwrap();
            assert_eq!(result.draws.len(), 0);
            assert_eq!(rng.cursor, 0);
            assert_eq!(
                state.fighters[1].sprite_frame,
                if previous == 1033 { 1034 } else { 1033 },
                "C{character_id:02} must toggle from prior frame {previous}"
            );
        }
    }
}

#[test]
fn victory_toggle_characters_preserve_sprite_frame_off_boundary() {
    for character_id in [6, 9, 14] {
        let mut state = victory_state(character_id, 2, 1047);
        let mut rng = ScriptedRng::default();
        state
            .update_victory_animation(1, victory_context(), &mut rng)
            .unwrap();
        assert_eq!(state.fighters[1].sprite_frame, 1047);
        assert_eq!(rng.cursor, 0);
    }
}

#[test]
fn c02_victory_consumes_exactly_one_shared_random_two_draw() {
    let mut state = victory_state(2, 17, 0);
    let mut rng = ScriptedRng::with_values(&[0]);
    let result = state
        .update_victory_animation(1, victory_context(), &mut rng)
        .unwrap();
    assert_eq!(state.fighters[1].sprite_frame, 1034);
    assert_eq!(rng.cursor, 1);
    assert_eq!(result.draws.len(), 1);
    assert_eq!((result.draws[0].range, result.draws[0].value), (2, 0));
}

#[test]
fn c12_victory_preserves_movement_clamps_rng_and_particle_order() {
    let mut state = victory_state(12, 8, 0);
    state.fighters[1].x = 289.0;
    state.fighters[1].facing = SIDE_RIGHT;
    // Random(2) chooses no reversal; then five ordered Random(40)/Random(10) pairs.
    let mut values = vec![1];
    for pair in 0..5 {
        values.push(pair);
        values.push(9 - pair);
    }
    let mut rng = ScriptedRng::with_values(&values);
    let result = state
        .update_victory_animation(1, victory_context(), &mut rng)
        .unwrap();
    assert_eq!(state.fighters[1].x, 301.0);
    assert_eq!(state.fighters[1].facing, SIDE_LEFT);
    assert_eq!(state.fighters[1].sprite_frame, 1033);
    assert_eq!(rng.cursor, 11);
    assert_eq!(
        result
            .draws
            .iter()
            .map(|draw| draw.range)
            .collect::<Vec<_>>(),
        vec![2, 40, 10, 40, 10, 40, 10, 40, 10, 40, 10]
    );
    assert_eq!(result.spawned_effect_slots, vec![0, 1, 2, 3, 4]);
    assert_eq!(state.effects.active_count(), 5);
    let first_effect = state
        .effects
        .instance(0)
        .expect("first live C12 dust effect");
    assert_eq!(first_effect.effect_type, 2);
    assert_eq!(first_effect.owner, 0);
    assert_eq!(first_effect.x, 271.0);
    assert_eq!(first_effect.y, 299.0);
}

#[test]
fn c12_non_eighth_frames_skip_the_direction_draw_but_keep_particle_draws() {
    let mut state = victory_state(12, 9, 0);
    let mut values = Vec::new();
    for pair in 0..5 {
        values.push(pair);
        values.push(pair);
    }
    let mut rng = ScriptedRng::with_values(&values);
    let result = state
        .update_victory_animation(1, victory_context(), &mut rng)
        .unwrap();
    assert_eq!(rng.cursor, 10);
    assert_eq!(state.effects.active_count(), 5);
    assert_eq!(result.spawned_effect_slots.len(), 5);
    assert_eq!(
        result
            .draws
            .iter()
            .map(|draw| draw.range)
            .collect::<Vec<_>>(),
        vec![40, 10, 40, 10, 40, 10, 40, 10, 40, 10]
    );
}

#[test]
fn c12_random_zero_reverses_the_post_clamp_facing() {
    let mut state = victory_state(12, 8, 0);
    state.fighters[1].x = 289.0;
    let mut values = vec![0];
    for _ in 0..5 {
        values.extend([0, 0]);
    }
    let mut rng = ScriptedRng::with_values(&values);
    state
        .update_victory_animation(1, victory_context(), &mut rng)
        .unwrap();
    // Movement crosses the right threshold (sets LEFT), then Random(2)=0 flips it.
    assert_eq!(state.fighters[1].facing, SIDE_RIGHT);
    assert_eq!(rng.cursor, 11);
}

#[test]
fn c08_and_c13_apply_ground_relative_vertical_motion() {
    let mut c08 = victory_state(8, 0, 0);
    c08.fighters[1].y = 299.0;
    let mut rng = ScriptedRng::default();
    c08.update_victory_animation(1, victory_context(), &mut rng)
        .unwrap();
    assert_eq!(c08.fighters[1].y, 295.0);

    let mut c13 = victory_state(13, 0, 0);
    c13.fighters[1].y = 299.0;
    c13.update_victory_animation(1, victory_context(), &mut rng)
        .unwrap();
    assert_eq!(c13.fighters[1].y, 290.0);
}

#[test]
fn c15_frame_ten_keeps_the_later_independent_assignment() {
    let mut state = victory_state(15, 10, 0);
    let mut rng = ScriptedRng::default();
    state
        .update_victory_animation(1, victory_context(), &mut rng)
        .unwrap();
    assert_eq!(state.fighters[1].sprite_frame, 1034);
}

#[test]
fn victory_command_matches_the_shared_pascal_contract_for_all_original_fighters() {
    // C01-C15 each define the same [Acao] victory guard and state writes.
    for character_id in 1..=15 {
        let (mut state, _) = ai_scenario(PASCAL_TICK_316, 316);
        let fighter = &mut state.fighters[1];
        fighter.character_id = character_id;
        fighter.action_id = match character_id % 3 {
            0 => ACTION_IDLE,
            1 => ACTION_WALKING,
            _ => ACTION_CROUCHING,
        };
        fighter.action_frame = 9;
        fighter.action_duration = 17;

        assert!(
            state.apply_character_victory(1),
            "C{character_id:02} should accept victory from idle/walking/crouching"
        );
        assert_eq!(state.fighters[1].action_id, ACTION_VICTORY);
        assert_eq!(state.fighters[1].action_frame, 0);
        assert_eq!(state.fighters[1].action_duration, 250);
        assert_eq!(state.fighters[1].character_id, character_id);
    }
}

#[test]
fn victory_command_rejects_actions_outside_the_pascal_eligibility_set() {
    for action_id in [3, 4, 5, 6, 8, 12, 21, 22, 23] {
        let (mut state, _) = ai_scenario(PASCAL_TICK_316, 316);
        let fighter = &mut state.fighters[1];
        fighter.action_id = action_id;
        fighter.action_frame = 7;
        fighter.action_duration = 31;
        let before = fighter.clone();

        assert!(!state.apply_character_victory(1));
        assert_eq!(state.fighters[1], before);
    }
}

#[test]
fn victory_command_rejects_an_invalid_fighter_index_without_mutation() {
    let (mut state, _) = ai_scenario(PASCAL_TICK_316, 316);
    let before = state.clone();

    assert!(!state.apply_character_victory(2));
    assert_eq!(state, before);
}

#[test]
fn pose_command_matches_pascal_contract_for_fighters_that_define_it() {
    // C01-C12 all accept idle/walking and write acPose, frame 0, duration 20.
    for character_id in 1..=12 {
        for action_id in [ACTION_IDLE, ACTION_WALKING] {
            let (mut state, _) = ai_scenario(PASCAL_TICK_316, 316);
            let fighter = &mut state.fighters[1];
            fighter.character_id = character_id;
            fighter.action_id = action_id;
            fighter.action_frame = 9;
            fighter.action_duration = 17;

            assert!(
                state.apply_character_pose(1),
                "C{character_id:02} should accept pose"
            );
            assert_eq!(state.fighters[1].action_id, ACTION_POSE);
            assert_eq!(state.fighters[1].action_frame, 0);
            assert_eq!(state.fighters[1].action_duration, 20);
        }
    }
}

#[test]
fn pose_command_rejects_fighters_without_cm_pose_and_ineligible_actions() {
    for character_id in 13..=15 {
        let (mut state, _) = ai_scenario(PASCAL_TICK_316, 316);
        state.fighters[1].character_id = character_id;
        state.fighters[1].action_id = ACTION_IDLE;
        let before = state.fighters[1].clone();

        assert!(!state.apply_character_pose(1));
        assert_eq!(state.fighters[1], before);
    }

    for action_id in [ACTION_CROUCHING, ACTION_VICTORY, 3, 4, 5, 6, 8, 12, 21, 22] {
        let (mut state, _) = ai_scenario(PASCAL_TICK_316, 316);
        state.fighters[1].character_id = 1;
        state.fighters[1].action_id = action_id;
        let before = state.fighters[1].clone();

        assert!(!state.apply_character_pose(1));
        assert_eq!(state.fighters[1], before);
    }
}

#[test]
fn pose_command_rejects_an_invalid_fighter_index_without_mutation() {
    let (mut state, _) = ai_scenario(PASCAL_TICK_316, 316);
    let before = state.clone();

    assert!(!state.apply_character_pose(2));
    assert_eq!(state, before);
}

#[test]
fn pose_animation_writes_audited_frames_for_c01_through_c12() {
    for character_id in 1..=12 {
        let mut at_start = victory_state(character_id, 0, 777);
        at_start.fighters[1].action_id = ACTION_POSE;
        assert!(at_start.update_pose_animation(1, 1000));
        assert_eq!(
            at_start.fighters[1].sprite_frame, 1012,
            "C{character_id:02} frame 0"
        );

        let mut at_second_boundary = victory_state(character_id, 2, 777);
        at_second_boundary.fighters[1].action_id = ACTION_POSE;
        assert!(at_second_boundary.update_pose_animation(1, 1000));
        assert_eq!(
            at_second_boundary.fighters[1].sprite_frame, 1030,
            "C{character_id:02} frame 2"
        );

        let mut between_boundaries = victory_state(character_id, 1, 777);
        between_boundaries.fighters[1].action_id = ACTION_POSE;
        assert!(between_boundaries.update_pose_animation(1, 1000));
        assert_eq!(
            between_boundaries.fighters[1].sprite_frame, 777,
            "C{character_id:02} frame 1 retains P.Quadro"
        );
    }
}

#[test]
fn pose_animation_rejects_non_pose_actions_unsupported_fighters_and_bad_indices() {
    let mut wrong_action = victory_state(1, 0, 777);
    let before = wrong_action.fighters[1].clone();
    assert!(!wrong_action.update_pose_animation(1, 1000));
    assert_eq!(wrong_action.fighters[1], before);

    for character_id in 13..=15 {
        let mut unsupported = victory_state(character_id, 0, 777);
        unsupported.fighters[1].action_id = ACTION_POSE;
        let before = unsupported.fighters[1].clone();
        assert!(!unsupported.update_pose_animation(1, 1000));
        assert_eq!(unsupported.fighters[1], before);
    }

    let mut invalid_index = victory_state(1, 0, 777);
    let before = invalid_index.clone();
    assert!(!invalid_index.update_pose_animation(2, 1000));
    assert_eq!(invalid_index, before);
}

#[test]
fn c01_special1_frame10_spawns_a_real_type7_and_consumes_random_five() {
    let mut state = c01_frame_state(ACTION_SPECIAL1, 10, 0);
    let mut rng = ScriptedRng::with_values(&[0]);

    let result = state
        .update_c01_character_frame(1, victory_context(), &mut rng)
        .unwrap();

    assert_eq!(state.fighters[1].sprite_frame, 1039);
    assert_eq!(state.pending_sound, Some(14));
    assert_eq!(rng.cursor, 1);
    assert_eq!(
        result
            .draws
            .iter()
            .map(|draw| (draw.range, draw.value))
            .collect::<Vec<_>>(),
        vec![(5, 0)]
    );
    assert_eq!(result.spawned_effect_slots, vec![0]);

    let effect = state.effects.instance(0).expect("Esp1 live effect");
    assert_eq!(effect.effect_type, 7);
    assert_eq!(effect.source_variant, Some('A'));
    assert_eq!(effect.owner, 2);
    assert_eq!(effect.facing, SIDE_RIGHT);
    assert_eq!(effect.x, 245.0);
    assert_eq!(effect.y, 290.0);
    assert_eq!(effect.horizontal_velocity, 8);
    assert!(effect.variant_flag);
}

#[test]
fn c01_special2_uses_sprite_frame_gate_and_spawns_dust_and_trail_instances() {
    let mut state = c01_frame_state(ACTION_SPECIAL2, 9, 1041);
    let values = [1, 2, 3, 4, 5, 6];
    let mut rng = ScriptedRng::with_values(&values);

    let result = state
        .update_c01_character_frame(1, victory_context(), &mut rng)
        .unwrap();

    assert_eq!(rng.cursor, 6);
    assert_eq!(
        result
            .draws
            .iter()
            .map(|draw| draw.range)
            .collect::<Vec<_>>(),
        vec![40, 10, 40, 10, 40, 10]
    );
    assert_eq!(result.spawned_effect_slots, vec![0, 1, 2, 3]);
    assert_eq!(state.effects.active_count(), 4);
    assert_eq!(state.fighters[1].x, 219.0);
    assert!(state.fighters[1].is_kick);
    for slot in 0..3 {
        let effect = state.effects.instance(slot).unwrap();
        assert_eq!(effect.effect_type, 2);
        assert_eq!(effect.owner, 0);
    }
    let trail = state.effects.instance(3).expect("Esp2 trail instance");
    assert_eq!(trail.effect_type, 16);
    assert_eq!(trail.owner, 2);
    assert_eq!(trail.facing, SIDE_RIGHT);
    assert_eq!(trail.x, 279.0);
    assert_eq!(trail.y, 205.0);
}

#[test]
fn c01_special3_applies_invulnerability_in_source_order_and_moves_each_frame() {
    let mut state = c01_frame_state(ACTION_SPECIAL3, 9, 1043);
    state.fighters[1].hit_count = 4;
    state.fighters[0].invulnerable = false;
    let mut rng = ScriptedRng::default();

    state
        .update_c01_character_frame(1, victory_context(), &mut rng)
        .unwrap();

    assert!(state.fighters[0].invulnerable);
    assert_eq!(state.fighters[1].sprite_frame, 1044);
    assert_eq!(state.fighters[1].y, 280.0);
    assert_eq!(state.fighters[1].x, 205.0);
    assert!(state.fighters[1].is_kick);
    assert_eq!(state.pending_sound, None);
    assert_eq!(rng.cursor, 0);

    let mut early = c01_frame_state(ACTION_SPECIAL3, 0, 0);
    early.fighters[1].hit_count = 1;
    early.fighters[0].invulnerable = true;
    early
        .update_c01_character_frame(1, victory_context(), &mut rng)
        .unwrap();
    assert!(!early.fighters[0].invulnerable);
    assert_eq!(early.pending_sound, Some(16));
}

#[test]
fn c01_special3_sets_airborne_at_action_frame_twenty() {
    let mut state = c01_frame_state(ACTION_SPECIAL3, 20, 0);
    let mut rng = ScriptedRng::default();
    state
        .update_c01_character_frame(1, victory_context(), &mut rng)
        .unwrap();
    assert!(state.fighters[1].in_air);
}

#[test]
fn c01_special4_applies_weak_and_strong_frame_pushback_windows() {
    let mut weak = c01_frame_state(ACTION_SPECIAL4, 5, 777);
    weak.fighters[1].force_strength = FORCE_WEAK;
    let mut rng = ScriptedRng::default();
    weak.update_c01_character_frame(1, victory_context(), &mut rng)
        .unwrap();
    assert_eq!(weak.fighters[1].x, 217.0);
    assert_eq!(weak.fighters[1].sprite_frame, 777);
    assert!(!weak.fighters[1].is_kick);

    let mut strong = c01_frame_state(ACTION_SPECIAL4, 13, 777);
    strong.fighters[1].force_strength = FORCE_STRONG;
    strong
        .update_c01_character_frame(1, victory_context(), &mut rng)
        .unwrap();
    assert_eq!(strong.fighters[1].x, 221.0);
    assert!(strong.fighters[1].is_kick);
}

#[test]
fn c01_final_special_frame9_spawns_type8_and_requests_shake_and_sound() {
    let mut state = c01_frame_state(ACTION_SPECIAL_FINAL, 9, 0);
    state.fighters[1].facing = SIDE_LEFT;
    let mut rng = ScriptedRng::default();

    let result = state
        .update_c01_character_frame(1, victory_context(), &mut rng)
        .unwrap();

    assert_eq!(state.fighters[1].sprite_frame, 1039);
    assert_eq!(state.pending_sound, Some(18));
    assert_eq!(state.screen_shake, 20);
    assert_eq!(result.spawned_effect_slots, vec![0]);
    let effect = state.effects.instance(0).unwrap();
    assert_eq!(effect.effect_type, 8);
    assert_eq!(effect.owner, 2);
    assert_eq!(effect.facing, SIDE_LEFT);
    assert_eq!(effect.x, 90.0);
    assert_eq!(effect.damage, 40);
    assert_eq!(effect.guard_damage, 3);
    assert!(effect.can_hit);
    assert!(effect.terminating);
}

#[test]
fn c01_strong_special2_hit_reaction_sets_twenty_recoil() {
    let mut state = c01_frame_state(ACTION_SPECIAL2, 9, 1041);
    state.fighters[1].force_strength = FORCE_STRONG;
    assert!(state.apply_c01_hit_reaction(1));
    assert_eq!(state.fighters[0].recoil, 20);

    state.fighters[1].force_strength = FORCE_WEAK;
    assert!(!state.apply_c01_hit_reaction(1));
    assert_eq!(state.fighters[0].recoil, 20);
}
