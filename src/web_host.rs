//! WebAssembly entry point for the Street Chaves browser runtime.
//!
//! JavaScript owns the HTML page and requestAnimationFrame callback. The game
//! simulation, startup flow, fixed timestep, presentation composition and audio
//! events remain in Rust; Foundation owns Canvas 2D, input and WebAudio.

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
                "Opening -> Start -> Character Select -> Fight".to_owned(),
            ],
        })
    }

    /// Fetch the Construct-style web asset manifest and register its resources
    /// with Foundation. Asset paths are resolved relative to the manifest URL.
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

    /// Resume browser audio. Call this directly from a Start button click/tap.
    pub async fn resume_audio(&self) -> Result<(), JsValue> {
        self.host
            .host()
            .resume_audio()
            .await
            .map_err(js_error)
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
                StartupEvent::StartFight { characters } => {
                    self.fight = build_fight(characters, DEFAULT_SEED);
                    if let Some(assets) = self.assets.as_ref() {
                        let messages = self
                            .host
                            .load_web_resources(assets, 1, characters, &mut self.fight);
                        self.diagnostics.extend(messages);
                        match self.host.play_web_music() {
                            Ok(true) => self.diagnostics.push("fight music started".to_owned()),
                            Ok(false) => {}
                            Err(error) => self.diagnostics.push(format!("fight music: {error}")),
                        }
                    } else {
                        self.diagnostics.push(
                            "no asset manifest loaded; using vector training stage and fighters"
                                .to_owned(),
                        );
                    }
                }
                StartupEvent::None if self.startup.phase() == StartupPhase::Fighting => {
                    self.host.tick_fight(&mut self.fight, inputs);
                }
                StartupEvent::None => {}
            }
        }

        if self.startup.phase() == StartupPhase::Fighting {
            self.host.update_camera(&mut self.fight);
            self.host
                .render_fight(&self.fight, &[])
                .map_err(js_error)?;
        } else {
            self.host
                .render_startup(&self.startup.screen())
                .map_err(js_error)?;
        }
        Ok(())
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
