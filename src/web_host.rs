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
use js_sys::{Array, Object, Reflect};
use crate::custom_character::{CustomCharacterPackage, PackageFile};
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
    standalone_winner: Option<usize>,
    custom_packages: Vec<CustomCharacterPackage>,
    custom_selections: [Option<String>; 2],
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
            standalone_winner: None,
            custom_packages: Vec::new(),
            custom_selections: [None, None],
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
        self.diagnostics.extend(self.host.load_web_audio_resources(&assets));
        let (packages, package_messages) = discover_custom_character_packages(&assets);
        self.diagnostics.extend(package_messages);
        self.custom_packages = packages;
        for package in self.custom_packages.clone() {
            match self.fight.register_custom_character_package(package.clone()) {
                Ok(runtime_id) => self.diagnostics.push(format!(
                    "registered Web custom fighter {} as runtime ID {runtime_id}",
                    package.manifest.id
                )),
                Err(error) => self.diagnostics.push(format!(
                    "custom fighter {}: {error}", package.manifest.id
                )),
            }
        }
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

    /// Return discovered packages for the optional browser roster selector.
    pub fn list_custom_characters(&self) -> Result<Array, JsValue> {
        let output = Array::new();
        for package in &self.custom_packages {
            // Metadata-only examples are packaged and registered, but not advertised
            // as selectable until a valid sprite image is present in the Web manifest.
            if self.custom_sprite_image_key(&package.manifest.id).is_none() {
                continue;
            }
            let item = Object::new();
            Reflect::set(&item, &JsValue::from_str("id"), &JsValue::from_str(&package.manifest.id))?;
            Reflect::set(&item, &JsValue::from_str("name"), &JsValue::from_str(&package.manifest.display_name))?;
            Reflect::set(&item, &JsValue::from_str("version"), &JsValue::from_str(&package.manifest.version))?;
            let runtime_id = package.manifest.fighter.get("runtime_id").and_then(serde_json::Value::as_u64).unwrap_or(0);
            Reflect::set(&item, &JsValue::from_str("runtime_id"), &JsValue::from_f64(runtime_id as f64))?;
            output.push(&item);
        }
        Ok(output)
    }

    /// Select a manifest-installed custom fighter for P1 or P2.
    /// The selection is retained across the next character-select transition.
    pub fn select_custom_character(&mut self, player: usize, package_id: &str) -> Result<u8, JsValue> {
        if player >= 2 {
            return Err(JsValue::from_str("player slot must be 0 or 1"));
        }
        let package = self.custom_packages.iter()
            .find(|package| package.manifest.id == package_id)
            .ok_or_else(|| JsValue::from_str("custom character package was not found in the Web manifest"))?;
        let runtime_id = package.manifest.fighter.get("runtime_id")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u8::try_from(value).ok())
            .ok_or_else(|| JsValue::from_str("custom fighter manifest has no valid runtime_id"))?;
        self.custom_sprite_image_key(package_id)
            .ok_or_else(|| JsValue::from_str("custom package has no supported fighter sprite image in the Web manifest"))?;
        if self.fight.state.fighters[player].character_id != runtime_id {
            self.fight.select_custom_character(player, package_id).map_err(js_error)?;
        }
        self.register_selected_custom_sprite(player, package_id, runtime_id)?;
        self.custom_selections[player] = Some(package_id.to_owned());
        if player == 0 { self.player_character = runtime_id; }
        Ok(runtime_id)
    }

    fn custom_sprite_image_key(&self, package_id: &str) -> Option<String> {
        let package = self.custom_packages.iter()
            .find(|package| package.manifest.id == package_id)?;
        let assets = self.assets.as_ref()?;
        let preferred = package.manifest.assets.get("fighter_strip")
            .map(|path| ("fighter_strip", path));
        let candidates = preferred.into_iter().chain(
            package.manifest.assets.iter()
                .filter(|(name, _)| name.as_str() != "fighter_strip")
                .map(|(name, path)| (name.as_str(), path)),
        );
        for (name, path) in candidates {
            let lower = path.to_ascii_lowercase();
            let is_image = [".png", ".bmp", ".jpg", ".jpeg", ".webp"]
                .iter().any(|extension| lower.ends_with(extension));
            if !is_image { continue; }
            let key = format!(
                "custom_package_{}_image_{}",
                web_resource_slug(package_id),
                web_resource_slug(name)
            );
            if assets.images.contains_key(&key) { return Some(key); }
        }
        None
    }

    fn register_selected_custom_sprite(
        &mut self,
        player: usize,
        package_id: &str,
        runtime_id: u8,
    ) -> Result<(), JsValue> {
        let image_key = self.custom_sprite_image_key(package_id)
            .ok_or_else(|| JsValue::from_str("custom package sprite image is missing from the Web manifest"))?;
        let image = self.assets.as_ref()
            .and_then(|assets| assets.images.get(&image_key))
            .cloned()
            .ok_or_else(|| JsValue::from_str(&format!("custom sprite image {image_key} is missing")))?;
        self.host.register_custom_fighter_textures(player, runtime_id, &image, &self.fight)
            .map_err(js_error)?;
        Ok(())
    }

    fn reapply_custom_selections(&mut self) {
        for package in self.custom_packages.clone() {
            if let Err(error) = self.fight.register_custom_character_package(package.clone()) {
                self.diagnostics.push(format!("custom fighter {}: {error}", package.manifest.id));
            }
        }
        for player in 0..2 {
            let Some(package_id) = self.custom_selections[player].clone() else { continue; };
            match self.fight.select_custom_character(player, &package_id) {
                Ok(runtime_id) => {
                    if let Err(error) = self.register_selected_custom_sprite(player, &package_id, runtime_id) {
                        self.diagnostics.push(error.as_string().unwrap_or_else(|| "custom sprite registration failed".to_owned()));
                    }
                }
                Err(error) => self.diagnostics.push(format!("custom fighter selection: {error}")),
            }
        }
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
                    self.standalone_winner = None;
                    self.rounds = Some(ArcadeRoundProgress::new(settings));
                    self.fight = build_fight(characters, DEFAULT_SEED);
                    for package in self.custom_packages.clone() {
                        if let Err(error) = self.fight.register_custom_character_package(package.clone()) {
                            self.diagnostics.push(format!("custom fighter {}: {error}", package.manifest.id));
                        }
                    }
                    configure_fight(&mut self.fight, cpu_opponent, settings.difficulty, settings.power_mode);
                    if let Some(assets) = self.assets.as_ref() {
                        let messages = self.host.load_web_resources(assets, 1, characters, &mut self.fight);
                        self.diagnostics.extend(messages);
                        for player in 0..2 {
                            if let Some(package_id) = self.custom_selections[player].clone() {
                                if let Err(error) = self.fight.select_custom_character(player, &package_id) {
                                    self.diagnostics.push(format!("custom fighter selection: {error}"));
                                } else if let Some(runtime_id) = self.fight.state.fighters.get(player).map(|fighter| fighter.character_id) {
                                    if player == 0 { self.player_character = runtime_id; }
                                    if let Err(error) = self.register_selected_custom_sprite(player, &package_id, runtime_id) {
                                        self.diagnostics.push(error.as_string().unwrap_or_else(|| "custom sprite registration failed".to_owned()));
                                    }
                                }
                            }
                        }
                        if cpu_opponent && self.custom_selections[0].is_none() {
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
                        if self.campaign.is_some() {
                            if let Err(error) = self.host.stop_web_music() {
                                self.diagnostics.push(format!("menu music stop: {error}"));
                            }
                        } else {
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
                                                        let track = self.settings.music_track;
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
                    } else if self.standalone_winner.is_none() {
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
                            let result = self.rounds.as_mut().map(|progress| progress.record_result(winner));
                            match result {
                                Some(RoundAdvance::Continue { .. }) | Some(RoundAdvance::Draw { .. }) => {
                                    self.fight.reset_for_campaign_fight();
                                    configure_fight(&mut self.fight, self.cpu_opponent, self.settings.difficulty, self.settings.power_mode);
                                }
                                Some(RoundAdvance::MatchWon { winner }) => {
                                    self.standalone_winner = Some(winner);
                                    let _ = self.host.stop_web_music();
                                    if winner == 0 {
                                        let _ = self.host.play_web_victory_voice(self.player_character, 1);
                                    }
                                    self.diagnostics.push(format!("match complete: player {} wins", winner + 1));
                                }
                                _ => {}
                            }
                        }
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
            } else if let Some(winner) = self.standalone_winner {
                self.host.render_match_result(winner).map_err(js_error)?;
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


fn web_resource_slug(value: &str) -> String {
    let mut output = String::new();
    let mut separator = false;
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() {
            output.push((byte as char).to_ascii_lowercase());
            separator = false;
        } else if !separator {
            output.push('_');
            separator = true;
        }
    }
    output.trim_matches('_').to_owned()
}

fn discover_custom_character_packages(
    assets: &LoadedWebAssets,
) -> (Vec<CustomCharacterPackage>, Vec<String>) {
    let mut packages = Vec::new();
    let mut messages = Vec::new();
    for (manifest_key, manifest_bytes) in &assets.data {
        let Some(slug) = manifest_key.strip_prefix("custom_package_")
            .and_then(|key| key.strip_suffix("_manifest")) else { continue; };
        let manifest: serde_json::Value = match serde_json::from_slice(manifest_bytes) {
            Ok(value) => value,
            Err(error) => {
                messages.push(format!("custom manifest {manifest_key}: invalid JSON: {error}"));
                continue;
            }
        };
        let Some(asset_map) = manifest.get("assets").and_then(serde_json::Value::as_object) else {
            messages.push(format!("custom manifest {manifest_key}: assets must be an object"));
            continue;
        };
        let mut files = vec![PackageFile { path: "character.json".to_owned(), bytes: manifest_bytes.clone() }];
        let mut valid = true;
        for (asset_name, path_value) in asset_map {
            let Some(path) = path_value.as_str() else { valid = false; break; };
            let asset_slug = web_resource_slug(asset_name);
            let bytes_key = format!("custom_package_{slug}_bytes_{asset_slug}");
            let Some(bytes) = assets.data.get(&bytes_key) else {
                messages.push(format!("custom manifest {manifest_key}: missing byte resource {bytes_key}"));
                valid = false;
                break;
            };
            files.push(PackageFile { path: path.to_owned(), bytes: bytes.clone() });
        }
        if !valid { continue; }
        match CustomCharacterPackage::from_files(files) {
            Ok(package) => {
                messages.push(format!("discovered custom fighter {} ({})", package.manifest.display_name, package.manifest.id));
                packages.push(package);
            }
            Err(error) => messages.push(format!("custom manifest {manifest_key}: {error}")),
        }
    }
    packages.sort_by(|a,b| a.manifest.id.cmp(&b.manifest.id));
    (packages, messages)
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
