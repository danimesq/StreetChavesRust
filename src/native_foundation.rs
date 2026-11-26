//! Shared Street Chaves presentation host using RustMultiConsoleFoundation.
//!
//! Foundation owns platform input, drawing, texture handles and audio output.
//! This module owns Pascal resource interpretation and maps game snapshots to draw commands.

#[cfg(not(target_arch = "wasm32"))]
use crate::audio_foundation::{FoundationAudioOutput, PascalAudioLayout};
use crate::combat_runtime::SIDE_LEFT;
use crate::fight_simulation::{FightPresentationFrame, FightSimulation, FighterPresentation};
use crate::fixed_timestep::{FixedTimestepClock, TickBatch};
use crate::input::{Button, Direction, InputEvent, InputFrame, InputSampler};
use crate::startup_flow::{StartupPhase, StartupScreen};
#[cfg(not(target_arch = "wasm32"))]
use foundation_runtime::native_sdl::Sdl3Host;
#[cfg(target_arch = "wasm32")]
use foundation_runtime::web::{Canvas2dHost, LoadedWebAssets};
use foundation_runtime::{
    AudioClipId, AudioRequest, DrawCommand, FrameOutput, PadDirection, PlayerInputSnapshot, Rect,
    RgbaImage, RuntimeConfig, RuntimeError, RuntimeHost, TextureId, TextureRegistrar,
};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Sprite source and pivot. The Pascal sprite frame is an index, not an atlas offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteFrameMapping {
    pub texture: TextureId,
    pub source: Rect,
    pub width: u32,
    pub height: u32,
    pub center_x: i32,
    pub sprite_height: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FighterAssetInfo {
    pub character_id: u8,
    pub sprite_height: i32,
    pub walk_speed: f64,
    pub frames: usize,
}

/// Backend-neutral presentation and fixed-step host for the two-player game core.
pub struct FightPresentationHost<H: RuntimeHost + TextureRegistrar> {
    host: H,
    clock: FixedTimestepClock,
    input: [InputSampler; 2],
    last_host_time: Duration,
    quit_requested: bool,
    logical_width: u32,
    logical_height: u32,
    sprite_frames: HashMap<(u8, i32), SpriteFrameMapping>,
    sprite_heights: [i32; 2],
    fighter_names: [String; 2],
    stage_texture: Option<TextureId>,
    stage_size: (u32, u32),
    camera_max_x: i32,
    damage_level: f64,
    diagnostics: Vec<String>,
    #[cfg(not(target_arch = "wasm32"))]
    audio: FoundationAudioOutput,
    #[cfg(not(target_arch = "wasm32"))]
    audio_layout: Option<PascalAudioLayout>,
    #[cfg(target_arch = "wasm32")]
    web_effect_clips: HashMap<u16, AudioClipId>,
    #[cfg(target_arch = "wasm32")]
    web_effect_bank_len: usize,
    #[cfg(target_arch = "wasm32")]
    web_music: Option<AudioClipId>,
}

/// Native public host name retained for existing consumers.
#[cfg(not(target_arch = "wasm32"))]
pub type NativeFightHost = FightPresentationHost<Sdl3Host>;

/// Browser public host name for the same presentation/simulation bridge.
#[cfg(target_arch = "wasm32")]
pub type WebFightHost = FightPresentationHost<Canvas2dHost>;

#[cfg(not(target_arch = "wasm32"))]
impl FightPresentationHost<Sdl3Host> {
    pub fn new(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        let host = Sdl3Host::new(config.clone())?;
        Self::with_host(host, config)
    }
}

impl<H: RuntimeHost + TextureRegistrar> FightPresentationHost<H> {
    /// Construct the shared game host around any Foundation runtime adapter.
    pub fn with_host(host: H, config: RuntimeConfig) -> Result<Self, RuntimeError> {
        if config.width == 0 || config.height == 0 {
            return Err(RuntimeError::InvalidConfig(
                "logical dimensions must be non-zero",
            ));
        }
        let now = host.now();
        Ok(Self {
            host,
            clock: FixedTimestepClock::new(),
            input: std::array::from_fn(|_| InputSampler::default()),
            last_host_time: now,
            quit_requested: false,
            logical_width: config.width,
            logical_height: config.height,
            sprite_frames: HashMap::new(),
            sprite_heights: [0; 2],
            fighter_names: ["PLAYER 1".to_owned(), "PLAYER 2".to_owned()],
            stage_texture: None,
            stage_size: (config.width.saturating_mul(2), config.height),
            camera_max_x: config.width as i32,
            damage_level: 0.9,
            diagnostics: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            audio: FoundationAudioOutput::new(),
            #[cfg(not(target_arch = "wasm32"))]
            audio_layout: None,
            #[cfg(target_arch = "wasm32")]
            web_effect_clips: HashMap::new(),
            #[cfg(target_arch = "wasm32")]
            web_effect_bank_len: 0,
            #[cfg(target_arch = "wasm32")]
            web_music: None,
        })
    }

    pub fn host(&self) -> &H {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut H {
        &mut self.host
    }

    pub fn should_quit(&self) -> bool {
        self.quit_requested
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn take_diagnostics(&mut self) -> Vec<String> {
        std::mem::take(&mut self.diagnostics)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_bmp_texture(&mut self, path: impl AsRef<Path>) -> Result<TextureId, RuntimeError> {
        let image = Sdl3Host::load_bmp(path)?;
        self.host.register_texture(&image)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_png_texture(&mut self, path: impl AsRef<Path>) -> Result<TextureId, RuntimeError> {
        let image = Sdl3Host::load_png(path)?;
        self.host.register_texture(&image)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_original_stage(
        &mut self,
        game_root: impl AsRef<Path>,
        stage_number: u8,
    ) -> Result<(), RuntimeError> {
        let root = game_root.as_ref();
        let resources = IniDocument::read(&root.join("Dados/Recursos.ini"));
        let pattern = resources
            .get("Cenarios", "Arquivo")
            .unwrap_or("Cenarios/Cenario-##.bmp");
        let path = rooted_path(root, &numbered_path(pattern, usize::from(stage_number)));
        let mut image = Sdl3Host::load_bmp(&path)?;
        apply_white_color_key(&mut image);
        let texture = self.host.register_texture(&image)?;
        self.stage_size = (image.width, image.height);
        self.camera_max_x = image.width.saturating_sub(self.logical_width) as i32;
        self.stage_texture = Some(texture);
        Ok(())
    }

    /// Loads a fighter's local Pascal sprite strip and its source INI pivot metadata.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_original_fighter(
        &mut self,
        game_root: impl AsRef<Path>,
        player: usize,
        character_id: u8,
        fight: &mut FightSimulation,
    ) -> Result<FighterAssetInfo, RuntimeError> {
        if player >= 2 {
            return Err(RuntimeError::InvalidConfig("player slot must be 0 or 1"));
        }
        if !(1..=15).contains(&character_id) {
            return Err(RuntimeError::InvalidConfig(
                "original fighter ID must be in 1..=15",
            ));
        }

        let root = game_root.as_ref();
        let ini_path = root.join(format!("Dados/Lutadores/C{character_id:02}.ini"));
        let fighter_ini = IniDocument::read(&ini_path);
        let sprite_height = fighter_ini
            .get("Geral", "AlturaSprite")
            .and_then(|v| v.parse::<i32>().ok())
            .filter(|v| *v > 0)
            .ok_or_else(|| {
                RuntimeError::Host(format!(
                    "{} has no valid [Geral] AlturaSprite",
                    ini_path.display()
                ))
            })?;
        let walk_speed = fighter_ini
            .get("Geral", "Velocidade")
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|v| v.is_finite() && *v > 0.0)
            .ok_or_else(|| {
                RuntimeError::Host(format!(
                    "{} has no valid [Geral] Velocidade",
                    ini_path.display()
                ))
            })?;

        let resources = IniDocument::read(&root.join("Dados/Recursos.ini"));
        let pattern = resources
            .get("Lutadores", "Arquivo")
            .unwrap_or("Lutadores/C##.bmp");
        let sheet_path = rooted_path(root, &numbered_path(pattern, usize::from(character_id)));
        let sheet = Sdl3Host::load_bmp(&sheet_path)?;
        let centers = fighter_ini
            .section("CentroX")
            .into_iter()
            .filter_map(|(key, value)| Some((key.parse::<i32>().ok()?, value.parse::<i32>().ok()?)))
            .collect::<HashMap<_, _>>();
        let frame_slices = split_pascal_sprite_sheet(&sheet)?;
        if frame_slices.is_empty() {
            return Err(RuntimeError::Host(format!(
                "{} has no Pascal-delimited sprite frames",
                sheet_path.display()
            )));
        }

        let list_offset = if player == 0 { 0 } else { 50 };
        for (index, (start_x, frame_width)) in frame_slices.iter().copied().enumerate() {
            let frame_number = i32::try_from(index + 1)
                .map_err(|_| RuntimeError::InvalidConfig("too many fighter sprite frames"))?;
            let mut pixels = crop_rgba(&sheet, start_x, frame_width)?;
            apply_white_color_key(&mut pixels);
            let texture = self.host.register_texture(&pixels)?;
            self.sprite_frames.insert(
                (character_id, list_offset + frame_number),
                SpriteFrameMapping {
                    texture,
                    source: Rect::new(0, 0, pixels.width, pixels.height),
                    width: pixels.width,
                    height: pixels.height,
                    center_x: centers.get(&frame_number).copied().unwrap_or(0),
                    sprite_height,
                },
            );
        }

        self.sprite_heights[player] = sprite_height;
        self.fighter_names[player] = fighter_display_name(character_id).to_owned();
        let fighter = &mut fight.state.fighters[player];
        fighter.speed = walk_speed;
        if fighter.sprite_frame == 0 {
            fighter.sprite_frame = list_offset + 1;
        }
        if player == 1 {
            fighter.facing = SIDE_LEFT;
        }

        Ok(FighterAssetInfo {
            character_id,
            sprite_height,
            walk_speed,
            frames: frame_slices.len(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_original_game_settings(&mut self, game_root: impl AsRef<Path>) {
        let ini = IniDocument::read(&game_root.as_ref().join("Dados/Jogo.ini"));
        self.damage_level = ini
            .get("Jogo", "NivelDano")
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|v| v.is_finite())
            .unwrap_or(0.9)
            .clamp(0.3, 2.0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_pascal_audio(&mut self, game_root: impl AsRef<Path>) -> PascalAudioLayout {
        let layout = self.audio.load_pascal_resources(&mut self.host, game_root);
        self.audio_layout = Some(layout.clone());
        layout
    }

    /// Loads original local assets after character select. Missing assets stay diagnostics.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_original_game(
        &mut self,
        game_root: impl AsRef<Path>,
        stage_number: u8,
        character_ids: [u8; 2],
        fight: &mut FightSimulation,
        load_audio: bool,
    ) -> Vec<String> {
        let root = game_root.as_ref();
        let mut messages = Vec::new();
        self.load_original_game_settings(root);

        if let Err(error) = self.load_original_stage(root, stage_number) {
            messages.push(format!("stage {stage_number}: {error}"));
        } else {
            messages.push(format!("loaded stage {stage_number}"));
        }
        for player in 0..2 {
            match self.load_original_fighter(root, player, character_ids[player], fight) {
                Ok(info) => messages.push(format!(
                    "P{} loaded C{:02}: {} frames, height {}, speed {:.1}",
                    player + 1,
                    info.character_id,
                    info.frames,
                    info.sprite_height,
                    info.walk_speed
                )),
                Err(error) => messages.push(format!("P{} fighter assets: {error}", player + 1)),
            }
        }
        if load_audio {
            let layout = self.load_pascal_audio(root);
            messages.extend(layout.warnings.iter().cloned());
            // Starting the menu track here would be incorrect: this point is already the fight.
            let scenario_ini = IniDocument::read(
                &root.join(format!("Dados/Cenarios/Cenario-{stage_number:02}.ini")),
            );
            if let Some(music) = scenario_ini
                .get("Geral", "Musica")
                .and_then(|v| v.parse::<usize>().ok())
                .filter(|v| *v > 0)
            {
                if self.audio.load_fight_music(&mut self.host, &layout, music) {
                    let _ = self.audio.play_music(&mut self.host, &layout);
                }
            }
        }
        self.diagnostics.extend(messages.iter().cloned());
        messages
    }

    /// Register images, INI metadata, and audio clips from the exported web package.
    ///
    /// Manifest IDs expected by the first web runtime:
    /// - stage_XX: stage image (for example stage_01)
    /// - fighter_XX: fighter sprite strip
    /// - fighter_ini_XX: fighter metadata INI bytes
    /// - game_settings: optional game settings INI bytes
    /// - effect_slot_NNN: clip placed at the matching Pascal sound-bank slot
    /// - fight_music: optional looping fight track
    #[cfg(target_arch = "wasm32")]
    pub fn load_web_resources(
        &mut self,
        assets: &LoadedWebAssets,
        stage_number: u8,
        character_ids: [u8; 2],
        fight: &mut FightSimulation,
    ) -> Vec<String> {
        let mut messages = Vec::new();
        self.sprite_frames.clear();
        self.sprite_heights = [0; 2];
        self.stage_texture = None;

        let stage_key = format!("stage_{stage_number:02}");
        if let Some(source) = assets.images.get(&stage_key) {
            let mut image = source.clone();
            apply_white_color_key(&mut image);
            match self.host.register_texture(&image) {
                Ok(texture) => {
                    self.stage_texture = Some(texture);
                    self.stage_size = (image.width, image.height);
                    self.camera_max_x =
                        image.width.saturating_sub(self.logical_width) as i32;
                    messages.push(format!("loaded web stage {stage_number}"));
                }
                Err(error) => messages.push(format!("stage {stage_number}: {error}")),
            }
        } else {
            messages.push(format!("web asset {stage_key} is missing; using training stage"));
        }

        if let Some(bytes) = assets.data.get("game_settings") {
            if let Ok(text) = std::str::from_utf8(bytes) {
                let ini = IniDocument::parse(text);
                self.damage_level = ini
                    .get("Jogo", "NivelDano")
                    .and_then(|value| value.parse::<f64>().ok())
                    .filter(|value| value.is_finite())
                    .unwrap_or(0.9)
                    .clamp(0.3, 2.0);
            } else {
                messages.push("game_settings is not UTF-8; using default damage level".to_owned());
            }
        }

        for player in 0..2 {
            let character_id = character_ids[player];
            let sheet_key = format!("fighter_{character_id:02}");
            let ini_key = format!("fighter_ini_{character_id:02}");
            let Some(sheet) = assets.images.get(&sheet_key) else {
                messages.push(format!(
                    "web asset {sheet_key} is missing; P{} uses vector fallback",
                    player + 1
                ));
                continue;
            };

            let fighter_ini = assets
                .data
                .get(&ini_key)
                .and_then(|bytes| std::str::from_utf8(bytes).ok())
                .map(IniDocument::parse)
                .unwrap_or_default();
            let sprite_height = fighter_ini
                .get("Geral", "AlturaSprite")
                .and_then(|value| value.parse::<i32>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(160);
            let walk_speed = fighter_ini
                .get("Geral", "Velocidade")
                .and_then(|value| value.parse::<f64>().ok())
                .filter(|value| value.is_finite() && *value > 0.0)
                .unwrap_or(8.0);
            if !assets.data.contains_key(&ini_key) {
                messages.push(format!(
                    "web asset {ini_key} is missing; using default fighter pivots and movement"
                ));
            }

            let centers = fighter_ini
                .section("CentroX")
                .into_iter()
                .filter_map(|(key, value)| {
                    Some((key.parse::<i32>().ok()?, value.parse::<i32>().ok()?))
                })
                .collect::<HashMap<_, _>>();
            let frame_slices = match split_pascal_sprite_sheet(sheet) {
                Ok(frames) if !frames.is_empty() => frames,
                Ok(_) => {
                    messages.push(format!("{sheet_key} contains no delimited frames"));
                    continue;
                }
                Err(error) => {
                    messages.push(format!("{sheet_key}: {error}"));
                    continue;
                }
            };

            let list_offset = if player == 0 { 0 } else { 50 };
            for (index, (start_x, frame_width)) in frame_slices.iter().copied().enumerate() {
                let frame_number = match i32::try_from(index + 1) {
                    Ok(number) => number,
                    Err(_) => {
                        messages.push(format!("{sheet_key} has too many frames"));
                        break;
                    }
                };
                let mut pixels = match crop_rgba(sheet, start_x, frame_width) {
                    Ok(image) => image,
                    Err(error) => {
                        messages.push(format!("{sheet_key} frame {frame_number}: {error}"));
                        continue;
                    }
                };
                apply_white_color_key(&mut pixels);
                match self.host.register_texture(&pixels) {
                    Ok(texture) => {
                        self.sprite_frames.insert(
                            (character_id, list_offset + frame_number),
                            SpriteFrameMapping {
                                texture,
                                source: Rect::new(0, 0, pixels.width, pixels.height),
                                width: pixels.width,
                                height: pixels.height,
                                center_x: centers.get(&frame_number).copied().unwrap_or(0),
                                sprite_height,
                            },
                        );
                    }
                    Err(error) => messages.push(format!(
                        "{sheet_key} frame {frame_number}: {error}"
                    )),
                }
            }

            self.sprite_heights[player] = sprite_height;
            self.fighter_names[player] = fighter_display_name(character_id).to_owned();
            let fighter = &mut fight.state.fighters[player];
            fighter.speed = walk_speed;
            if fighter.sprite_frame == 0 {
                fighter.sprite_frame = list_offset + 1;
            }
            if player == 1 {
                fighter.facing = SIDE_LEFT;
            }
            messages.push(format!(
                "P{} loaded web C{:02}: {} frames, height {}, speed {:.1}",
                player + 1,
                character_id,
                frame_slices.len(),
                sprite_height,
                walk_speed
            ));
        }

        self.web_effect_clips.clear();
        self.web_effect_bank_len = 0;
        for slot in 0_u16..=255 {
            let key = format!("effect_slot_{slot:03}");
            if let Some(clip) = assets.audio_clips.get(&key).copied() {
                self.web_effect_clips.insert(slot, clip);
                self.web_effect_bank_len = usize::from(slot) + 1;
            }
        }
        self.web_music = assets.audio_clips.get("fight_music").copied();
        if self.web_music.is_some() {
            messages.push("loaded web fight music".to_owned());
        }

        self.diagnostics.extend(messages.iter().cloned());
        messages
    }

    /// Start the configured web fight track after the host AudioContext is resumed.
    #[cfg(target_arch = "wasm32")]
    pub fn play_web_music(&mut self) -> Result<bool, RuntimeError> {
        let Some(clip) = self.web_music else {
            return Ok(false);
        };
        self.host.audio(&AudioRequest::PlayClip { clip, looped: true })?;
        Ok(true)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn stop_web_music(&mut self) -> Result<bool, RuntimeError> {
        let Some(clip) = self.web_music else {
            return Ok(false);
        };
        self.host.audio(&AudioRequest::StopClip { clip })?;
        Ok(true)
    }

    /// Poll Foundation once and return the input sample for each due 28 ms tick.
    pub fn poll_input_ticks(&mut self) -> Result<(TickBatch, Vec<[InputFrame; 2]>), RuntimeError> {
        let snapshot = self.host.poll_input()?;
        self.quit_requested = snapshot.quit_requested;
        for player in 0..2 {
            apply_player_snapshot(&mut self.input[player], snapshot.players[player]);
        }
        let now = self.host.now();
        let elapsed = now.saturating_sub(self.last_host_time);
        self.last_host_time = now;
        let batch = self.clock.advance(elapsed);
        let samples = (0..batch.ticks_due)
            .map(|_| std::array::from_fn(|player| self.input[player].sample_tick()))
            .collect();
        Ok((batch, samples))
    }

    /// Execute an already-sampled tick and its native audio/collision consequences.
    pub fn tick_fight(&mut self, fight: &mut FightSimulation, inputs: [InputFrame; 2]) {
        fight.tick_with_inputs(inputs);
        if self.sprite_heights.iter().all(|height| *height > 0) {
            if let Err(error) =
                fight.resolve_melee_contacts(self.sprite_heights, self.damage_level, false)
            {
                self.diagnostics.push(format!("melee resolution: {error}"));
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(layout) = self.audio_layout.as_ref() {
            let _ = self
                .audio
                .play_pending_fight_sound(&mut self.host, layout, fight);
        }

        #[cfg(target_arch = "wasm32")]
        if let Some(sound_id) = fight.state.pending_sound.filter(|id| *id != 0) {
            let slot = sound_id;
            if usize::from(slot) < self.web_effect_bank_len {
                let _ = fight.take_pending_sound();
                if let Some(clip) = self.web_effect_clips.get(&slot).copied() {
                    let _ = self.host.audio(&AudioRequest::PlayClip {
                        clip,
                        looped: false,
                    });
                }
            }
        }
    }

    /// Camera follows the midpoint between the two fighters, as in Pascal L7_CenarioECamera.
    pub fn update_camera(&self, fight: &mut FightSimulation) {
        let midpoint = (fight.state.fighters[0].x + fight.state.fighters[1].x) / 2.0;
        let camera = midpoint.round() as i32 - self.logical_width as i32 / 2;
        fight.state.camera_x = f64::from(camera.clamp(0, self.camera_max_x));
    }

    pub fn render_startup(&mut self, screen: &StartupScreen) -> Result<(), RuntimeError> {
        let mut commands = Vec::with_capacity(500);
        commands.push(DrawCommand::Clear {
            rgba: [12, 17, 32, 255],
        });
        match screen.phase {
            StartupPhase::Opening => {
                append_training_stage(
                    &mut commands,
                    self.logical_width,
                    self.logical_height,
                    (screen.ticks / 4) as f64,
                );
                commands.push(DrawCommand::Rect {
                    rect: Rect::new(0, 0, self.logical_width, 50),
                    rgba: [11, 16, 29, 220],
                });
                draw_text(&mut commands, 112, 15, "PRESENTS", [251, 226, 170, 255], 2);
                let beat = ((screen.ticks / 42) % 3) as i32;
                for index in 0..3 {
                    let x = 22 + index * 122;
                    let active = beat == index;
                    commands.push(DrawCommand::Rect {
                        rect: Rect::new(x, 75, 110, 74),
                        rgba: if active {
                            [62, 94, 119, 255]
                        } else {
                            [35, 54, 78, 255]
                        },
                    });
                    commands.push(DrawCommand::Rect {
                        rect: Rect::new(x + 8, 83, 94, 45),
                        rgba: [113, 160, 178, 255],
                    });
                    commands.push(DrawCommand::Rect {
                        rect: Rect::new(x + 8, 129, 94, 12),
                        rgba: [117, 88, 62, 255],
                    });
                    if active {
                        draw_text(
                            &mut commands,
                            x + 26,
                            154,
                            ["THE VILA", "NEIGHBORS", "THE FIGHT"][index as usize],
                            [255, 229, 156, 255],
                            1,
                        );
                    }
                }
                draw_text(
                    &mut commands,
                    77,
                    204,
                    "STREET CHAVES",
                    [255, 221, 130, 255],
                    3,
                );
                draw_text(
                    &mut commands,
                    124,
                    243,
                    "O LUTADOR DA VILA",
                    [241, 239, 231, 255],
                    1,
                );
                if (screen.ticks / 14) % 2 == 0 {
                    draw_text(
                        &mut commands,
                        126,
                        274,
                        "PRESS Z OR R TO SKIP",
                        [255, 255, 255, 255],
                        1,
                    );
                }
            }
            StartupPhase::Title => {
                append_training_stage(&mut commands, self.logical_width, self.logical_height, 0.0);
                commands.push(DrawCommand::Rect {
                    rect: Rect::new(25, 34, self.logical_width.saturating_sub(50), 224),
                    rgba: [12, 18, 35, 226],
                });
                commands.push(DrawCommand::Rect {
                    rect: Rect::new(29, 38, self.logical_width.saturating_sub(58), 216),
                    rgba: [48, 70, 100, 255],
                });
                draw_text(&mut commands, 65, 82, "STREET", [255, 224, 138, 255], 4);
                draw_text(&mut commands, 73, 127, "CHAVES", [255, 247, 221, 255], 4);
                draw_text(
                    &mut commands,
                    105,
                    174,
                    "O LUTADOR DA VILA",
                    [194, 222, 238, 255],
                    1,
                );
                if (screen.ticks / 16) % 2 == 0 {
                    draw_text(
                        &mut commands,
                        93,
                        212,
                        "PRESS Z TO START",
                        [255, 255, 255, 255],
                        2,
                    );
                }
                draw_text(
                    &mut commands,
                    112,
                    239,
                    "P2: PRESS R TO JOIN",
                    [190, 224, 247, 255],
                    1,
                );
            }
            StartupPhase::CharacterSelect => {
                commands.push(DrawCommand::Rect {
                    rect: Rect::new(0, 0, self.logical_width, self.logical_height),
                    rgba: [18, 29, 51, 255],
                });
                draw_text(
                    &mut commands,
                    76,
                    10,
                    "SELECT FIGHTER",
                    [255, 226, 145, 255],
                    2,
                );
                draw_text(
                    &mut commands,
                    14,
                    32,
                    &format!("P1 {}", fighter_display_name(screen.selected[0])),
                    if screen.confirmed[0] {
                        [117, 239, 162, 255]
                    } else {
                        [255, 233, 171, 255]
                    },
                    1,
                );
                let p2_label = if screen.confirmed[1] {
                    format!("P2 {}", fighter_display_name(screen.selected[1]))
                } else if screen.p2_active {
                    format!("P2 {}", fighter_display_name(screen.selected[1]))
                } else {
                    "P2 PRESS R".to_owned()
                };
                draw_text(&mut commands, 236, 32, &p2_label, [168, 221, 249, 255], 1);

                const NAMES: [&str; 15] = [
                    "CHAVES",
                    "MADRUGA",
                    "CHIQUINHA",
                    "QUICO",
                    "FLORINDA",
                    "GIRAFAL",
                    "BARRIGA",
                    "BRUXA",
                    "PATY",
                    "NHONHO",
                    "GODINES",
                    "FURTADO",
                    "JAIMINHO",
                    "NEVES",
                    "GLORIA",
                ];
                for index in 0..15 {
                    let col = index % 5;
                    let row = index / 5;
                    let x = 10 + col as i32 * 77;
                    let y = 60 + row as i32 * 53;
                    let id = index as u8 + 1;
                    commands.push(DrawCommand::Rect {
                        rect: Rect::new(x, y, 73, 47),
                        rgba: [38, 51, 73, 255],
                    });
                    commands.push(DrawCommand::Rect {
                        rect: Rect::new(x + 6, y + 7, 17, 31),
                        rgba: fighter_palette(id).0,
                    });
                    draw_text(
                        &mut commands,
                        x + 27,
                        y + 20,
                        NAMES[index],
                        [245, 244, 236, 255],
                        1,
                    );
                    if screen.selected[0] == id {
                        outline_rect(&mut commands, x, y, 73, 47, [255, 219, 107, 255]);
                    }
                    if screen.p2_active && screen.selected[1] == id {
                        outline_rect(&mut commands, x + 2, y + 2, 69, 43, [100, 210, 255, 255]);
                    }
                }
                draw_text(
                    &mut commands,
                    14,
                    229,
                    "P1 ARROWS + Z",
                    [255, 225, 140, 255],
                    1,
                );
                draw_text(
                    &mut commands,
                    230,
                    229,
                    "P2 WASD + R",
                    [161, 222, 251, 255],
                    1,
                );
                draw_text(
                    &mut commands,
                    73,
                    264,
                    if screen.confirmed[0] && screen.confirmed[1] {
                        "GET READY"
                    } else {
                        "SELECT AND CONFIRM"
                    },
                    [235, 237, 243, 255],
                    1,
                );
            }
            StartupPhase::Fighting => {}
        }
        self.host.present(&FrameOutput {
            draw_commands: commands,
            audio: Vec::new(),
        })
    }

    pub fn render_fight(
        &mut self,
        fight: &FightSimulation,
        scene: &[DrawCommand],
    ) -> Result<(), RuntimeError> {
        let frame = fight.presentation_frame();
        self.render(&frame, scene)
    }

    pub fn render(
        &mut self,
        frame: &FightPresentationFrame,
        scene: &[DrawCommand],
    ) -> Result<(), RuntimeError> {
        let mut commands = Vec::with_capacity(scene.len() + 800);
        commands.push(DrawCommand::Clear {
            rgba: [12, 18, 30, 255],
        });
        if let Some(texture) = self.stage_texture {
            commands.push(DrawCommand::Texture {
                texture,
                source: None,
                destination: Rect::new(
                    -(frame.camera_x.round() as i32),
                    0,
                    self.stage_size.0,
                    self.stage_size.1,
                ),
                flip_horizontal: false,
            });
        } else {
            append_training_stage(
                &mut commands,
                self.logical_width,
                self.logical_height,
                frame.camera_x,
            );
        }
        commands.extend_from_slice(scene);
        for fighter in &frame.fighters {
            append_fighter(&mut commands, fighter, frame.camera_x, &self.sprite_frames);
        }
        append_hud(
            &mut commands,
            self.logical_width,
            self.logical_height,
            frame,
            &self.fighter_names,
        );
        if frame.screen_flash > 0 {
            commands.push(DrawCommand::Rect {
                rect: Rect::new(0, 0, self.logical_width, self.logical_height),
                rgba: [255, 255, 255, frame.screen_flash.saturating_mul(8)],
            });
        }
        self.host.present(&FrameOutput {
            draw_commands: commands,
            audio: Vec::new(),
        })
    }
}

fn apply_player_snapshot(sampler: &mut InputSampler, snapshot: PlayerInputSnapshot) {
    sampler.apply_event(InputEvent::DirectionChanged(map_direction(
        snapshot.direction,
    )));
    for slot in 0..32 {
        let bit = 1_u32 << slot;
        if snapshot.pressed_buttons & bit != 0 {
            sampler.apply_event(InputEvent::ButtonChanged {
                button: Button(slot as u8),
                down: true,
            });
        }
        if snapshot.released_buttons & bit != 0 {
            sampler.apply_event(InputEvent::ButtonChanged {
                button: Button(slot as u8),
                down: false,
            });
        }
    }
}

fn map_direction(direction: PadDirection) -> Direction {
    match direction {
        PadDirection::Neutral => Direction::Neutral,
        PadDirection::Up => Direction::Up,
        PadDirection::UpRight => Direction::UpRight,
        PadDirection::Right => Direction::Right,
        PadDirection::DownRight => Direction::DownRight,
        PadDirection::Down => Direction::Down,
        PadDirection::DownLeft => Direction::DownLeft,
        PadDirection::Left => Direction::Left,
        PadDirection::UpLeft => Direction::UpLeft,
    }
}

fn append_fighter(
    commands: &mut Vec<DrawCommand>,
    fighter: &FighterPresentation,
    camera_x: f64,
    mappings: &HashMap<(u8, i32), SpriteFrameMapping>,
) {
    let x = (fighter.x - camera_x).round() as i32;
    let feet = fighter.y.round() as i32;
    if let Some(frame) = mappings.get(&(fighter.character_id, fighter.sprite_frame)) {
        commands.push(DrawCommand::Texture {
            texture: frame.texture,
            source: Some(frame.source),
            destination: Rect::new(
                x.saturating_sub(frame.center_x),
                feet.saturating_sub(frame.sprite_height),
                frame.width,
                frame.height,
            ),
            flip_horizontal: fighter.facing == SIDE_LEFT,
        });
    } else {
        append_vector_fighter(commands, fighter, x, feet);
    }
}

fn append_vector_fighter(
    commands: &mut Vec<DrawCommand>,
    fighter: &FighterPresentation,
    x: i32,
    feet: i32,
) {
    let (shirt, shorts, hat) = fighter_palette(fighter.character_id);
    let phase = (fighter.action_frame % 8) as i32;
    let walking = fighter.action_id == 2;
    let punching = matches!(fighter.action_id, 3 | 5 | 8 | 10 | 13 | 15);
    let kicking = matches!(fighter.action_id, 4 | 6 | 9 | 11 | 14 | 16);
    let crouching = matches!(fighter.action_id, 7 | 8 | 9 | 10 | 11 | 21 | 22);
    let airborne =
        fighter.in_air || fighter.action_id == 12 || (13..=16).contains(&fighter.action_id);
    let bob = if walking { phase % 4 / 2 } else { 0 };
    let body_y =
        feet - if airborne {
            43
        } else if crouching {
            27
        } else {
            39
        } + bob;
    let face = if fighter.facing == SIDE_LEFT { -1 } else { 1 };

    commands.push(DrawCommand::Rect {
        rect: Rect::new(x - 18, feet - 2, 36, 4),
        rgba: [0, 0, 0, 96],
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x - 8, body_y + 13, 16, if crouching { 12 } else { 18 }),
        rgba: shirt,
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x - 7, body_y + 1, 14, 13),
        rgba: [236, 190, 146, 255],
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x - 9, body_y - 2, 18, 5),
        rgba: hat,
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x + face * 2, body_y + 6, 2, 2),
        rgba: [25, 27, 34, 255],
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x - 7, body_y + 28, 14, 7),
        rgba: shorts,
    });
    if kicking {
        let extension = if fighter.action_frame % 4 < 2 { 9 } else { 15 };
        commands.push(DrawCommand::Rect {
            rect: Rect::new(
                x + face * 4 - if face < 0 { extension } else { 0 },
                feet - 13,
                extension as u32,
                5,
            ),
            rgba: [32, 35, 44, 255],
        });
    } else {
        let stride = if walking {
            if phase < 4 {
                3
            } else {
                -3
            }
        } else {
            0
        };
        commands.push(DrawCommand::Rect {
            rect: Rect::new(x - 6 - stride, feet - 11, 5, 11),
            rgba: [34, 38, 49, 255],
        });
        commands.push(DrawCommand::Rect {
            rect: Rect::new(x + 2 + stride, feet - 11, 5, 11),
            rgba: [34, 38, 49, 255],
        });
    }
    let arm_length = if punching {
        15
    } else if fighter.defending {
        8
    } else {
        5
    };
    commands.push(DrawCommand::Rect {
        rect: Rect::new(
            x + face * 7 - if face < 0 { arm_length } else { 0 },
            body_y + 18,
            arm_length as u32,
            5,
        ),
        rgba: shirt,
    });
}

fn fighter_palette(id: u8) -> ([u8; 4], [u8; 4], [u8; 4]) {
    match id {
        1 => (
            [76, 166, 104, 255],
            [194, 52, 48, 255],
            [218, 190, 128, 255],
        ),
        2 => ([112, 122, 146, 255], [74, 82, 104, 255], [92, 78, 54, 255]),
        3 => ([236, 126, 154, 255], [76, 86, 118, 255], [91, 52, 76, 255]),
        4 => ([210, 66, 56, 255], [46, 90, 170, 255], [242, 226, 184, 255]),
        7 => ([154, 86, 172, 255], [64, 64, 86, 255], [190, 150, 210, 255]),
        _ => (
            [76, 128, 188, 255],
            [54, 72, 104, 255],
            [180, 192, 214, 255],
        ),
    }
}

fn append_training_stage(commands: &mut Vec<DrawCommand>, width: u32, height: u32, camera_x: f64) {
    let camera = camera_x.round() as i32;
    commands.push(DrawCommand::Rect {
        rect: Rect::new(0, 0, width, height),
        rgba: [105, 166, 206, 255],
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(0, 194, width, height.saturating_sub(194)),
        rgba: [110, 153, 98, 255],
    });
    for index in 0..8 {
        let x = (index * 126 - camera / 3).rem_euclid(width as i32 + 126) - 80;
        let building_height = 36 + (index * 17 % 44);
        commands.push(DrawCommand::Rect {
            rect: Rect::new(x, 194 - building_height, 74, building_height as u32),
            rgba: [77, 105, 118, 255],
        });
        for window in 0..3 {
            commands.push(DrawCommand::Rect {
                rect: Rect::new(x + 10 + window * 20, 163 - building_height / 2, 7, 8),
                rgba: [224, 210, 150, 255],
            });
        }
    }
    commands.push(DrawCommand::Rect {
        rect: Rect::new(0, 194, width, 4),
        rgba: [53, 93, 62, 255],
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(0, 198, width, height.saturating_sub(198)),
        rgba: [142, 112, 78, 255],
    });
    for line in 0..8 {
        commands.push(DrawCommand::Rect {
            rect: Rect::new(0, 210 + line * 11, width, 1),
            rgba: [121, 94, 66, 255],
        });
    }
}

fn append_hud(
    commands: &mut Vec<DrawCommand>,
    width: u32,
    height: u32,
    frame: &FightPresentationFrame,
    names: &[String; 2],
) {
    const LIFE_MAX: f64 = 125.0;
    const CHARGE_MAX: f64 = 102.0;
    let bar_width = 142_i32;
    commands.push(DrawCommand::Rect {
        rect: Rect::new(0, 0, width, 43),
        rgba: [10, 13, 21, 210],
    });
    for player in 0..2 {
        let left = if player == 0 { 12 } else { width as i32 - 154 };
        let fighter = &frame.fighters[player];
        draw_text(commands, left, 7, &names[player], [245, 241, 222, 255], 1);
        commands.push(DrawCommand::Rect {
            rect: Rect::new(left, 18, bar_width as u32, 9),
            rgba: [45, 47, 57, 255],
        });
        let health = fighter.health.clamp(0.0, LIFE_MAX);
        commands.push(DrawCommand::Rect {
            rect: Rect::new(left, 18, (bar_width as f64 * health / LIFE_MAX) as u32, 9),
            rgba: if health > 40.0 {
                [73, 194, 115, 255]
            } else {
                [219, 74, 64, 255]
            },
        });
        commands.push(DrawCommand::Rect {
            rect: Rect::new(left, 31, 86, 4),
            rgba: [45, 47, 57, 255],
        });
        let charge = fighter.charge_power.clamp(0.0, CHARGE_MAX);
        commands.push(DrawCommand::Rect {
            rect: Rect::new(left, 31, (86.0 * charge / CHARGE_MAX) as u32, 4),
            rgba: [70, 164, 232, 255],
        });
        draw_text(
            commands,
            left + 94,
            29,
            &format!("P{}", fighter.power),
            [182, 211, 241, 255],
            1,
        );
    }
    draw_text(
        commands,
        width as i32 / 2 - 28,
        7,
        "STREET CHAVES",
        [255, 222, 130, 255],
        1,
    );
    draw_text(
        commands,
        width as i32 / 2 - 18,
        23,
        &format!("TICK {}", frame.tick_index),
        [213, 219, 229, 255],
        1,
    );
    commands.push(DrawCommand::Rect {
        rect: Rect::new(0, height.saturating_sub(16) as i32, width, 16),
        rgba: [10, 13, 21, 222],
    });
    draw_text(
        commands,
        5,
        height as i32 - 12,
        "P1 ARROWS Z X C V   P2 WASD R T Y U   ESC QUIT",
        [225, 230, 238, 255],
        1,
    );
}

fn outline_rect(commands: &mut Vec<DrawCommand>, x: i32, y: i32, w: u32, h: u32, rgba: [u8; 4]) {
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x, y, w, 2),
        rgba,
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x, y + h as i32 - 2, w, 2),
        rgba,
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x, y, 2, h),
        rgba,
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x + w as i32 - 2, y, 2, h),
        rgba,
    });
}

fn draw_text(
    commands: &mut Vec<DrawCommand>,
    x: i32,
    y: i32,
    text: &str,
    rgba: [u8; 4],
    scale: u32,
) {
    let mut cursor = x;
    for ch in text.chars() {
        for (row, bits) in glyph(ch.to_ascii_uppercase()).iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    commands.push(DrawCommand::Rect {
                        rect: Rect::new(
                            cursor + col as i32 * scale as i32,
                            y + row as i32 * scale as i32,
                            scale,
                            scale,
                        ),
                        rgba,
                    });
                }
            }
        }
        cursor += 6 * scale as i32;
    }
}

fn glyph(ch: char) -> [u8; 7] {
    match ch {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 25, 21, 19, 19, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        ' ' => [0; 7],
        _ => [31, 17, 5, 4, 4, 0, 4],
    }
}

fn apply_white_color_key(image: &mut RgbaImage) {
    for pixel in image.pixels.chunks_exact_mut(4) {
        if pixel[0] == 255 && pixel[1] == 255 && pixel[2] == 255 {
            pixel[3] = 0;
        }
    }
}

fn split_pascal_sprite_sheet(image: &RgbaImage) -> Result<Vec<(u32, u32)>, RuntimeError> {
    if image.width == 0 || image.height == 0 {
        return Err(RuntimeError::InvalidConfig("empty sprite sheet"));
    }
    let stride = image.width as usize * 4;
    let dark = |x: u32, y: u32| {
        let offset = y as usize * stride + x as usize * 4;
        let pixel = &image.pixels[offset..offset + 3];
        pixel[0] <= 100 && pixel[1] <= 100 && pixel[2] <= 100
    };
    let mut frames = Vec::new();
    let mut start = 0_u32;
    for x in 0..image.width {
        if dark(x, 0) && dark(x, image.height - 1) {
            if x > start {
                frames.push((start, x - start));
            }
            start = x + 1;
        }
    }
    if start < image.width {
        frames.push((start, image.width - start));
    }
    Ok(frames)
}

fn crop_rgba(source: &RgbaImage, x: u32, width: u32) -> Result<RgbaImage, RuntimeError> {
    if width == 0 || x.saturating_add(width) > source.width {
        return Err(RuntimeError::InvalidConfig("sprite crop outside sheet"));
    }
    let output_stride = width as usize * 4;
    let input_stride = source.width as usize * 4;
    let mut pixels = vec![0; output_stride * source.height as usize];
    for row in 0..source.height as usize {
        let start = row * input_stride + x as usize * 4;
        let target = row * output_stride;
        pixels[target..target + output_stride]
            .copy_from_slice(&source.pixels[start..start + output_stride]);
    }
    RgbaImage::new(width, source.height, pixels)
}

#[cfg(not(target_arch = "wasm32"))]
fn rooted_path(root: &Path, name: &str) -> PathBuf {
    let path = Path::new(name);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn numbered_path(pattern: &str, number: usize) -> String {
    let Some(start) = pattern.find('#') else {
        return pattern.to_owned();
    };
    let width = pattern[start..].chars().take_while(|c| *c == '#').count();
    let end = start + width;
    format!("{}{:0width$}{}", &pattern[..start], number, &pattern[end..])
}

#[derive(Default)]
struct IniDocument {
    values: HashMap<(String, String), String>,
}

impl IniDocument {
    #[cfg(not(target_arch = "wasm32"))]
    fn read(path: &Path) -> Self {
        fs::read_to_string(path)
            .map(|contents| Self::parse(&contents))
            .unwrap_or_default()
    }

    fn parse(contents: &str) -> Self {
        let mut parsed = Self::default();
        let mut section = String::new();
        for raw in contents.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_ascii_lowercase();
            } else if let Some((key, value)) = line.split_once('=') {
                parsed.values.insert(
                    (section.clone(), key.trim().to_ascii_lowercase()),
                    value.trim().to_owned(),
                );
            }
        }
        parsed
    }

    fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.values
            .get(&(section.to_ascii_lowercase(), key.to_ascii_lowercase()))
            .map(String::as_str)
    }

    fn section(&self, section: &str) -> Vec<(&str, &str)> {
        let section = section.to_ascii_lowercase();
        self.values
            .iter()
            .filter(|((name, _), _)| name == &section)
            .map(|((_, key), value)| (key.as_str(), value.as_str()))
            .collect()
    }
}

fn fighter_display_name(id: u8) -> &'static str {
    match id {
        1 => "CHAVES",
        2 => "SEU MADRUGA",
        3 => "CHIQUINHA",
        4 => "QUICO",
        5 => "DONA FLORINDA",
        6 => "PROF GIRAFAL",
        7 => "SR BARRIGA",
        8 => "BRUXA",
        9 => "PATY",
        10 => "NHONHO",
        11 => "GODINES",
        12 => "SR FURTADO",
        13 => "JAIMINHO",
        14 => "DONA NEVES",
        15 => "GLORIA",
        _ => "CUSTOM",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_patterns_keep_pascal_hash_width() {
        assert_eq!(
            numbered_path("Cenarios/Cenario-##.bmp", 1),
            "Cenarios/Cenario-01.bmp"
        );
        assert_eq!(
            numbered_path("Efeitos/Efeito-###.bmp", 7),
            "Efeitos/Efeito-007.bmp"
        );
    }

    #[test]
    fn sprite_strip_uses_top_and_bottom_dark_columns() {
        let width = 8_usize;
        let height = 2_usize;
        let mut pixels = vec![0_u8; width * height * 4];
        for y in 0..height {
            for x in 0..width {
                let offset = (y * width + x) * 4;
                let color = if [2, 5, 7].contains(&x) {
                    [10, 10, 10, 255]
                } else {
                    [180, 120, 80, 255]
                };
                pixels[offset..offset + 4].copy_from_slice(&color);
            }
        }
        let image = RgbaImage::new(width as u32, height as u32, pixels).unwrap();
        assert_eq!(
            split_pascal_sprite_sheet(&image).unwrap(),
            vec![(0, 2), (3, 2), (6, 1)]
        );
    }

    #[test]
    fn color_key_only_makes_pure_white_transparent() {
        let mut image = RgbaImage::new(2, 1, vec![255, 255, 255, 255, 10, 20, 30, 255]).unwrap();
        apply_white_color_key(&mut image);
        assert_eq!(&image.pixels[0..4], &[255, 255, 255, 0]);
        assert_eq!(&image.pixels[4..8], &[10, 20, 30, 255]);
    }
}
