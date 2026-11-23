use street_chaves::chaves_specials::{
    ChavesSpecialCommand as Command, ChavesSpecialContext, ChavesSpecialError,
    ChavesSpecialEvent as Event, ChavesSpecialRejection as Rejection, ACTION_SPECIAL_1,
    ACTION_SPECIAL_2, ACTION_SPECIAL_3, ACTION_SPECIAL_4, ACTION_SPECIAL_5, CHARACTER_CHAVES,
    SOUND_FINAL_SPECIAL, STRENGTH_SPECIAL, STRENGTH_STRONG, STRENGTH_WEAK,
};
use street_chaves::combat_runtime::{
    BodyBox, CombatState, FighterState, ACTION_CROUCHING, ACTION_DEFENDING, ACTION_IDLE,
    ACTION_JUMPING, ACTION_LOW_DEFENDING, ACTION_STRONG_PUNCH, SIDE_RIGHT,
};

fn fighter(character_id: u8, action_id: u16) -> FighterState {
    FighterState {
        character_id,
        action_id,
        action_frame: 5,
        action_duration: 99,
        facing: SIDE_RIGHT,
        defending: false,
        vertical_velocity: -4,
        in_air: true,
        x: 100.0,
        y: 240.0,
        speed: 8.0,
        distance_ai: 100,
        health: 100.0,
        sprite_frame: 7,
        power: 2,
        charge_power: 10.0,
        attack_strength: 0,
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

fn state_with_player_action(action_id: u16) -> CombatState {
    CombatState {
        tick_index: 0,
        game_state: 7,
        camera_x: 0.0,
        clock_stopped: false,
        screen_flash: 0,
        pending_sound: 0,
        fighters: [fighter(CHARACTER_CHAVES, action_id), fighter(2, ACTION_IDLE)],
    }
}

fn context(effect_7_exists: bool) -> ChavesSpecialContext {
    ChavesSpecialContext {
        charge_gain: 2.0,
        charge_max: 102.0,
        ground_y: 300,
        effect_7_exists,
    }
}

#[test]
fn command_codes_match_the_pascal_constants() {
    assert_eq!(Command::Esp1Weak.pascal_code(), 16);
    assert_eq!(Command::Esp1Strong.pascal_code(), 17);
    assert_eq!(Command::Esp2Weak.pascal_code(), 18);
    assert_eq!(Command::Esp2Strong.pascal_code(), 19);
    assert_eq!(Command::Esp3Weak.pascal_code(), 20);
    assert_eq!(Command::Esp3Strong.pascal_code(), 21);
    assert_eq!(Command::Esp4Weak.pascal_code(), 22);
    assert_eq!(Command::Esp4Strong.pascal_code(), 23);
    assert_eq!(Command::Final.pascal_code(), 24);
}

#[test]
fn special_one_checks_the_real_effect_query_and_preserves_state_when_blocked() {
    let mut state = state_with_player_action(ACTION_DEFENDING);
    let before = state.fighters[0].clone();

    let blocked = state
        .start_chaves_special(0, Command::Esp1Weak, context(true))
        .unwrap();

    assert!(!blocked.applied);
    assert_eq!(blocked.rejection, Some(Rejection::Effect7AlreadyExists));
    assert!(blocked.events.is_empty());
    assert_eq!(state.fighters[0], before);

    let started = state
        .start_chaves_special(0, Command::Esp1Weak, context(false))
        .unwrap();

    assert!(started.applied);
    assert_eq!(started.rejection, None);
    assert_eq!(state.fighters[0].action_id, ACTION_SPECIAL_1);
    assert_eq!(state.fighters[0].action_frame, 0);
    assert_eq!(state.fighters[0].action_duration, 28);
    assert_eq!(state.fighters[0].charge_power, 20.0);
    assert_eq!(state.fighters[0].attack_strength, STRENGTH_WEAK);
    assert_eq!(
        started.events,
        vec![
            Event::ActionChanged {
                value: ACTION_SPECIAL_1
            },
            Event::ActionFrameChanged { value: 0 },
            Event::ActionDurationChanged { value: 28 },
            Event::ChargePowerChanged { value: 20.0 },
            Event::AttackStrengthChanged {
                value: STRENGTH_WEAK
            },
        ]
    );
}

#[test]
fn special_two_uses_strength_specific_durations_and_does_not_clamp_charge() {
    let mut weak = state_with_player_action(ACTION_LOW_DEFENDING);
    weak.fighters[0].charge_power = 101.0;

    let result = weak
        .start_chaves_special(0, Command::Esp2Weak, context(false))
        .unwrap();

    assert!(result.applied);
    assert_eq!(weak.fighters[0].action_id, ACTION_SPECIAL_2);
    assert_eq!(weak.fighters[0].action_duration, 16);
    assert_eq!(weak.fighters[0].attack_strength, STRENGTH_WEAK);
    // Pascal tests the pre-increment value; it does not clamp the result to CargaMax.
    assert_eq!(weak.fighters[0].charge_power, 111.0);

    let mut strong = state_with_player_action(ACTION_IDLE);
    strong.fighters[0].charge_power = 102.0;
    let result = strong
        .start_chaves_special(0, Command::Esp2Strong, context(false))
        .unwrap();

    assert!(result.applied);
    assert_eq!(strong.fighters[0].action_duration, 27);
    assert_eq!(strong.fighters[0].attack_strength, STRENGTH_STRONG);
    assert_eq!(strong.fighters[0].charge_power, 102.0);
    assert_eq!(
        result.events,
        vec![
            Event::ActionChanged {
                value: ACTION_SPECIAL_2
            },
            Event::ActionFrameChanged { value: 0 },
            Event::ActionDurationChanged { value: 27 },
            Event::AttackStrengthChanged {
                value: STRENGTH_STRONG
            },
        ]
    );
}

#[test]
fn special_three_requires_ground_proximity_and_resets_jump_state_in_source_order() {
    let mut too_high = state_with_player_action(ACTION_JUMPING);
    too_high.fighters[0].y = 209.99;
    let before = too_high.fighters[0].clone();

    let blocked = too_high
        .start_chaves_special(0, Command::Esp3Strong, context(false))
        .unwrap();
    assert!(!blocked.applied);
    assert_eq!(blocked.rejection, Some(Rejection::TooFarAboveGround));
    assert_eq!(too_high.fighters[0], before);

    let mut state = state_with_player_action(ACTION_JUMPING);
    state.fighters[0].y = 210.0;
    let result = state
        .start_chaves_special(0, Command::Esp3Strong, context(false))
        .unwrap();

    assert!(result.applied);
    let fighter = &state.fighters[0];
    assert_eq!(fighter.action_id, ACTION_SPECIAL_3);
    assert_eq!(fighter.action_frame, 0);
    assert_eq!(fighter.action_duration, 16);
    assert_eq!(fighter.charge_power, 22.0);
    assert_eq!(fighter.y, 300.0);
    assert!(!fighter.in_air);
    assert_eq!(fighter.vertical_velocity, 0);
    assert_eq!(fighter.jump_direction, 0);
    assert_eq!(fighter.low_time, 0);
    assert_eq!(fighter.sequence_state, 0);
    assert_eq!(fighter.attack_strength, STRENGTH_STRONG);
    assert_eq!(
        result.events,
        vec![
            Event::ActionChanged {
                value: ACTION_SPECIAL_3
            },
            Event::ChargePowerChanged { value: 22.0 },
            Event::PositionYChanged { value: 300.0 },
            Event::InAirChanged { value: false },
            Event::ActionFrameChanged { value: 0 },
            Event::VerticalVelocityChanged { value: 0 },
            Event::JumpDirectionChanged { value: 0 },
            Event::LowTimeChanged { value: 0 },
            Event::SequenceStateChanged { value: 0 },
            Event::ActionDurationChanged { value: 16 },
            Event::AttackStrengthChanged {
                value: STRENGTH_STRONG
            },
        ]
    );
}

#[test]
fn special_three_is_also_allowed_from_the_second_special() {
    let mut state = state_with_player_action(ACTION_SPECIAL_2);
    state.fighters[0].y = 220.0;

    let result = state
        .start_chaves_special(0, Command::Esp3Weak, context(false))
        .unwrap();

    assert!(result.applied);
    assert_eq!(state.fighters[0].action_id, ACTION_SPECIAL_3);
    assert_eq!(state.fighters[0].action_duration, 10);
    assert_eq!(state.fighters[0].attack_strength, STRENGTH_WEAK);
}

#[test]
fn special_four_is_only_accepted_from_its_four_source_states() {
    let mut state = state_with_player_action(ACTION_LOW_DEFENDING);

    let result = state
        .start_chaves_special(0, Command::Esp4Strong, context(false))
        .unwrap();

    assert!(result.applied);
    assert_eq!(state.fighters[0].action_id, ACTION_SPECIAL_4);
    assert_eq!(state.fighters[0].action_frame, 0);
    assert_eq!(state.fighters[0].action_duration, 22);
    assert_eq!(state.fighters[0].charge_power, 20.0);
    assert_eq!(state.fighters[0].attack_strength, STRENGTH_STRONG);

    let mut forbidden = state_with_player_action(ACTION_CROUCHING);
    let before = forbidden.fighters[0].clone();
    let result = forbidden
        .start_chaves_special(0, Command::Esp4Weak, context(false))
        .unwrap();
    assert_eq!(result.rejection, Some(Rejection::IneligibleAction));
    assert_eq!(forbidden.fighters[0], before);
}

#[test]
fn final_special_sets_word_power_sprite_flash_and_sound_in_pascal_order() {
    let mut state = state_with_player_action(ACTION_STRONG_PUNCH);
    state.fighters[0].power = 3;
    state.fighters[0].charge_power = 44.0;

    let result = state
        .start_chaves_special(0, Command::Final, context(false))
        .unwrap();

    assert!(result.applied);
    let fighter = &state.fighters[0];
    assert_eq!(fighter.action_id, ACTION_SPECIAL_5);
    assert_eq!(fighter.action_frame, 0);
    assert_eq!(fighter.action_duration, 38);
    assert_eq!(fighter.charge_power, 0.0);
    assert_eq!(fighter.power, 2);
    assert_eq!(fighter.sprite_frame, 36);
    assert_eq!(fighter.attack_strength, STRENGTH_SPECIAL);
    assert_eq!(state.screen_flash, 22);
    assert_eq!(state.pending_sound, SOUND_FINAL_SPECIAL);
    assert_eq!(
        result.events,
        vec![
            Event::ActionChanged {
                value: ACTION_SPECIAL_5
            },
            Event::ActionFrameChanged { value: 0 },
            Event::ActionDurationChanged { value: 38 },
            Event::ChargePowerChanged { value: 0.0 },
            Event::PowerChanged { value: 2 },
            Event::SpriteFrameChanged { value: 36 },
            Event::AttackStrengthChanged {
                value: STRENGTH_SPECIAL
            },
            Event::ScreenFlashChanged { value: 22 },
            Event::PendingSoundChanged {
                value: SOUND_FINAL_SPECIAL
            },
        ]
    );
}

#[test]
fn final_special_uses_low_health_gate_and_preserves_word_underflow() {
    let mut state = state_with_player_action(ACTION_CROUCHING);
    state.fighters[0].health = 15.0;
    state.fighters[0].power = 0;
    state.fighters[0].charge_power = 12.0;

    let result = state
        .start_chaves_special(0, Command::Final, context(false))
        .unwrap();

    assert!(result.applied);
    // ChavesTipos declares Poder as Word; the Pascal script assignment narrows
    // the result back to 16 bits rather than producing a signed negative value.
    assert_eq!(state.fighters[0].power, u16::MAX);
    assert_eq!(state.fighters[0].charge_power, 12.0);

    let mut unavailable = state_with_player_action(ACTION_IDLE);
    unavailable.fighters[0].health = 15.01;
    unavailable.fighters[0].power = 0;
    let before = unavailable.fighters[0].clone();
    let blocked = unavailable
        .start_chaves_special(0, Command::Final, context(false))
        .unwrap();
    assert_eq!(blocked.rejection, Some(Rejection::FinalUnavailable));
    assert_eq!(unavailable.fighters[0], before);
}

#[test]
fn player_two_uses_the_pascal_sprite_offset_of_fifty() {
    let mut state = state_with_player_action(ACTION_IDLE);
    state.fighters[1] = fighter(CHARACTER_CHAVES, ACTION_IDLE);

    let result = state
        .start_chaves_special(1, Command::Final, context(false))
        .unwrap();

    assert!(result.applied);
    assert_eq!(state.fighters[1].sprite_frame, 86);
}

#[test]
fn special_entry_rejects_the_wrong_character_and_invalid_player_index() {
    let mut state = state_with_player_action(ACTION_IDLE);
    let wrong_character = state
        .start_chaves_special(1, Command::Final, context(false))
        .unwrap_err();
    assert_eq!(
        wrong_character,
        ChavesSpecialError::NotChaves {
            actual_character_id: 2
        }
    );

    let invalid_index = state
        .start_chaves_special(2, Command::Final, context(false))
        .unwrap_err();
    assert_eq!(
        invalid_index,
        ChavesSpecialError::FighterIndexOutOfRange(2)
    );
}

#[test]
fn all_four_ordinary_specials_leave_pending_sound_untouched_at_entry() {
    let commands = [
        (ACTION_IDLE, Command::Esp1Strong),
        (ACTION_IDLE, Command::Esp2Strong),
        (ACTION_IDLE, Command::Esp3Strong),
        (ACTION_IDLE, Command::Esp4Strong),
    ];

    for (action, command) in commands {
        let mut state = state_with_player_action(action);
        state.pending_sound = 77;
        state.screen_flash = 9;
        if matches!(command, Command::Esp3Strong) {
            state.fighters[0].y = 220.0;
        }

        let result = state
            .start_chaves_special(0, command, context(false))
            .unwrap();

        assert!(result.applied);
        assert_eq!(state.pending_sound, 77);
        assert_eq!(state.screen_flash, 9);
    }
}
