//! Native startup-to-fight executable for Street Chaves Rust.

use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use foundation_runtime::RuntimeConfig;
use street_chaves::arcade::{ArcadePreferences, ArcadeRoundProgress, RoundAdvance};
use street_chaves::campaign::{CampaignAdvance, CampaignProfile, CampaignRuntime, CampaignRuntimeState, CampaignStageProfile};
use street_chaves::input::Button;
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
    let preferences_path = PathBuf::from("StreetChavesRust.settings.json");
    let preferences = std::fs::read_to_string(&preferences_path)
        .ok()
        .and_then(|contents| ArcadePreferences::from_json(&contents))
        .unwrap_or_default();
    let mut fight = build_fight(options.players, options.seed);
    let mut startup = StartupFlow::with_preferences(options.players, preferences);

    let mut host = NativeFightHost::new(RuntimeConfig {
        title: "Street Chaves Rust".to_owned(),
        width: 400,
        height: 300,
        pixel_art: true,
    })?;
    host.set_player_one_key_map(startup.preferences().player_one_keys);
    host.set_damage_level(startup.settings().damage_level);

    if !options.no_audio {
        if let Some(root) = options.game_root.as_ref() {
            if host.start_menu_audio(root) {
                eprintln!("[audio] original menu music ready");
            }
        }
    }

    let mut campaign: Option<CampaignRuntime> = None;
    let mut rounds: Option<ArcadeRoundProgress> = None;
    let mut cpu_opponent_active = false;
    let mut standalone_winner: Option<usize> = None;
    let mut player_character = options.players[0];
    let mut match_settings = startup.settings();

    eprintln!("[boot] Opening -> Title -> F1 Config / F2 Remap -> Character Select -> Fight");
    eprintln!("[arcade] 5 inserts a credit; 1 starts P1; 2 starts or joins P2");
    eprintln!("[controls] P1 arrows + Z/X/C/V; P2 WASD + R/T/Y/U; ESC quits");

    let mut previous_frame = Instant::now();
    while !host.should_quit() {
        let (_batch, samples) = host.poll_input_ticks()?;
        for inputs in samples {
            match startup.tick(inputs) {
                StartupEvent::StartFight {
                    characters,
                    cpu_opponent,
                } => {
                    let settings = startup.settings();
                    host.set_arcade_settings(settings);
                    host.set_credits(startup.credits());
                    match_settings = settings;
                    player_character = characters[0];
                    cpu_opponent_active = cpu_opponent;
                    campaign = None;
                    standalone_winner = None;
                    rounds = Some(ArcadeRoundProgress::new(settings));
                    eprintln!(
                        "[select] P1={} P2={} CPU={}",
                        characters[0], characters[1], cpu_opponent
                    );
                    fight = build_fight(characters, options.seed);
                    configure_fight(
                        &mut fight,
                        cpu_opponent,
                        settings.difficulty,
                        settings.power_mode,
                    );
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
                        if cpu_opponent {
                            let path = root.join(format!("Dados/Lutadores/C{:02}.ini", characters[0]));
                            match std::fs::read_to_string(&path)
                                .map_err(|error| format!("{}: {error}", path.display()))
                                .and_then(|ini| CampaignProfile::from_ini(characters[0], &ini))
                                .and_then(|profile| fight.generate_campaign_plan(&profile, 15)) {
                                Ok(plan) => {
                                    campaign = Some(CampaignRuntime::new(plan));
                                    eprintln!("[campaign] eight-phase route generated from {}", path.display());
                                }
                                Err(error) => eprintln!("[campaign] unavailable; using single fight: {error}"),
                            }
                        }
                    } else {
                        eprintln!("[assets] no --game-root: procedural training stage and fighter fallbacks");
                    }
                    host.set_damage_level(settings.damage_level);
                    host.set_player_one_key_map(startup.preferences().player_one_keys);
                }
                StartupEvent::JoinPlayer { player: 2 } => {
                    campaign = None;
                    cpu_opponent_active = false;
                    fight.configure_cpu_opponent(false, startup.settings().difficulty);
                    eprintln!("[arcade] Player 2 joined; CPU disabled");
                }
                StartupEvent::PreferencesChanged => {
                    let preferences = startup.preferences();
                    host.set_arcade_settings(preferences.settings);
                    host.set_credits(startup.credits());
                    if !options.no_audio {
                        if startup.phase() == StartupPhase::Config {
                            if preferences.settings.sound_test > 0 {
                                let _ = host.play_sound_test(preferences.settings.sound_test);
                            }
                            let _ = host.play_selected_fight_music(usize::from(preferences.settings.music_track));
                        } else if let Some(root) = options.game_root.as_ref() {
                            let _ = host.start_menu_audio(root);
                        }
                    }
                    if let Err(error) = std::fs::write(&preferences_path, preferences.to_json()) {
                        eprintln!(
                            "[settings] could not save {}: {error}",
                            preferences_path.display()
                        );
                    }
                    match_settings = preferences.settings;
                    host.set_player_one_key_map(preferences.player_one_keys);
                    host.set_damage_level(preferences.settings.damage_level);
                    eprintln!("[settings] preferences saved");
                }
                StartupEvent::None if startup.phase() == StartupPhase::Fighting => {
                    if let Some(run) = campaign.as_mut() {
                        match run.state() {
                            CampaignRuntimeState::Adversaries => {
                                if run.tick_adversaries(inputs[0].was_pressed(Button(0))) {
                                    let phase = run.current_phase().unwrap_or(1);
                                    let opponent = run.current_opponent().unwrap_or(1);
                                    let path = options.game_root.as_ref()
                                        .map(|root| root.join(format!("Dados/Lutadores/C{opponent:02}.ini")));
                                    let stage = path.as_ref()
                                        .ok_or_else(|| "campaign requires --game-root".to_owned())
                                        .and_then(|path| std::fs::read_to_string(path)
                                            .map_err(|error| format!("{}: {error}", path.display())))
                                        .and_then(|ini| CampaignStageProfile::from_ini(&ini))
                                        .and_then(|profile| fight.choose_campaign_stage(profile, phase, 13, None));
                                    match stage {
                                        Ok(stage) => {
                                            if let Err(error) = fight.load_campaign_opponent(run.plan(), &run.progress()) {
                                                eprintln!("[campaign] opponent load: {error}");
                                            } else {
                                                fight.reset_for_campaign_fight();
                                                configure_fight(&mut fight, true, match_settings.difficulty, match_settings.power_mode);
                                                if let Some(root) = options.game_root.as_ref() {
                                                    for line in host.load_original_game(root, stage.stage, [player_character, opponent], &mut fight, !options.no_audio) {
                                                        eprintln!("[campaign] {line}");
                                                    }
                                                }
                                                match run.begin_fight(stage) {
                                                    Ok(_) => {
                                                        rounds = Some(ArcadeRoundProgress::new(match_settings));
                                                        eprintln!("[campaign] phase {phase}/8, C{opponent:02}, stage {}", stage.stage);
                                                    }
                                                    Err(error) => eprintln!("[campaign] transition: {error}"),
                                                }
                                            }
                                        }
                                        Err(error) => eprintln!("[campaign] stage selection: {error}"),
                                    }
                                }
                            }
                            CampaignRuntimeState::Fighting => {
                                host.tick_fight(&mut fight, inputs);
                                let timeout = rounds.as_mut().is_some_and(ArcadeRoundProgress::tick);
                                let p1 = fight.state.fighters[0].health;
                                let p2 = fight.state.fighters[1].health;
                                let winner = if p1 <= 0.0 && p2 <= 0.0 { Some(None) }
                                    else if p2 <= 0.0 { Some(Some(0)) }
                                    else if p1 <= 0.0 { Some(Some(1)) }
                                    else if timeout { Some(if p1 > p2 { Some(0) } else if p2 > p1 { Some(1) } else { None }) }
                                    else { None };
                                if let Some(winner) = winner {
                                    let advance = rounds.as_mut().map(|rounds| rounds.record_result(winner));
                                    match advance {
                                        Some(RoundAdvance::Continue { .. }) | Some(RoundAdvance::Draw { .. }) => {
                                            fight.reset_for_campaign_fight();
                                            configure_fight(&mut fight, true, match_settings.difficulty, match_settings.power_mode);
                                        }
                                        Some(RoundAdvance::MatchWon { winner: 0 }) => {
                                            let result = run.record_victory();
                                            if result == CampaignAdvance::Completed && !options.no_audio {
                                                let _ = host.play_victory_voice(player_character, 1);
                                            }
                                            match result {
                                                CampaignAdvance::NextPhase(phase) => {
                                                    rounds = Some(ArcadeRoundProgress::new(match_settings));
                                                    eprintln!("[campaign] victory; next phase {phase}/8");
                                                }
                                                CampaignAdvance::Completed => {
                                                    eprintln!("[campaign] completed all eight fights!");
                                                }
                                                _ => {}
                                            }
                                        }
                                        Some(RoundAdvance::MatchWon { winner: 1 }) => {
                                            run.record_defeat();
                                            eprintln!("[campaign] defeat in phase {}", run.current_phase().unwrap_or(0));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            CampaignRuntimeState::Completed | CampaignRuntimeState::Defeated => {}
                        }
                    } else if standalone_winner.is_none() {
                        host.tick_fight(&mut fight, inputs);
                        let timeout = rounds.as_mut().is_some_and(ArcadeRoundProgress::tick);
                        let p1 = fight.state.fighters[0].health;
                        let p2 = fight.state.fighters[1].health;
                        let winner = if p1 <= 0.0 && p2 <= 0.0 { Some(None) }
                            else if p2 <= 0.0 { Some(Some(0)) }
                            else if p1 <= 0.0 { Some(Some(1)) }
                            else if timeout { Some(if p1 > p2 { Some(0) } else if p2 > p1 { Some(1) } else { None }) }
                            else { None };
                        if let Some(winner) = winner {
                            let result = rounds.as_mut().map(|progress| progress.record_result(winner));
                            match result {
                                Some(RoundAdvance::Continue { round, winner }) => {
                                    fight.reset_for_campaign_fight();
                                    configure_fight(&mut fight, cpu_opponent_active, match_settings.difficulty, match_settings.power_mode);
                                    eprintln!("[rounds] round {round}; player {} leads", winner + 1);
                                }
                                Some(RoundAdvance::Draw { next_round }) => {
                                    fight.reset_for_campaign_fight();
                                    configure_fight(&mut fight, cpu_opponent_active, match_settings.difficulty, match_settings.power_mode);
                                    eprintln!("[rounds] draw; round {next_round}");
                                }
                                Some(RoundAdvance::MatchWon { winner }) => {
                                    standalone_winner = Some(winner);
                                    if !options.no_audio {
                                        let _ = host.stop_original_music();
                                        if winner == 0 { let _ = host.play_victory_voice(player_character, 1); }
                                    }
                                    eprintln!("[rounds] match complete; player {} wins", winner + 1);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                StartupEvent::None => {}
                StartupEvent::JoinPlayer { player } => {
                    eprintln!("[arcade] unsupported player slot join event: {player}");
                }
            }
        }

        if startup.phase() == StartupPhase::Fighting {
            if let Some(run) = campaign.as_ref() {
                if run.state() != CampaignRuntimeState::Fighting {
                    host.render_campaign(run, player_character)?;
                } else {
                    host.update_camera(&mut fight);
                    host.render_fight(&fight, &[])?;
                }
            } else if let Some(winner) = standalone_winner {
                host.render_match_result(winner)?;
            } else {
                host.update_camera(&mut fight);
                host.render_fight(&fight, &[])?;
            }
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

fn configure_fight(
    fight: &mut FightSimulation,
    cpu_opponent: bool,
    difficulty: u8,
    power_mode: u8,
) {
    fight.configure_cpu_opponent(cpu_opponent, difficulty);
    for player in 0..2 {
        let receives_max_power = match power_mode {
            2 => player == 0,
            3 => player == 1,
            4 => true,
            _ => false,
        };
        fight.state.fighters[player].power = if receives_max_power { 3 } else { 0 };
    }
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
