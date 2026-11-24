//! Headless integration tests for the concrete fight tick orchestrator.

use std::time::Duration;

use street_chaves::chaves_specials::{ChavesSpecialCommand, ChavesSpecialContext};
use street_chaves::combat_runtime::{
    BodyBox, CombatState, FighterState, VictoryAnimationContext, ACTION_CROUCHING,
    ACTION_CROUCH_STRONG_KICK, ACTION_IDLE, ACTION_SPECIAL1, ACTION_SPECIAL4, ACTION_SPECIAL_FINAL,
    ACTION_WEAK_PUNCH, FORCE_WEAK, SIDE_LEFT, SIDE_RIGHT,
};
use street_chaves::effect_runtime::EFFECT_SIDE_RIGHT;
use street_chaves::fight_simulation::{FightSimulation, FightSimulationConfig};
use street_chaves::input::{Button, Direction, InputEvent, InputFrame};
use street_chaves::pascal_replay::PascalRandomSource;
use street_chaves::runtime::{FixedStepRunner, SimulationTick};

fn fighter(character_id: u8, action_id: u16, action_frame: u16) -> FighterState {
    FighterState {
        sprite_frame: 0,
        character_id,
        action_id,
        action_frame,
        action_duration: 30,
        facing: SIDE_RIGHT,
        force_strength: FORCE_WEAK,
        defending: false,
        invulnerable: false,
        hit_count: 0,
        attack_active: false,
        walking_forward: false,
        attack_box: None,
        is_kick: false,
        recoil: 0,
        vertical_velocity: 0,
        in_air: false,
        x: 100.0,
        y: 240.0,
        speed: 0.0,
        distance_ai: 100,
        health: 100.0,
        power: 100,
        charge_power: 0.0,
        jump_direction: 0,
        low_time: 0,
        back_time: 0,
        sequence_state: 0,
        sequence_timer: 0,
        punch_charge: 0,
        kick_charge: 0,
        body: BodyBox {
            x1: -18.0,
            y1: 30.0,
            x2: 13.0,
            y2: 170.0,
        },
    }
}

fn simulation(first_action: u16, first_frame: u16) -> FightSimulation {
    simulation_for_character(1, first_action, first_frame)
}

fn simulation_for_character(
    first_character_id: u8,
    first_action: u16,
    first_frame: u16,
) -> FightSimulation {
    simulation_for_matchup(first_character_id, 2, first_action, first_frame)
}

fn simulation_for_matchup(
    first_character_id: u8,
    second_character_id: u8,
    first_action: u16,
    first_frame: u16,
) -> FightSimulation {
    let state = CombatState {
        tick_index: 0,
        game_state: 7,
        camera_x: 0.0,
        clock_stopped: false,
        fighters: [
            fighter(first_character_id, first_action, first_frame),
            fighter(second_character_id, ACTION_IDLE, 0),
        ],
        effects: Default::default(),
        pending_sound: None,
        screen_flash: 0,
        screen_shake: 0,
    };
    FightSimulation::new(
        state,
        FightSimulationConfig {
            special: ChavesSpecialContext {
                charge_gain: 1.0,
                charge_max: 100.0,
                ground_y: 240.0,
            },
            frame: VictoryAnimationContext {
                sprite_offset: 0,
                ground_y: 240.0,
                camera_x: 0.0,
                camera_left_limit: 0.0,
                camera_right_limit: 450.0,
            },
            effect_camera_y: 0.0,
        },
        2026,
    )
}

#[test]
fn fixed_runner_invokes_concrete_fight_at_28ms_and_advances_c01_frame() {
    let mut fight = simulation(ACTION_SPECIAL1, 0);
    let mut runner = FixedStepRunner::new();

    runner.advance(Duration::from_millis(27), &mut fight);
    assert_eq!(fight.state.fighters[0].action_frame, 0);

    let batch = runner.advance(Duration::from_millis(1), &mut fight);
    assert_eq!(batch.ticks_due, 1);
    assert_eq!(fight.state.tick_index, 1);
    assert_eq!(fight.state.fighters[0].sprite_frame, 36);
    assert_eq!(fight.state.fighters[0].action_frame, 1);
    assert_eq!(fight.last_report().c01_frames.len(), 1);
}

#[test]
fn common_and_character_quadro_update_both_players_before_box_application() {
    let mut fight = simulation_for_matchup(1, 4, ACTION_IDLE, 0);
    fight.tick(InputFrame::default());

    assert_eq!(fight.state.fighters[0].sprite_frame, 1);
    assert_eq!(fight.state.fighters[1].sprite_frame, 51);
    assert_eq!(
        fight.state.fighters[0].body,
        BodyBox {
            x1: -13.0,
            y1: 35.0,
            x2: 19.0,
            y2: 164.0,
        }
    );
    assert_eq!(
        fight.state.fighters[1].body,
        BodyBox {
            x1: -17.0,
            y1: 20.0,
            x2: 14.0,
            y2: 160.0,
        }
    );
    assert_eq!(
        fight.last_report().rot_frame_executions,
        vec![(0, 1), (1, 4)]
    );
    assert_eq!(fight.last_report().box_updates, vec![0, 1]);
    assert!(fight.last_report().errors.is_empty(), "frame/runtime errors: {:?}", fight.last_report().errors);
}

#[test]
fn applied_hurtbox_is_mirrored_when_the_fighter_faces_left() {
    let mut fight = simulation(ACTION_IDLE, 0);
    fight.state.fighters[0].facing = SIDE_LEFT;
    fight.tick(InputFrame::default());

    assert_eq!(fight.state.fighters[0].body.x1, -19.0);
    assert_eq!(fight.state.fighters[0].body.x2, 13.0);
    assert_eq!(fight.state.fighters[0].body.y1, 35.0);
    assert_eq!(fight.state.fighters[0].body.y2, 164.0);
}

#[test]
fn queued_c01_command_enters_special_then_runs_its_frame_executor() {
    let mut fight = simulation(ACTION_IDLE, 0);
    assert!(fight.queue_chaves_special(0, ChavesSpecialCommand::Esp1Weak));

    fight.tick(InputFrame::default());

    assert_eq!(fight.state.fighters[0].action_id, ACTION_SPECIAL1);
    assert_eq!(fight.state.fighters[0].sprite_frame, 36);
    assert_eq!(fight.last_report().special_outcomes.len(), 1);
    assert!(fight.last_report().special_outcomes[0].1.applied);
}

#[test]
fn c01_frame_10_spawns_a_live_type7_effect_before_effect_phase() {
    let mut fight = simulation(ACTION_SPECIAL1, 10);
    fight.tick(InputFrame::default());

    assert_eq!(fight.state.effects.active_count(), 1);
    let effect = fight.state.effects.instance(0).unwrap();
    assert_eq!(effect.effect_type, 7);
    assert_eq!(effect.owner, 1);
    assert_eq!(
        fight.last_report().c01_frames[0].1.spawned_effect_slots,
        vec![0]
    );
    assert_eq!(fight.last_report().effects.render_snapshots.len(), 1);
}

#[test]
fn effect_arena_advances_once_per_logical_tick_without_renderer() {
    struct NoDrawRng;
    impl PascalRandomSource for NoDrawRng {
        type Error = String;
        fn draw_longint(&mut self, _range: i32) -> Result<i32, Self::Error> {
            Err("unexpected random draw".into())
        }
        fn draw_int64(&mut self, _range: i64) -> Result<i64, Self::Error> {
            Err("unexpected random draw".into())
        }
    }

    let mut fight = simulation(ACTION_IDLE, 0);
    fight
        .state
        .effects
        .spawn(2, None, 0, EFFECT_SIDE_RIGHT, 120.0, 80.0, &mut NoDrawRng)
        .unwrap();
    fight.tick(InputFrame::default());

    let effect = fight.state.effects.instance(0).unwrap();
    assert_eq!(effect.frame, 1);
    assert_eq!(effect.y, 79.0);
    assert_eq!(fight.last_report().effects.render_snapshots.len(), 1);
}

#[test]
fn input_is_delivered_to_fight_tick_without_inventing_button_commands() {
    let mut fight = simulation(ACTION_IDLE, 0);
    let mut runner = FixedStepRunner::new();
    runner.handle_input_event(InputEvent::ButtonChanged {
        button: Button(3),
        down: true,
    });
    runner.advance(Duration::from_millis(28), &mut fight);

    assert!(fight.last_report().input.is_held(Button(3)));
    assert!(fight.last_report().input.was_pressed(Button(3)));
    assert_eq!(
        fight.state.fighters[0].action_id,
        street_chaves::combat_runtime::ACTION_STRONG_KICK
    );
    assert_eq!(fight.state.fighters[0].action_duration, 11);
}

#[test]
fn stopped_fight_retains_queued_command_and_freezes_fighter_frame() {
    let mut fight = simulation(ACTION_IDLE, 0);
    fight.state.clock_stopped = true;
    assert!(fight.queue_chaves_special(0, ChavesSpecialCommand::Esp1Weak));
    fight.tick(InputFrame::default());

    assert_eq!(fight.state.fighters[0].action_id, ACTION_IDLE);
    assert_eq!(fight.state.fighters[0].action_frame, 0);
    assert_eq!(fight.last_report().special_outcomes.len(), 0);

    fight.state.clock_stopped = false;
    fight.tick(InputFrame::default());
    assert_eq!(fight.state.fighters[0].action_id, ACTION_SPECIAL1);
}

#[test]
fn real_c01_sequence_and_button_rules_reach_the_c01_dispatcher() {
    let mut fight = simulation(ACTION_IDLE, 0);
    fight.tick_with_inputs([
        InputFrame {
            direction: Direction::Down,
            ..Default::default()
        },
        InputFrame::default(),
    ]);
    assert_eq!(fight.state.fighters[0].sequence_state, 1);

    fight.tick_with_inputs([
        InputFrame {
            direction: Direction::Right,
            ..Default::default()
        },
        InputFrame::default(),
    ]);
    assert_eq!(fight.state.fighters[0].sequence_state, 2);

    fight.tick_with_inputs([
        InputFrame {
            pressed_buttons: 1,
            ..Default::default()
        },
        InputFrame::default(),
    ]);
    assert_eq!(fight.last_report().command_matches.len(), 2);
    assert_eq!(fight.last_report().command_matches[0].command_code, 16);
    assert_eq!(fight.last_report().rot_action_executions, vec![(0, 1, 16)]);
    assert!(fight.last_report().special_outcomes.is_empty());
    assert_eq!(fight.state.fighters[0].action_id, ACTION_SPECIAL1);
    assert!(!fight.last_report().unhandled_commands.iter().any(|item| {
        item.command_code == 3
            && item.source
                == street_chaves::fight_simulation::UnhandledCommandSource::BasicButtonAction
    }));
}

#[test]
fn real_character_data_exposes_unimplemented_commands_without_faking_execution() {
    let mut fight = simulation_for_character(13, ACTION_IDLE, 0);
    fight.state.fighters[0].sequence_state = 6;
    fight.tick(InputFrame {
        pressed_buttons: 8,
        ..Default::default()
    });
    assert!(fight
        .last_report()
        .command_matches
        .iter()
        .any(|item| item.command_code == 25));
    assert_eq!(fight.state.fighters[0].action_id, ACTION_SPECIAL4);
    assert_eq!(fight.state.fighters[0].force_strength, 4);
    assert!(!fight
        .last_report()
        .unhandled_commands
        .iter()
        .any(|item| item.command_code == 25));
    assert!(fight.last_report().errors.is_empty(), "frame/runtime errors: {:?}", fight.last_report().errors);
}

#[test]
fn sequence_timeout_is_paused_during_hitstop_and_resets_after_nine_ticks() {
    let mut fight = simulation(ACTION_IDLE, 0);
    fight.state.fighters[0].sequence_state = 6;
    fight.state.fighters[0].sequence_timer = 1;
    fight.state.clock_stopped = true;
    fight.tick(InputFrame::default());
    assert_eq!(fight.state.fighters[0].sequence_state, 6);
    assert_eq!(fight.state.fighters[0].sequence_timer, 1);

    fight.state.clock_stopped = false;
    fight.state.screen_flash = 0;
    fight.tick(InputFrame::default());
    assert_eq!(fight.state.fighters[0].sequence_state, 0);
    assert_eq!(fight.state.fighters[0].sequence_timer, 0);
}

#[test]
fn common_weak_punch_executes_with_pascal_duration_charge_and_sound() {
    let mut fight = simulation_for_character(2, ACTION_IDLE, 0);
    fight.tick(InputFrame {
        pressed_buttons: 1,
        ..InputFrame::default()
    });

    let fighter = &fight.state.fighters[0];
    assert_eq!(fighter.action_id, ACTION_WEAK_PUNCH);
    assert_eq!(fighter.action_frame, 0);
    assert_eq!(fighter.action_duration, 6);
    assert_eq!(fighter.charge_power, 1.0);
    assert_eq!(fight.state.pending_sound, Some(5));
    assert!(fight.last_report().unhandled_commands.is_empty());
}

#[test]
fn jaiminho_crouched_strong_kick_uses_the_18_tick_pascal_duration() {
    let mut fight = simulation_for_character(13, ACTION_CROUCHING, 0);
    fight.tick(InputFrame {
        // L7_Entrada processes direction before button edges. Holding Down
        // prevents the neutral-input crouch-release path from running first.
        direction: Direction::Down,
        pressed_buttons: 8,
        ..InputFrame::default()
    });

    let fighter = &fight.state.fighters[0];
    assert_eq!(fighter.action_id, ACTION_CROUCH_STRONG_KICK);
    assert_eq!(fighter.action_duration, 18);
    assert!(fighter.is_kick);
}

#[test]
fn c13_extra1_uses_its_character_specific_force_and_special4_entry() {
    let mut fight = simulation_for_character(13, ACTION_IDLE, 0);
    fight.state.fighters[0].sequence_state = 6;
    fight.tick(InputFrame {
        pressed_buttons: 8,
        ..InputFrame::default()
    });

    let fighter = &fight.state.fighters[0];
    assert_eq!(fighter.action_id, ACTION_SPECIAL4);
    assert_eq!(fighter.action_duration, 30);
    assert_eq!(fighter.force_strength, 4);
    assert_eq!(fighter.charge_power, 5.0);
}

#[test]
fn c13_extra2_consumes_power_and_enters_final_special() {
    let mut fight = simulation_for_character(13, ACTION_IDLE, 0);
    fight.state.fighters[0].sequence_state = 10;
    fight.state.fighters[0].power = 1;
    fight.tick(InputFrame {
        pressed_buttons: 1,
        ..InputFrame::default()
    });

    let fighter = &fight.state.fighters[0];
    assert_eq!(fighter.action_id, ACTION_SPECIAL_FINAL);
    assert_eq!(fighter.action_duration, 30);
    assert_eq!(fighter.power, 0);
    assert_eq!(fighter.sprite_frame, 37);
    assert_eq!(fight.state.screen_flash, 22);
}
