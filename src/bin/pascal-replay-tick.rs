//! Replay the captured Pascal tick-316 AI path through the Rust runtime.
//!
//! Usage:
//! cargo run --bin pascal-replay-tick -- tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl
//!
//! Fighter state and all five AI inputs are reconstructed from the captured
//! script_start.ai_inputs snapshot. Camera position and clock-stopped state are
//! not included in that snapshot; this sample does not enter those edge cases.

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

    // Exact body boxes and fighter state captured at tick-316 IA entry.
    let mut state = CombatState {
        tick_index: 316,
        game_state: 7,
        camera_x: 0.0,
        clock_stopped: false,
        effects: Default::default(),
        pending_sound: None,
        screen_flash: 0,
        screen_shake: 0,
        // This captured Common [IA] fixture omits GolpeAtivo; the replayed path does not read it.
        fighters: [
            FighterState {
                sprite_frame: 0,
                character_id: 2,
                action_id: 1,
                action_frame: 0,
                action_duration: 99,
                facing: SIDE_RIGHT,
                force_strength: 0,
                defending: false,
                invulnerable: false,
                hit_count: 0,
                hit_timer: 0,
                attack_damage: 0,
                angular_velocity: 0,
                attack_active: false,
                walking_forward: false,
                attack_box: None,
                is_kick: false,
                recoil: 0,
                vertical_velocity: 0,
                in_air: false,
                x: 234.0,
                y: 300.0,
                speed: 8.0,
                distance_ai: 0,
                health: 125.0,
                power: 0,
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
            },
            FighterState {
                sprite_frame: 0,
                character_id: 1,
                action_id: 1,
                action_frame: 0,
                action_duration: 99,
                facing: SIDE_LEFT,
                force_strength: 0,
                defending: false,
                invulnerable: false,
                hit_count: 0,
                hit_timer: 0,
                attack_damage: 0,
                angular_velocity: 0,
                attack_active: false,
                walking_forward: false,
                attack_box: None,
                is_kick: false,
                recoil: 0,
                vertical_velocity: 0,
                in_air: false,
                x: 366.0,
                y: 300.0,
                speed: 8.0,
                distance_ai: 0,
                health: 125.0,
                power: 0,
                charge_power: 0.0,
                jump_direction: 0,
                low_time: 0,
                back_time: 0,
                sequence_state: 0,
                sequence_timer: 0,
                punch_charge: 0,
                kick_charge: 0,
                body: BodyBox {
                    x1: -19.0,
                    y1: 35.0,
                    x2: 13.0,
                    y2: 164.0,
                },
            },
        ],
    };

    let tick = match state.tick_common_ai(
        AiTickInputs {
            distance: 132,
            projectile_distance: 0,
            opponent_y: 300,
            threat: 0,
            difficulty_factor: 3.6,
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
        eprintln!(
            "RNG {:?} range={} -> {}",
            draw.width, draw.range, draw.value
        );
    }
    print!("{}", tick.checkpoint_jsonl);
    ExitCode::SUCCESS
}
