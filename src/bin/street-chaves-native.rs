//! First native runnable loop for the Street Chaves Rust reconstruction.
//!
//! Without --game-root it opens a procedural training stage with animated vector fighters.
//! With --game-root it loads the original local Pascal BMP/INI/WAV resource layout.

use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use foundation_runtime::RuntimeConfig;
use street_chaves::combat_runtime::{
    BodyBox, FighterState, ACTION_IDLE, SIDE_LEFT, SIDE_RIGHT,
};
use street_chaves::fight_simulation::{FightSimulation, FightSimulationConfig};
use street_chaves::native_foundation::NativeFightHost;
use street_chaves::{ChavesSpecialContext, CombatState, VictoryAnimationContext};

#[derive(Debug)]
struct Options {
    game_root: Option<PathBuf>,
    stage: u8,
    players: [u8; 2],
    seed: i32,
    no_audio: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("street-chaves-native: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_args()?;
    let mut fight = build_fight(options.players, options.seed);

    let mut native = NativeFightHost::new(RuntimeConfig {
        title: "Street Chaves Rust".to_owned(),
        width: 400,
        height: 300,
        pixel_art: true,
    })?;

    if let Some(root) = options.game_root.as_ref() {
        for message in native.load_original_game(
            root,
            options.stage,
            options.players,
            &mut fight,
            !options.no_audio,
        ) {
            eprintln!("[assets] {message}");
        }
    } else {
        eprintln!("[assets] no --game-root supplied; using procedural training stage and vector fighters");
    }

    eprintln!("[game] P1={}, P2={}, stage={}, fixed tick=28ms", options.players[0], options.players[1], options.stage);
    eprintln!("[controls] P1 arrows + Z/X/C/V; P2 WASD + R/T/Y/U; ESC exits");

    let mut last_frame = std::time::Instant::now();
    while !native.should_quit() {
        native.poll_and_tick(&mut fight)?;
        native.update_camera(&mut fight);
        native.render_fight(&fight, &[])?;

        // Rendering is intentionally decoupled from the authoritative 28 ms simulation clock.
        let elapsed = last_frame.elapsed();
        let target = Duration::from_millis(16);
        if elapsed < target {
            thread::sleep(target - elapsed);
        }
        last_frame = std::time::Instant::now();
    }
    Ok(())
}

fn build_fight(players: [u8; 2], seed: i32) -> FightSimulation {
    let fighters = [
        make_fighter(players[0], 136.0, SIDE_RIGHT, 156, 8.0),
        make_fighter(players[1], 264.0, SIDE_LEFT, 149, 8.0),
    ];
    let state = CombatState {
        tick_index: 0,
        game_state: 7,
        camera_x: 0.0,
        clock_stopped: false,
        fighters,
        effects: Default::default(),
        pending_sound: None,
        screen_flash: 0,
        screen_shake: 0,
    };
    FightSimulation::new(
        state,
        FightSimulationConfig {
            special: ChavesSpecialContext {
                charge_gain: 2.0,
                charge_max: 100.0,
                ground_y: 300.0,
            },
            frame: VictoryAnimationContext {
                sprite_offset: 0,
                ground_y: 300.0,
                camera_x: 0.0,
                camera_left_limit: 0.0,
                camera_right_limit: 400.0,
            },
            effect_camera_y: 0.0,
        },
        seed,
    )
}

fn make_fighter(
    character_id: u8,
    x: f64,
    facing: u8,
    sprite_height: i32,
    walk_speed: f64,
) -> FighterState {
    FighterState {
        sprite_frame: 0,
        character_id,
        action_id: ACTION_IDLE,
        action_frame: 0,
        action_duration: 99,
        facing,
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
        hitstun: 0,
        rotation_step: 0,
        vertical_velocity: 0,
        in_air: false,
        x,
        y: 300.0,
        speed: walk_speed,
        distance_ai: 100,
        health: 100.0,
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
            y2: f64::from(sprite_height),
        },
    }
}

fn parse_args() -> Result<Options, Box<dyn std::error::Error>> {
    let mut options = Options {
        game_root: None,
        stage: 1,
        players: [1, 4],
        seed: 2026,
        no_audio: false,
    };
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--game-root" => {
                options.game_root = Some(PathBuf::from(
                    args.next().ok_or("--game-root expects a directory")?,
                ));
            }
            "--stage" => {
                options.stage = args.next().ok_or("--stage expects a number")?.parse()?;
            }
            "--p1" => {
                options.players[0] = args.next().ok_or("--p1 expects a character ID")?.parse()?;
            }
            "--p2" => {
                options.players[1] = args.next().ok_or("--p2 expects a character ID")?.parse()?;
            }
            "--seed" => {
                options.seed = args.next().ok_or("--seed expects an integer")?.parse()?;
            }
            "--no-audio" => options.no_audio = true,
            unknown => return Err(format!("unknown argument: {unknown}").into()),
        }
    }
    if !(1..=15).contains(&options.players[0]) || !(1..=15).contains(&options.players[1]) {
        return Err("--p1/--p2 must be original Pascal fighter IDs in 1..=15".into());
    }
    if options.stage == 0 {
        return Err("--stage must be greater than zero".into());
    }
    Ok(options)
}

fn print_help() {
    println!(
        "Street Chaves native runtime\n\
         \n\
         Usage:\n\
           cargo run --features native-foundation --bin street-chaves-native -- [options]\n\
         \n\
         Options:\n\
           --game-root PATH   Local OpenStreetChaves installation/data folder\n\
           --stage N          Original stage number (default: 1)\n\
           --p1 N             P1 Pascal fighter ID 1..15 (default: 1, Chaves)\n\
           --p2 N             P2 Pascal fighter ID 1..15 (default: 4, Quico)\n\
           --seed N           Deterministic RNG seed (default: 2026)\n\
           --no-audio         Do not load local WAV banks\n\
           --help             Show this help\n\
         \n\
         Controls:\n\
           P1: arrows + Z/X/C/V\n\
           P2: WASD + R/T/Y/U\n\
           ESC: quit"
    );
}
