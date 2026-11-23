//! Source-derived C01 special-entry and character-AI dispatcher contracts.

use street_chaves::chaves_specials::{
    ChavesSpecialCommand as Command, ChavesSpecialContext, ChavesSpecialError,
    ChavesSpecialEvent as Event, ChavesSpecialRejection as Rejection, CHARACTER_CHAVES,
    SOUND_FINAL_SPECIAL,
};
use street_chaves::combat_runtime::{
    AiCommand, AiTickInputs, BodyBox, CombatState, FighterState, RuntimeDrawWidth,
    ACTION_CROUCHING, ACTION_DEFENDING, ACTION_IDLE, ACTION_JUMPING, ACTION_LOW_DEFENDING,
    ACTION_SPECIAL1, ACTION_SPECIAL2, ACTION_SPECIAL3, ACTION_SPECIAL4, ACTION_SPECIAL_FINAL,
    ACTION_STRONG_PUNCH, ACTION_WALKING, FORCE_SPECIAL, FORCE_STRONG, FORCE_WEAK, SIDE_RIGHT,
};
use street_chaves::effect_runtime::EFFECT_SIDE_RIGHT;
use street_chaves::pascal_replay::PascalRandomSource;

#[derive(Default)]
struct ScriptedRng {
    values: Vec<i32>,
    cursor: usize,
    requests: Vec<i32>,
}

impl ScriptedRng {
    fn with_values(values: &[i32]) -> Self {
        Self {
            values: values.to_vec(),
            cursor: 0,
            requests: Vec::new(),
        }
    }
}

impl PascalRandomSource for ScriptedRng {
    type Error = String;

    fn draw_longint(&mut self, range: i32) -> Result<i32, Self::Error> {
        self.requests.push(range);
        let value = *self
            .values
            .get(self.cursor)
            .ok_or_else(|| format!("missing scripted RNG value at {}", self.cursor))?;
        self.cursor += 1;
        if value < 0 || value >= range {
            return Err(format!("scripted value {value} outside [0, {range})"));
        }
        Ok(value)
    }

    fn draw_int64(&mut self, range: i64) -> Result<i64, Self::Error> {
        self.draw_longint(range as i32).map(i64::from)
    }
}

fn fighter(character_id: u8, action_id: u16) -> FighterState {
    FighterState {
        sprite_frame: 7,
        character_id,
        action_id,
        action_frame: 5,
        action_duration: 99,
        facing: SIDE_RIGHT,
        force_strength: 0,
        defending: true,
        invulnerable: false,
        hit_count: 0,
        is_kick: false,
        recoil: 0,
        vertical_velocity: -4,
        in_air: true,
        x: 100.0,
        y: 240.0,
        speed: 8.0,
        distance_ai: 100,
        health: 100.0,
        power: 2,
        charge_power: 10.0,
        jump_direction: 3,
        low_time: 8,
        sequence_state: 4,
        body: BodyBox {
            x1: -18.0,
            y1: 30.0,
            x2: 13.0,
            y2: 170.0,
        },
    }
}

fn state_with_action(action_id: u16) -> CombatState {
    CombatState {
        tick_index: 0,
        game_state: 7,
        camera_x: 0.0,
        clock_stopped: false,
        fighters: [
            fighter(2, ACTION_IDLE),
            fighter(CHARACTER_CHAVES, action_id),
        ],
        effects: Default::default(),
        pending_sound: None,
        screen_flash: 0,
        screen_shake: 0,
    }
}

fn context() -> ChavesSpecialContext {
    ChavesSpecialContext {
        charge_gain: 2.0,
        charge_max: 102.0,
        ground_y: 300.0,
    }
}

fn ai_input(distance: i32, threat: i32, projectile_distance: i32, opponent_y: i32) -> AiTickInputs {
    AiTickInputs {
        distance,
        projectile_distance,
        opponent_y,
        threat,
        difficulty_factor: 1.0,
    }
}

#[test]
fn command_codes_match_the_pascal_special_ids() {
    let expected = [
        (Command::Esp1Weak, 16),
        (Command::Esp1Strong, 17),
        (Command::Esp2Weak, 18),
        (Command::Esp2Strong, 19),
        (Command::Esp3Weak, 20),
        (Command::Esp3Strong, 21),
        (Command::Esp4Weak, 22),
        (Command::Esp4Strong, 23),
        (Command::Final, 24),
    ];
    for (command, code) in expected {
        assert_eq!(command.pascal_code(), code);
        assert_eq!(Command::from_pascal_code(code), Some(command));
    }
    assert_eq!(Command::from_pascal_code(25), None);
}

#[test]
fn special_one_queries_the_live_effect_arena_by_type_and_owner() {
    let mut state = state_with_action(ACTION_DEFENDING);
    let mut rng = ScriptedRng::with_values(&[0]);
    assert_eq!(
        state
            .effects
            .spawn(7, Some('A'), 2, EFFECT_SIDE_RIGHT, 145.0, 240.0, &mut rng)
            .unwrap(),
        Some(0)
    );

    let result = state
        .dispatch_chaves_special(1, Command::Esp1Weak, context())
        .unwrap();

    assert!(!result.applied);
    assert_eq!(result.rejection, Some(Rejection::Effect7AlreadyExists));
    assert_eq!(state.fighters[1].action_id, ACTION_DEFENDING);
    assert!(
        !state.fighters[1].defending,
        "Common [Acao] clears this first"
    );
    assert_eq!(result.events, vec![Event::CommonDefendingCleared]);
}

#[test]
fn rejected_special_runs_common_action_end_when_fighter_keeps_walking() {
    let mut state = state_with_action(ACTION_WALKING);
    state.fighters[1].action_frame = 16;

    // A live type-7 owned by player 2 rejects Special 1 after Common [Acao].
    let mut effect_rng = ScriptedRng::with_values(&[0]);
    state
        .effects
        .spawn(
            7,
            Some('A'),
            2,
            EFFECT_SIDE_RIGHT,
            145.0,
            240.0,
            &mut effect_rng,
        )
        .unwrap();

    let result = state
        .dispatch_chaves_special(1, Command::Esp1Weak, context())
        .unwrap();

    assert!(!result.applied);
    assert_eq!(result.rejection, Some(Rejection::Effect7AlreadyExists));
    assert_eq!(state.fighters[1].action_id, ACTION_WALKING);
    assert_eq!(state.fighters[1].action_frame, 0, "frame 17 wraps to zero");
    assert_eq!(
        result.events.last(),
        Some(&Event::CommonWalkingFrameAdvanced { value: 0 })
    );
}

#[test]
fn special_one_starts_when_the_live_effect_slot_is_empty_and_records_source_order() {
    let mut state = state_with_action(ACTION_DEFENDING);
    let result = state
        .dispatch_chaves_special(1, Command::Esp1Weak, context())
        .unwrap();

    assert!(result.applied);
    assert_eq!(state.fighters[1].action_id, ACTION_SPECIAL1);
    assert_eq!(state.fighters[1].action_frame, 0);
    assert_eq!(state.fighters[1].action_duration, 28);
    assert_eq!(state.fighters[1].charge_power, 20.0);
    assert_eq!(state.fighters[1].force_strength, FORCE_WEAK);
    assert_eq!(
        result.events,
        vec![
            Event::CommonDefendingCleared,
            Event::ActionChanged {
                value: ACTION_SPECIAL1
            },
            Event::ActionFrameChanged { value: 0 },
            Event::ActionDurationChanged { value: 28 },
            Event::ChargePowerChanged { value: 20.0 },
            Event::ForceStrengthChanged { value: FORCE_WEAK },
        ]
    );
}

#[test]
fn special_two_preserves_strength_durations_and_does_not_clamp_charge() {
    let mut weak = state_with_action(ACTION_LOW_DEFENDING);
    weak.fighters[1].charge_power = 101.0;
    let result = weak
        .dispatch_chaves_special(1, Command::Esp2Weak, context())
        .unwrap();
    assert!(result.applied);
    assert_eq!(weak.fighters[1].action_id, ACTION_SPECIAL2);
    assert_eq!(weak.fighters[1].action_duration, 16);
    assert_eq!(weak.fighters[1].force_strength, FORCE_WEAK);
    assert_eq!(weak.fighters[1].charge_power, 111.0);

    let mut strong = state_with_action(ACTION_IDLE);
    strong.fighters[1].charge_power = 102.0;
    let result = strong
        .dispatch_chaves_special(1, Command::Esp2Strong, context())
        .unwrap();
    assert!(result.applied);
    assert_eq!(strong.fighters[1].action_duration, 27);
    assert_eq!(strong.fighters[1].force_strength, FORCE_STRONG);
    assert_eq!(strong.fighters[1].charge_power, 102.0);
}

#[test]
fn special_three_requires_ground_proximity_and_resets_jump_state() {
    let mut too_high = state_with_action(ACTION_JUMPING);
    too_high.fighters[1].y = 209.99;
    let blocked = too_high
        .dispatch_chaves_special(1, Command::Esp3Strong, context())
        .unwrap();
    assert!(!blocked.applied);
    assert_eq!(blocked.rejection, Some(Rejection::TooFarAboveGround));

    let mut accepted = state_with_action(ACTION_JUMPING);
    accepted.fighters[1].y = 210.0;
    let result = accepted
        .dispatch_chaves_special(1, Command::Esp3Strong, context())
        .unwrap();
    assert!(result.applied);
    let fighter = &accepted.fighters[1];
    assert_eq!(fighter.action_id, ACTION_SPECIAL3);
    assert_eq!(fighter.action_frame, 0);
    assert_eq!(fighter.action_duration, 16);
    assert_eq!(fighter.charge_power, 22.0);
    assert_eq!(fighter.y, 300.0);
    assert!(!fighter.in_air);
    assert_eq!(fighter.vertical_velocity, 0);
    assert_eq!(fighter.jump_direction, 0);
    assert_eq!(fighter.low_time, 0);
    assert_eq!(fighter.sequence_state, 0);
    assert_eq!(fighter.force_strength, FORCE_STRONG);
    assert_eq!(result.events.first(), Some(&Event::CommonDefendingCleared));
}

#[test]
fn final_special_preserves_word_wrap_flash_sound_and_player_offsets() {
    let mut final_move = state_with_action(ACTION_STRONG_PUNCH);
    final_move.fighters[1].power = 3;
    final_move.fighters[1].charge_power = 44.0;
    let result = final_move
        .dispatch_chaves_special(1, Command::Final, context())
        .unwrap();

    assert!(result.applied);
    assert_eq!(final_move.fighters[1].action_id, ACTION_SPECIAL_FINAL);
    assert_eq!(final_move.fighters[1].action_duration, 38);
    assert_eq!(final_move.fighters[1].power, 2);
    assert_eq!(final_move.fighters[1].charge_power, 0.0);
    assert_eq!(final_move.fighters[1].sprite_frame, 86);
    assert_eq!(final_move.fighters[1].force_strength, FORCE_SPECIAL);
    assert_eq!(final_move.screen_flash, 22);
    assert_eq!(final_move.screen_shake, 0);
    assert_eq!(final_move.pending_sound, Some(SOUND_FINAL_SPECIAL));

    let mut low_health = state_with_action(ACTION_CROUCHING);
    low_health.fighters[1].health = 15.0;
    low_health.fighters[1].power = 0;
    assert!(
        low_health
            .dispatch_chaves_special(1, Command::Final, context())
            .unwrap()
            .applied
    );
    assert_eq!(low_health.fighters[1].power, u16::MAX);

    let mut player_one = state_with_action(ACTION_IDLE);
    player_one.fighters[0] = fighter(CHARACTER_CHAVES, ACTION_IDLE);
    assert!(
        player_one
            .dispatch_chaves_special(0, Command::Final, context())
            .unwrap()
            .applied
    );
    assert_eq!(player_one.fighters[0].sprite_frame, 36);
}

#[test]
fn numeric_dispatcher_rejects_unknown_codes_without_side_effects() {
    let mut state = state_with_action(ACTION_IDLE);
    let before = state.clone();
    assert_eq!(
        state.dispatch_chaves_special_code(1, 25, context()),
        Err(ChavesSpecialError::UnknownCommandCode(25))
    );
    assert_eq!(state, before);
    assert_eq!(
        state.dispatch_chaves_special_code(2, 16, context()),
        Err(ChavesSpecialError::FighterIndexOutOfRange(2))
    );
}

#[test]
fn c01_ai_specials_keep_independent_if_order_and_share_the_common_rng_cursor() {
    let mut state = state_with_action(ACTION_IDLE);
    state.fighters[1].defending = false;
    state.fighters[1].y = 220.0;
    state.fighters[1].distance_ai = 100;
    state.fighters[1].facing = SIDE_RIGHT;
    let mut rng = ScriptedRng::with_values(&[1, 0, 0, 0, 1, 0, 0, 0]);

    let result = state
        .tick_common_ai_with_chaves(ai_input(100, 0, 0, 200), context(), &mut rng)
        .unwrap();

    assert_eq!(
        result.commands,
        vec![
            AiCommand::WalkRight,
            AiCommand::ChavesSpecial(Command::Esp2Weak),
            AiCommand::ChavesSpecial(Command::Esp3Strong),
            AiCommand::ChavesSpecial(Command::Esp4Weak),
            AiCommand::ChavesSpecial(Command::Final),
        ]
    );
    assert_eq!(
        result
            .draws
            .iter()
            .map(|draw| (draw.width, draw.range, draw.value))
            .collect::<Vec<_>>(),
        vec![
            (RuntimeDrawWidth::LongInt, 13, 1),
            (RuntimeDrawWidth::LongInt, 40, 0),
            (RuntimeDrawWidth::LongInt, 2, 0),
            (RuntimeDrawWidth::LongInt, 50, 0),
            (RuntimeDrawWidth::LongInt, 2, 1),
            (RuntimeDrawWidth::LongInt, 40, 0),
            (RuntimeDrawWidth::LongInt, 2, 0),
            (RuntimeDrawWidth::LongInt, 50, 0),
        ]
    );
    assert_eq!(rng.cursor, 8);
    // Both first requests are accepted; later independent requests are gated by
    // the resulting action state but still consume their own source RNG calls.
    assert_eq!(state.fighters[1].action_id, ACTION_SPECIAL3);
    assert_eq!(state.fighters[1].charge_power, 32.0);
}

#[test]
fn c01_ai_projectile_and_long_distance_special_one_paths_are_distinct() {
    let mut projectile = state_with_action(ACTION_IDLE);
    projectile.fighters[1].defending = false;
    projectile.fighters[1].distance_ai = 300;
    let mut projectile_rng = ScriptedRng::with_values(&[1, 0, 1]);
    let projectile_tick = projectile
        .tick_common_ai_with_chaves(ai_input(220, 1, 200, 300), context(), &mut projectile_rng)
        .unwrap();

    assert!(projectile_tick
        .commands
        .contains(&AiCommand::ChavesSpecial(Command::Esp1Strong)));
    assert_eq!(
        projectile_tick
            .draws
            .iter()
            .map(|draw| draw.range)
            .collect::<Vec<_>>(),
        vec![13, 7, 2]
    );

    let mut long_range = state_with_action(ACTION_IDLE);
    long_range.fighters[1].defending = false;
    long_range.fighters[1].distance_ai = 300;
    let mut long_range_rng = ScriptedRng::with_values(&[1, 0, 0]);
    let long_range_tick = long_range
        .tick_common_ai_with_chaves(ai_input(220, 0, 0, 300), context(), &mut long_range_rng)
        .unwrap();

    assert!(long_range_tick
        .commands
        .contains(&AiCommand::ChavesSpecial(Command::Esp1Weak)));
    assert_eq!(
        long_range_tick
            .draws
            .iter()
            .map(|draw| draw.range)
            .collect::<Vec<_>>(),
        vec![13, 30, 2]
    );
}

#[test]
fn c01_ai_respects_its_defending_guard_and_character_id() {
    let mut defending = state_with_action(ACTION_IDLE);
    defending.fighters[1].defending = true;
    let mut defending_rng = ScriptedRng::with_values(&[1]);
    let defending_tick = defending
        .tick_common_ai_with_chaves(ai_input(100, 0, 500, 100), context(), &mut defending_rng)
        .unwrap();
    assert!(defending_tick.commands.is_empty());
    assert_eq!(defending_tick.draws.len(), 1);

    let mut other = state_with_action(ACTION_IDLE);
    other.fighters[1].character_id = 2;
    other.fighters[1].defending = true;
    let mut other_rng = ScriptedRng::with_values(&[1]);
    let other_tick = other
        .tick_common_ai_with_chaves(ai_input(100, 0, 500, 100), context(), &mut other_rng)
        .unwrap();
    assert!(other_tick.commands.is_empty());
    assert_eq!(other_rng.cursor, 1);
}
