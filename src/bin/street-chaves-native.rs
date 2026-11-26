//! Native startup-to-fight executable for Street Chaves Rust.

use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use foundation_runtime::RuntimeConfig;
use street_chaves::chaves_specials::ChavesSpecialContext;
use street_chaves::combat_runtime::{
    BodyBox, CombatState, FighterState, VictoryAnimationContext, ACTION_IDLE, SIDE_LEFT, SIDE_RIGHT,
};
use street_chaves::fight_simulation::{FightSimulation, FightSimulationConfig};
use street_chaves::native_foundation::NativeFightHost;
use street_chaves::startup_flow::{StartupEvent, StartupFlow, StartupPhase};

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
    let mut startup = StartupFlow::new(options.players);

    let mut host = NativeFightHost::new(RuntimeConfig {
        title: "Street Chaves Rust".to_owned(),
        width: 400,
        height: 300,
        pixel_art: true,
    })?;

    eprintln!("[boot] Opening -> Start -> Character Select -> Fight");
    eprintln!("[controls] P1 arrows + Z/X/C/V; P2 WASD + R/T/Y/U; ESC quits");

    let mut previous_frame = Instant::now();
    while !host.should_quit() {
        let (_batch, samples) = host.poll_input_ticks()?;
        for inputs in samples {
            match startup.tick(inputs) {
                StartupEvent::StartFight { characters } => {
                    eprintln!(
                        "[select] P1={} P2={}",
                        characters[0],
                        characters[1]
                    );
                    fight = build_fight(characters, options.seed);
                    if let Some(root) = options.game_root.as_ref() {
                        for line in host.load_original_game(
                            root,
                            options.stage,
                            characters,
                            &mut fight,
                            !options.no_audio,
                        ) {
                            eprintln!("[assets] {line}");
                        }
                    } else {
                        eprintln!("[assets] no --game-root: procedural training stage and fighter fallbacks");
                    }
                }
                StartupEvent::None if startup.phase() == StartupPhase::Fighting => {
                    host.tick_fight(&mut fight, inputs);
                }
                StartupEvent::None => {}
            }
        }

        if startup.phase() == StartupPhase::Fighting {
            host.update_camera(&mut fight);
            host.render_fight(&fight, &[])?;
        } else {
            host.render_startup(&startup.screen())?;
        }

        // Render at roughly 60 Hz; the simulation only advances on full 28 ms ticks.
        let frame_time = previous_frame.elapsed();
        let target = Duration::from_millis(16);
        if frame_time < target {
            thread::sleep(target - frame_time);
        }
        previous_frame = Instant::now();
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
                charge_max: 102.0,
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
    height: i32,
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
            y2: f64::from(height),
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
        return Err("--p1 and --p2 must be original Pascal fighter IDs 1..=15".into());
    }
    if options.stage == 0 {
        return Err("--stage must be greater than zero".into());
    }
    Ok(options)
}

fn print_help() {
    println!(
        "Street Chaves native runtime\n\n\
         Usage:\n\
           cargo run --features native-foundation --bin street-chaves-native -- [options]\n\n\
         Options:\n\
           --game-root PATH   Local OpenStreetChaves installation/data directory\n\
           --stage N          Original stage ID (default: 1)\n\
           --p1 N             Initial P1 selection (default: 1)\n\
           --p2 N             Initial P2 selection (default: 4)\n\
           --seed N           Deterministic RNG seed (default: 2026)\n\
           --no-audio         Skip local WAV resource loading\n\
           --help             Show this help\n\n\
         Startup: Opening -> Start -> Fighter Select -> Fight\n\
         P1: arrows + Z/X/C/V; P2: WASD + R/T/Y/U; ESC quits"
    );
}
