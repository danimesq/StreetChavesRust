//! WebAssembly entry point for the Street Chaves browser runtime.
//!
//! JavaScript owns the HTML page and requestAnimationFrame callback. The game
//! simulation, startup flow, fixed timestep, presentation composition and audio
//! events remain in Rust; Foundation owns Canvas 2D, input and WebAudio.

use crate::arcade::{ArcadePreferences, ArcadeRoundProgress, RoundAdvance};
use crate::campaign::{CampaignAdvance, CampaignProfile, CampaignRuntime, CampaignRuntimeState, CampaignStageProfile};
use crate::input::{Button, Direction, InputEvent};
use crate::chaves_specials::ChavesSpecialContext;
use crate::combat_runtime::{
    BodyBox, CombatState, FighterState, VictoryAnimationContext, ACTION_IDLE, SIDE_LEFT, SIDE_RIGHT,
};
use crate::fight_simulation::{FightSimulation, FightSimulationConfig};
use crate::native_foundation::WebFightHost;
use crate::startup_flow::{StartupEvent, StartupFlow, StartupPhase};
use foundation_runtime::web::LoadedWebAssets;
use foundation_runtime::RuntimeConfig;
use js_sys::Array;
use wasm_bindgen::prelude::*;

const LOGICAL_WIDTH: u32 = 400;
const LOGICAL_HEIGHT: u32 = 300;
const DEFAULT_SEED: i32 = 2026;

/// Browser game runtime. Call frame() once from requestAnimationFrame.
#[wasm_bindgen]
pub struct StreetChavesWebRuntime {
    host: WebFightHost,
    fight: FightSimulation,
    startup: StartupFlow,
    assets: Option<LoadedWebAssets>,
    diagnostics: Vec<String>,
    preferences_dirty: bool,
    campaign: Option<CampaignRuntime>,
    rounds: Option<ArcadeRoundProgress>,
    cpu_opponent: bool,
    player_character: u8,
    settings: crate::arcade::ArcadeSettings,
}

#[wasm_bindgen]
impl StreetChavesWebRuntime {
    /// Create a runtime attached to an existing HTML canvas element ID.
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<StreetChavesWebRuntime, JsValue> {
        let config = RuntimeConfig {
            title: "Street Chaves Rust".to_owned(),
            width: LOGICAL_WIDTH,
            height: LOGICAL_HEIGHT,
            pixel_art: true,
        };
        let browser = foundation_runtime::web::Canvas2dHost::from_canvas_id(canvas_id, config.clone())
            .map_err(js_error)?;
        let host = WebFightHost::with_host(browser, config).map_err(js_error)?;
        Ok(Self {
            host,
            fight: build_fight([1, 4], DEFAULT_SEED),
            startup: StartupFlow::new([1, 4]),
            assets: None,
            diagnostics: vec![
                "WebAssembly runtime initialized".to_owned(),
                "Opening -> Title -> Config/Remap -> Character Select -> Fight".to_owned(),
            ],
            preferences_dirty: false,
            campaign: None,
            rounds: None,
            cpu_opponent: false,
            player_character: 1,
            settings: crate::arcade::ArcadeSettings::default(),
        })
    }

    /// Load the shared Foundation Web export manifest and register its resources.
    /// Asset paths are resolved relative to the manifest URL.
    pub async fn load_assets(&mut self, manifest_url: &str) -> Result<u32, JsValue> {
        let assets = self
            .host
            .host_mut()
            .load_web_export(manifest_url)
            .await
            .map_err(js_error)?;
        let count = assets.textures.len() + assets.audio_clips.len() + assets.data.len();
        self.assets = Some(assets);
        self.diagnostics
            .push(format!("loaded {count} web package resources"));
        Ok(count as u32)
    }

    /// Restore saved arcade preferences from browser localStorage.
    pub fn load_preferences(&mut self, source: &str) {
        if let Some(preferences) = ArcadePreferences::from_json(source) {
            self.host.set_player_one_key_map(preferences.player_one_keys);
            self.host.set_damage_level(preferences.settings.damage_level);
            self.settings = preferences.settings;
            self.startup.set_preferences(preferences);
            self.diagnostics.push("restored arcade preferences".to_owned());
        } else {
            self.diagnostics.push("saved arcade preferences were invalid; using defaults".to_owned());
        }
    }

    /// Return updated preferences once after an F1/F2 change, for browser storage.
    pub fn take_preferences_json(&mut self) -> String {
        if self.preferences_dirty {
            self.preferences_dirty = false;
            self.startup.preferences().to_json()
        } else {
            String::new()
        }
    }

    /// Resume browser audio. Call this directly from a Start button click/tap.
    pub async fn resume_audio(&self) -> Result<(), JsValue> {
        self.host
            .host()
            .resume_audio()
            .await
            .map_err(js_error)
    }

    /// Unlock browser audio and play the embedded original menu track.
    pub fn play_menu_music(&mut self) -> Result<bool, JsValue> {
        self.host.play_web_menu_music().map_err(js_error)
    }

    /// Preview one of the 32 original victory voices after audio unlock.
    pub fn preview_victory_voice(&mut self, character_id: u8, variant: u8) -> Result<bool, JsValue> {
        self.host.play_web_victory_voice(character_id, variant).map_err(js_error)
    }

    /// Preview an F1 sound slot (one-based, zero means no preview).
    pub fn preview_sound_effect(&mut self, slot: u8) -> Result<bool, JsValue> {
        self.host.play_web_sound_effect(slot).map_err(js_error)
    }

    /// Preview the F1 music selector (zero stops music).
    pub fn preview_music_track(&mut self, track: u8) -> Result<bool, JsValue> {
        self.host.play_web_music_track(track).map_err(js_error)
    }

    /// Advance all due 28 ms simulation ticks, then render one presentation frame.
    ///
    /// The page should call this from requestAnimationFrame. The frame rate only
    /// controls presentation; it never changes the logical simulation interval.
    pub fn frame(&mut self) -> Result<(), JsValue> {
        if self.host.should_quit() {
            return Ok(());
        }

        let (_batch, tick_inputs) = self.host.poll_input_ticks().map_err(js_error)?;
        for inputs in tick_inputs {
            match self.startup.tick(inputs) {
                StartupEvent::StartFight { characters, cpu_opponent } => {
                    let settings = self.startup.settings();
                    self.settings = settings;
                    self.player_character = characters[0];
                    self.cpu_opponent = cpu_opponent;
                    self.campaign = None;
                    self.rounds = Some(ArcadeRoundProgress::new(settings));
                    self.fight = build_fight(characters, DEFAULT_SEED);
                    configure_fight(&mut self.fight, cpu_opponent, settings.difficulty, settings.power_mode);
                    if let Some(assets) = self.assets.as_ref() {
                        let messages = self.host.load_web_resources(assets, 1, characters, &mut self.fight);
                        self.diagnostics.extend(messages);
                        if cpu_opponent {
                            let ini_key = format!("fighter_ini_{:02}", characters[0]);
                            if let Some(ini) = assets.data.get(&ini_key).and_then(|bytes| std::str::from_utf8(bytes).ok()) {
                                match CampaignProfile::from_ini(characters[0], ini)
                                    .and_then(|profile| self.fight.generate_campaign_plan(&profile, 15)) {
                                    Ok(plan) => {
                                        self.campaign = Some(CampaignRuntime::new(plan));
                                        self.diagnostics.push("eight-phase campaign initialized from fighter INI".to_owned());
                                    }
                                    Err(error) => self.diagnostics.push(format!("campaign unavailable: {error}")),
                                }
                            } else {
                                self.diagnostics.push(format!("campaign metadata {ini_key} is missing; falling back to single fight"));
                            }
                        }
                        if self.campaign.is_none() {
                            match self.host.play_web_music_track(settings.music_track) {
                                Ok(true) => self.diagnostics.push(format!("fight music track {} started", settings.music_track)),
                                Ok(false) => {}
                                Err(error) => self.diagnostics.push(format!("fight music: {error}")),
                            }
                        }
                    } else {
                        self.diagnostics.push("no asset manifest loaded; using vector training stage and fighters".to_owned());
                    }
                    self.host.set_damage_level(settings.damage_level);
                    self.host.set_player_one_key_map(self.startup.preferences().player_one_keys);
                }
                StartupEvent::JoinPlayer { player: 2 } => {
                    self.campaign = None;
                    self.cpu_opponent = false;
                    self.fight.configure_cpu_opponent(false, self.startup.settings().difficulty);
                    self.diagnostics.push("Player 2 joined; CPU disabled".to_owned());
                }
                StartupEvent::PreferencesChanged => {
                    let preferences = self.startup.preferences();
                    self.host.set_player_one_key_map(preferences.player_one_keys);
                    self.settings = preferences.settings;
                    self.host.set_damage_level(preferences.settings.damage_level);
                    if let Err(error) = self.host.play_web_music_track(preferences.settings.music_track) {
                        self.diagnostics.push(format!("music preview: {error}"));
                    }
                    if preferences.settings.sound_test > 0 {
                        if let Err(error) = self.host.play_web_sound_effect(preferences.settings.sound_test) {
                            self.diagnostics.push(format!("sound preview: {error}"));
                        }
                    }
                    self.preferences_dirty = true;
                    self.diagnostics.push("arcade preferences updated".to_owned());
                }
                StartupEvent::None if self.startup.phase() == StartupPhase::Fighting => {
                    if let Some(campaign) = self.campaign.as_mut() {
                        match campaign.state() {
                            CampaignRuntimeState::Adversaries => {
                                if campaign.tick_adversaries(inputs[0].was_pressed(Button(0))) {
                                    let phase = campaign.current_phase().unwrap_or(1);
                                    let opponent = campaign.current_opponent().unwrap_or(1);
                                    let ini_key = format!("fighter_ini_{opponent:02}");
                                    let prepared = self.assets.as_ref()
                                        .and_then(|assets| assets.data.get(&ini_key))
                                        .and_then(|bytes| std::str::from_utf8(bytes).ok())
                                        .ok_or_else(|| format!("campaign metadata {ini_key} is missing"))
                                        .and_then(CampaignStageProfile::from_ini)
                                        .and_then(|profile| self.fight.choose_campaign_stage(profile, phase, 13, None));
                                    match prepared {
                                        Ok(stage) => {
                                            if let Err(error) = self.fight.load_campaign_opponent(campaign.plan(), &campaign.progress()) {
                                                self.diagnostics.push(format!("campaign opponent: {error}"));
                                            } else {
                                                self.fight.reset_for_campaign_fight();
                                                configure_fight(&mut self.fight, true, self.settings.difficulty, self.settings.power_mode);
                                                if let Some(assets) = self.assets.as_ref() {
                                                    let messages = self.host.load_web_resources(assets, stage.stage, [self.player_character, opponent], &mut self.fight);
                                                    self.diagnostics.extend(messages);
                                                }
                                                match campaign.begin_fight(stage) {
                                                    Ok(_) => {
                                                        self.rounds = Some(ArcadeRoundProgress::new(self.settings));
                                                        let track = self.assets.as_ref().map(|assets| stage_music_number(assets, stage.stage, self.settings.music_track)).unwrap_or(self.settings.music_track);
                                                        match self.host.play_web_music_track(track) {
                                                            Ok(true) => self.diagnostics.push(format!("phase {phase} fight music {track} started")),
                                                            Ok(false) => {}
                                                            Err(error) => self.diagnostics.push(format!("campaign music: {error}")),
                                                        }
                                                    }
                                                    Err(error) => self.diagnostics.push(format!("campaign transition: {error}")),
                                                }
                                            }
                                        }
                                        Err(error) => self.diagnostics.push(format!("campaign stage selection: {error}")),
                                    }
                                }
                            }
                            CampaignRuntimeState::Fighting => {
                                self.host.tick_fight(&mut self.fight, inputs);
                                let timeout = self.rounds.as_mut().is_some_and(ArcadeRoundProgress::tick);
                                let p1 = self.fight.state.fighters[0].health;
                                let p2 = self.fight.state.fighters[1].health;
                                let winner = if p1 <= 0.0 && p2 <= 0.0 { Some(None) }
                                    else if p2 <= 0.0 { Some(Some(0)) }
                                    else if p1 <= 0.0 { Some(Some(1)) }
                                    else if timeout { Some(if p1 > p2 { Some(0) } else if p2 > p1 { Some(1) } else { None }) }
                                    else { None };
                                if let Some(winner) = winner {
                                    let advance = self.rounds.as_mut().map(|rounds| rounds.record_result(winner));
                                    match advance {
                                        Some(RoundAdvance::Continue { .. }) | Some(RoundAdvance::Draw { .. }) => {
                                            self.fight.reset_for_campaign_fight();
                                            configure_fight(&mut self.fight, self.cpu_opponent, self.settings.difficulty, self.settings.power_mode);
                                        }
                                        Some(RoundAdvance::MatchWon { winner: 0 }) => {
                                            if self.campaign.as_ref().is_some_and(|run| run.current_phase() == Some(8)) {
                                                let _ = self.host.play_web_victory_voice(self.player_character, 1);
                                            }
                                            let advance = self.campaign.as_mut().map(CampaignRuntime::record_victory);
                                            if let Some(CampaignAdvance::NextPhase(phase)) = advance {
                                                self.host.stop_web_music();
                                                self.rounds = Some(ArcadeRoundProgress::new(self.settings));
                                                self.diagnostics.push(format!("campaign victory; preparing phase {phase}"));
                                            } else if advance == Some(CampaignAdvance::Completed) {
                                                self.diagnostics.push("campaign completed: eight opponents defeated".to_owned());
                                                self.host.stop_web_music();
                                            }
                                        }
                                        Some(RoundAdvance::MatchWon { winner: 1 }) => {
                                            if let Some(campaign) = self.campaign.as_mut() {
                                                campaign.record_defeat();
                                                self.host.stop_web_music();
                                                self.diagnostics.push("campaign defeated".to_owned());
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            CampaignRuntimeState::Completed | CampaignRuntimeState::Defeated => {}
                        }
                    } else {
                        self.host.tick_fight(&mut self.fight, inputs);
                    }
                }
                StartupEvent::None => {}
                StartupEvent::JoinPlayer { player } => {
                    self.diagnostics.push(format!("unsupported player slot join event: {player}"));
                }
            }
        }

        if self.startup.phase() == StartupPhase::Fighting {
            self.host.update_camera(&mut self.fight);
            if let Some(campaign) = self.campaign.as_ref() {
                if campaign.state() != CampaignRuntimeState::Fighting {
                    self.host.render_campaign(campaign, self.player_character).map_err(js_error)?;
                } else {
                    self.host.render_fight(&self.fight, &[]).map_err(js_error)?;
                }
            } else {
                self.host.render_fight(&self.fight, &[]).map_err(js_error)?;
            }
        } else {
            self.host
                .render_startup(&self.startup.screen())
                .map_err(js_error)?;
        }
        Ok(())
    }


    /// Browser gamepad/touch adapter: directions use the shared 0..=8 keypad order.
    pub fn set_virtual_direction(&mut self, player: usize, direction: u8) -> bool {
        let direction = match direction {
            0 => Direction::Neutral, 1 => Direction::Up, 2 => Direction::UpRight,
            3 => Direction::Right, 4 => Direction::DownRight, 5 => Direction::Down,
            6 => Direction::DownLeft, 7 => Direction::Left, 8 => Direction::UpLeft,
            _ => return false,
        };
        self.host.inject_input_event(player, InputEvent::DirectionChanged(direction))
    }

    /// Browser gamepad/touch adapter: logical action slots 0..=31.
    pub fn set_virtual_button(&mut self, player: usize, button: u8, pressed: bool) -> bool {
        if button >= 32 { return false; }
        self.host.inject_input_event(player, InputEvent::ButtonChanged {
            button: Button(button), down: pressed
        })
    }

    /// Resize the browser canvas backing store for a CSS viewport and device-pixel ratio.
    pub fn resize_canvas(
        &mut self,
        css_width: u32,
        css_height: u32,
        device_pixel_ratio: f64,
    ) -> Result<(), JsValue> {
        self.host
            .host_mut()
            .resize_backing_store(css_width, css_height, device_pixel_ratio)
            .map_err(js_error)
    }

    /// Call when the browser tab becomes visible again after suspension.
    pub fn reset_after_suspend(&mut self) {
        self.host.reset_after_suspend();
    }

    /// Report whether Escape was pressed.
    pub fn should_quit(&self) -> bool {
        self.host.should_quit()
    }

    /// Current startup phase: opening, title, character_select, or fighting.
    pub fn phase(&self) -> String {
        match self.startup.phase() {
            StartupPhase::Opening => "opening",
            StartupPhase::Title => "title",
            StartupPhase::CharacterSelect => "character_select",
            StartupPhase::Fighting => "fighting",
        }
        .to_owned()
    }

    /// Return and clear runtime diagnostics.
    pub fn take_diagnostics(&mut self) -> Array {
        let output = Array::new();
        for message in std::mem::take(&mut self.diagnostics) {
            output.push(&JsValue::from_str(&message));
        }
        output
    }

    /// Return a shallow asset status useful to the surrounding HTML host.
    pub fn assets_loaded(&self) -> bool {
        self.assets.is_some()
    }
}

fn stage_music_number(assets: &LoadedWebAssets, stage: u8, fallback: u8) -> u8 {
    let key = format!("stage_ini_{stage:02}");
    let Some(source) = assets.data.get(&key).and_then(|bytes| std::str::from_utf8(bytes).ok()) else {
        return fallback;
    };
    let mut section = "";
    for line in source.lines().map(str::trim) {
        if line.starts_with('[') && line.ends_with(']') {
            section = &line[1..line.len() - 1];
        } else if section.eq_ignore_ascii_case("Geral") {
            if let Some((key, value)) = line.split_once('=') {
                if key.trim().eq_ignore_ascii_case("Musica") {
                    return value.trim().parse::<u8>().unwrap_or(fallback);
                }
            }
        }
    }
    fallback
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

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
