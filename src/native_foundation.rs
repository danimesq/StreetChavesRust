//! StreetChaves native presentation host backed by RustMultiConsoleFoundation.
//! Foundation owns the SDL3 window, input queue, textures, WAV decoding and PCM mixer.
//! This module adapts Pascal resource data and renderer-neutral fight snapshots.

use crate::audio_foundation::{FoundationAudioOutput, PascalAudioLayout};
use crate::combat_runtime::{SIDE_LEFT, SIDE_RIGHT};
use crate::fight_simulation::{FightPresentationFrame, FightSimulation, FighterPresentation};
use crate::fixed_timestep::{FixedTimestepClock, TickBatch};
use crate::input::{Button, Direction, InputEvent, InputSampler};
use crate::startup_flow::{StartupPhase, StartupScreen};
use foundation_runtime::native_sdl::Sdl3Host;
use foundation_runtime::{
    DrawCommand, FrameOutput, PadDirection, PlayerInputSnapshot, Rect, RgbaImage,
    RuntimeConfig, RuntimeError, RuntimeHost, TextureId,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Explicit sprite mapping based on audited source rectangles and pivots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteFrameMapping {
    pub texture: TextureId,
    pub source: Rect,
    pub destination_width: u32,
    pub destination_height: u32,
    pub anchor_x: i32,
    pub anchor_y: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FighterAssetInfo {
    pub character_id: u8,
    pub sprite_height: i32,
    pub walk_speed: f64,
    pub frame_count: usize,
}

/// Native Foundation-backed host for two-player Street Chaves.
pub struct NativeFightHost {
    host: Sdl3Host,
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
    audio: FoundationAudioOutput,
    audio_layout: Option<PascalAudioLayout>,
}

impl NativeFightHost {
    /// Open the shared Foundation SDL3 host and prepare the game's 28 ms clock.
    pub fn new(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        let host = Sdl3Host::new(config.clone())?;
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
            audio: FoundationAudioOutput::new(),
            audio_layout: None,
        })
    }

    pub fn should_quit(&self) -> bool {
        self.quit_requested
    }

    pub fn host(&self) -> &Sdl3Host {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut Sdl3Host {
        &mut self.host
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn take_diagnostics(&mut self) -> Vec<String> {
        std::mem::take(&mut self.diagnostics)
    }

    pub fn load_bmp_texture(&mut self, path: impl AsRef<Path>) -> Result<TextureId, RuntimeError> {
        let image = Sdl3Host::load_bmp(path)?;
        self.host.register_texture(&image)
    }

    pub fn load_png_texture(&mut self, path: impl AsRef<Path>) -> Result<TextureId, RuntimeError> {
        let image = Sdl3Host::load_png(path)?;
        self.host.register_texture(&image)
    }

    pub fn register_sprite_frame(
        &mut self,
        character_id: u8,
        sprite_frame: i32,
        mapping: SpriteFrameMapping,
    ) -> Result<(), RuntimeError> {
        if mapping.source.width == 0
            || mapping.source.height == 0
            || mapping.destination_width == 0
            || mapping.destination_height == 0
        {
            return Err(RuntimeError::InvalidConfig(
                "sprite dimensions must be non-zero",
            ));
        }
        self.sprite_frames
            .insert((character_id, sprite_frame), mapping);
        Ok(())
    }

    /// Load a local original-game stage BMP.
    pub fn load_original_stage(
        &mut self,
        game_root: impl AsRef<Path>,
        stage_number: u8,
    ) -> Result<(), RuntimeError> {
        let root = game_root.as_ref();
        let resource_ini = IniDocument::read(&root.join("Dados/Recursos.ini"));
        let pattern = resource_ini
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

    /// Load a fighter strip using the exact Pascal dark-column frame separators.
    ///
    /// A separator column has RGB components <= 100 at both the top and bottom pixels.
    /// The player's image-list offset is 0 for P1 and 50 for P2, matching the original source.
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
                "original fighter ID must be in the Pascal C01-C15 roster",
            ));
        }

        let root = game_root.as_ref();
        let ini_path = root.join(format!("Dados/Lutadores/C{character_id:02}.ini"));
        let ini = IniDocument::read(&ini_path);
        let sprite_height = ini
            .get("Geral", "AlturaSprite")
            .and_then(|value| value.parse::<i32>().ok())
            .filter(|height| *height > 0)
            .ok_or_else(|| RuntimeError::Host(format!(
                "{} has no valid [Geral] AlturaSprite",
                ini_path.display()
            )))?;
        let walk_speed = ini
            .get("Geral", "Velocidade")
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|speed| speed.is_finite() && *speed > 0.0)
            .ok_or_else(|| RuntimeError::Host(format!(
                "{} has no valid [Geral] Velocidade",
                ini_path.display()
            )))?;

        let resource_ini = IniDocument::read(&root.join("Dados/Recursos.ini"));
        let sheet_pattern = resource_ini
            .get("Lutadores", "Arquivo")
            .unwrap_or("Lutadores/C##.bmp");
        let sheet_path = rooted_path(root, &numbered_path(sheet_pattern, usize::from(character_id)));
        let mut sheet = Sdl3Host::load_bmp(&sheet_path)?;

        let centers = ini
            .section("CentroX")
            .into_iter()
            .filter_map(|(key, value)| {
                Some((key.parse::<i32>().ok()?, value.parse::<i32>().ok()?))
            })
            .collect::<HashMap<_, _>>();

        let offset = if player == 0 { 0 } else { 50 };
        let frames = split_pascal_sprite_sheet(&sheet)?;
        if frames.is_empty() {
            return Err(RuntimeError::Host(format!(
                "{} contains no frames separated by Pascal dark columns",
                sheet_path.display()
            )));
        }

        for (frame_index, (x, width)) in frames.iter().copied().enumerate() {
            let frame_number = i32::try_from(frame_index + 1)
                .map_err(|_| RuntimeError::InvalidConfig("sprite frame count overflow"))?;
            let image = crop_rgba(&sheet, x, width)?;
            let texture = self.host.register_texture(&image)?;
            self.register_sprite_frame(
                character_id,
                offset + frame_number,
                SpriteFrameMapping {
                    texture,
                    source: Rect::new(0, 0, image.width, image.height),
                    destination_width: image.width,
                    destination_height: image.height,
                    anchor_x: centers.get(&frame_number).copied().unwrap_or(0),
                    anchor_y: sprite_height,
                },
            )?;
        }

        self.sprite_heights[player] = sprite_height;
        self.fighter_names[player] = fighter_display_name(character_id).to_owned();
        let fighter = &mut fight.state.fighters[player];
        fighter.speed = walk_speed;
        if fighter.sprite_frame == 0 {
            fighter.sprite_frame = offset + 1;
        }
        if player == 1 {
            fighter.facing = SIDE_LEFT;
        }

        Ok(FighterAssetInfo {
            character_id,
            sprite_height,
            walk_speed,
            frame_count: frames.len(),
        })
    }

    /// Read the game-wide NivelDano setting; the original Pascal default is 0.9.
    pub fn load_original_game_settings(&mut self, game_root: impl AsRef<Path>) {
        let ini = IniDocument::read(&game_root.as_ref().join("Dados/Jogo.ini"));
        self.damage_level = ini
            .get("Jogo", "NivelDano")
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite())
            .unwrap_or(0.9)
            .clamp(0.3, 2.0);
    }

    /// Load Pascal audio banks through Foundation's WAV decoder and shared mixer.
    pub fn load_pascal_audio(&mut self, game_root: impl AsRef<Path>) -> PascalAudioLayout {
        let layout = self.audio.load_pascal_resources(&mut self.host, game_root);
        self.audio_layout = Some(layout.clone());
        layout
    }

    pub fn audio_layout(&self) -> Option<&PascalAudioLayout> {
        self.audio_layout.as_ref()
    }

    /// Load stage, two fighter strips, INI metadata and optionally audio from a local game root.
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

        match self.load_original_stage(root, stage_number) {
            Ok(()) => messages.push(format!("loaded stage {stage_number}")),
            Err(error) => messages.push(format!("stage {stage_number}: {error}")),
        }
        for player in 0..2 {
            match self.load_original_fighter(root, player, character_ids[player], fight) {
                Ok(info) => messages.push(format!(
                    "P{}: loaded C{:02}, {} frames, {} px height",
                    player + 1,
                    info.character_id,
                    info.frame_count,
                    info.sprite_height
                )),
                Err(error) => messages.push(format!("P{} fighter sheet: {error}", player + 1)),
            }
        }

        if load_audio {
            let layout = self.load_pascal_audio(root);
            messages.extend(layout.warnings.iter().cloned());
            if !self.play_music() {
                messages.push("menu music could not be started".to_owned());
            }
        }
        self.diagnostics.extend(messages.iter().cloned());
        messages
    }

    /// Follow the source's horizontal midpoint camera rule and clamp to the loaded stage.
    pub fn update_camera(&self, fight: &mut FightSimulation) {
        let midpoint = (fight.state.fighters[0].x + fight.state.fighters[1].x) / 2.0;
        let desired = midpoint.round() as i32 - self.logical_width as i32 / 2;
        fight.state.camera_x = f64::from(desired.clamp(0, self.camera_max_x));
    }

    /// Poll the host and return one normalized input sample per due 28 ms tick.
    ///
    /// Startup/menu screens and the fight can consume the exact same authoritative input
    /// stream without polling SDL twice or advancing the simulation behind the menu.
    pub fn poll_input_ticks(
        &mut self,
    ) -> Result<(TickBatch, Vec<[crate::input::InputFrame; 2]>), RuntimeError> {
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

    /// Execute one already-sampled logical tick and its source-ordered host consequences.
    pub fn tick_fight(
        &mut self,
        fight: &mut FightSimulation,
        inputs: [crate::input::InputFrame; 2],
    ) {
        fight.tick_with_inputs(inputs);
        if self.sprite_heights.iter().all(|height| *height > 0) {
            if let Err(error) =
                fight.resolve_melee_contacts(self.sprite_heights, self.damage_level, false)
            {
                self.diagnostics
                    .push(format!("melee resolution: {error}"));
            }
        }
        if let Some(layout) = self.audio_layout.as_ref() {
            let _ = self
                .audio
                .play_pending_fight_sound(&mut self.host, layout, fight);
        }
    }

    /// Convenience wrapper for applications that enter directly into a fight.
    pub fn poll_and_tick(
        &mut self,
        fight: &mut FightSimulation,
    ) -> Result<TickBatch, RuntimeError> {
        let (batch, samples) = self.poll_input_ticks()?;
        for inputs in samples {
            self.tick_fight(fight, inputs);
        }
        Ok(batch)
    }

    /// Render the boot intro, title/start screen or character-selection screen.
    pub fn render_startup(&mut self, screen: &StartupScreen) -> Result<(), RuntimeError> {
        let width = self.logical_width;
        let height = self.logical_height;
        let mut commands = Vec::with_capacity(320);
        commands.push(DrawCommand::Clear {
            rgba: [13, 18, 35, 255],
        });

        match screen.phase {
            StartupPhase::Opening => {
                append_training_stage(&mut commands, width, height, f64::from(screen.ticks / 5));
                commands.push(DrawCommand::Rect {
                    rect: Rect::new(0, 0, width, 54),
                    rgba: [14, 22, 46, 175],
                });
                draw_text(&mut commands, 118, 18, "PRESENTS", [245, 230, 190, 255], 2);
                // Three compact story panels echo the original opening's vila-to-logo rhythm.
                let beat = (screen.ticks / 45) % 3;
                for index in 0..3 {
                    let x = 24 + index * 122;
                    let active = index == beat as i32;
                    commands.push(DrawCommand::Rect {
                        rect: Rect::new(x, 72, 108, 82),
                        rgba: if active {
                            [50, 76, 104, 255]
                        } else {
                            [29, 42, 65, 255]
                        },
                    });
                    commands.push(DrawCommand::Rect {
                        rect: Rect::new(x + 8, 82, 92, 52),
                        rgba: [105, 152, 174, 255],
                    });
                    commands.push(DrawCommand::Rect {
                        rect: Rect::new(x + 16, 118, 76, 18),
                        rgba: [115, 90, 65, 255],
                    });
                    if active {
                        draw_text(
                            &mut commands,
                            x + 24,
                            140,
                            match index {
                                0 => "A VILA",
                                1 => "OS VIZINHOS",
                                _ => "O DESAFIO",
                            },
                            [255, 224, 135, 255],
                            1,
                        );
                    }
                }
                draw_text(
                    &mut commands,
                    88,
                    190,
                    "STREET CHAVES",
                    [255, 224, 126, 255],
                    3,
                );
                draw_text(
                    &mut commands,
                    125,
                    224,
                    "O LUTADOR DA VILA",
                    [238, 238, 232, 255],
                    1,
                );
                if (screen.ticks / 14) % 2 == 0 {
                    draw_text(
                        &mut commands,
                        136,
                        264,
                        "PRESS Z OR R TO SKIP",
                        [255, 255, 255, 255],
                        1,
                    );
                }
            }
            StartupPhase::Title => {
                append_training_stage(&mut commands, width, height, 0.0);
                commands.push(DrawCommand::Rect {
                    rect: Rect::new(24, 38, width.saturating_sub(48), 214),
                    rgba: [9, 14, 30, 220],
                });
                commands.push(DrawCommand::Rect {
                    rect: Rect::new(28, 42, width.saturating_sub(56), 206),
                    rgba: [47, 66, 94, 255],
                });
                draw_text(&mut commands, 57, 82, "STREET", [255, 223, 130, 255], 4);
                draw_text(&mut commands, 66, 124, "CHAVES", [255, 244, 211, 255], 4);
                draw_text(
                    &mut commands,
                    105,
                    166,
                    "O LUTADOR DA VILA",
                    [194, 222, 237, 255],
                    1,
                );
                draw_text(
                    &mut commands,
                    100,
                    204,
                    "PRESS Z TO START",
                    [255, 255, 255, 255],
                    2,
                );
                if (screen.ticks / 16) % 2 == 0 {
                    draw_text(
                        &mut commands,
                        90,
                        230,
                        "P2: PRESS R TO JOIN",
                        [185, 225, 250, 255],
                        1,
                    );
                }
            }
            StartupPhase::CharacterSelect => {
                commands.push(DrawCommand::Rect {
                    rect: Rect::new(0, 0, width, height),
                    rgba: [20, 30, 54, 255],
                });
                draw_text(
                    &mut commands,
                    78,
                    12,
                    "SELECT YOUR FIGHTER",
                    [255, 225, 145, 255],
                    2,
                );
                draw_text(
                    &mut commands,
                    24,
                    34,
                    &format!("P1: {}", fighter_display_name(screen.selected[0])),
                    if screen.confirmed[0] {
                        [118, 238, 164, 255]
                    } else {
                        [255, 234, 170, 255]
                    },
                    1,
                );
                let p2_status = if screen.confirmed[1] {
                    "P2 READY"
                } else if screen.p2_active {
                    "P2 SELECTING"
                } else {
                    "P2 DEFAULT"
                };
                draw_text(&mut commands, 270, 34, p2_status, [176, 220, 248, 255], 1);

                const LABELS: [&str; 15] = [
                    "CHAVES", "MADRUGA", "CHIQUINHA", "QUICO", "FLORINDA",
                    "GIRAFAL", "BARRIGA", "BRUXA", "PATY", "NHONHO",
                    "GODINES", "FURTADO", "JAIMINHO", "NEVES", "GLORIA",
                ];
                for index in 0..15 {
                    let column = index % 5;
                    let row = index / 5;
                    let x = 17 + column as i32 * 73;
                    let y = 62 + row as i32 * 56;
                    let character_id = index as u8 + 1;
                    let color = fighter_palette(character_id).0;
                    commands.push(DrawCommand::Rect {
                        rect: Rect::new(x, y, 69, 49),
                        rgba: [38, 48, 68, 255],
                    });
                    commands.push(DrawCommand::Rect {
                        rect: Rect::new(x + 5, y + 5, 18, 31),
                        rgba: color,
                    });
                    draw_text(
                        &mut commands,
                        x + 27,
                        y + 20,
                        LABELS[index],
                        [244, 244, 235, 255],
                        1,
                    );
                    if screen.selected[0] == character_id {
                        outline_rect(&mut commands, x, y, 69, 49, [255, 218, 102, 255]);
                    }
                    if screen.p2_active && screen.selected[1] == character_id {
                        outline_rect(&mut commands, x + 2, y + 2, 65, 45, [96, 206, 255, 255]);
                    }
                }
                draw_text(
                    &mut commands,
                    18,
                    238,
                    "P1 ARROWS + Z CONFIRM",
                    [255, 224, 132, 255],
                    1,
                );
                draw_text(
                    &mut commands,
                    211,
                    238,
                    "P2 WASD + R CONFIRM",
                    [152, 217, 250, 255],
                    1,
                );
                draw_text(
                    &mut commands,
                    58,
                    264,
                    if screen.confirmed[0] && screen.confirmed[1] {
                        "GET READY!"
                    } else {
                        "P2 CAN JOIN BEFORE P1 CONFIRMS"
                    },
                    [232, 236, 242, 255],
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

    /// Render the background, fighters, a vector fallback and the compact fight HUD.
    pub fn render(
        &mut self,
        frame: &FightPresentationFrame,
        scene: &[DrawCommand],
    ) -> Result<(), RuntimeError> {
        let mut commands = Vec::with_capacity(scene.len() + 800);
        commands.push(DrawCommand::Clear {
            rgba: [11, 17, 30, 255],
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
            append_fighter_commands(&mut commands, fighter, frame.camera_x, &self.sprite_frames);
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

    pub fn render_fight(
        &mut self,
        fight: &FightSimulation,
        scene: &[DrawCommand],
    ) -> Result<(), RuntimeError> {
        self.render(&fight.presentation_frame(), scene)
    }

    pub fn load_fight_music(&mut self, number: usize) -> bool {
        let Some(layout) = self.audio_layout.as_ref() else {
            return false;
        };
        self.audio.load_fight_music(&mut self.host, layout, number)
    }

    pub fn play_music(&mut self) -> bool {
        let Some(layout) = self.audio_layout.as_ref() else {
            return false;
        };
        self.audio.play_music(&mut self.host, layout)
    }

    pub fn stop_music(&mut self) -> bool {
        let Some(layout) = self.audio_layout.as_ref() else {
            return false;
        };
        self.audio.stop_music(&mut self.host, layout)
    }

    pub fn play_victory_voice(&mut self, index: usize) -> bool {
        let Some(layout) = self.audio_layout.as_ref() else {
            return false;
        };
        self.audio.play_victory_voice(&mut self.host, layout, index)
    }
}

fn apply_player_snapshot(sampler: &mut InputSampler, player: PlayerInputSnapshot) {
    sampler.apply_event(InputEvent::DirectionChanged(map_direction(player.direction)));
    for slot in 0..32 {
        let bit = 1_u32 << slot;
        if player.pressed_buttons & bit != 0 {
            sampler.apply_event(InputEvent::ButtonChanged {
                button: Button(slot as u8),
                down: true,
            });
        }
        if player.released_buttons & bit != 0 {
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

fn append_fighter_commands(
    commands: &mut Vec<DrawCommand>,
    fighter: &FighterPresentation,
    camera_x: f64,
    mappings: &HashMap<(u8, i32), SpriteFrameMapping>,
) {
    let x = (fighter.x - camera_x).round() as i32;
    let feet_y = fighter.y.round() as i32;
    if let Some(mapping) = mappings.get(&(fighter.character_id, fighter.sprite_frame)) {
        commands.push(DrawCommand::Texture {
            texture: mapping.texture,
            source: Some(mapping.source),
            destination: Rect::new(
                x.saturating_sub(mapping.anchor_x),
                feet_y.saturating_sub(mapping.anchor_y),
                mapping.destination_width,
                mapping.destination_height,
            ),
            flip_horizontal: fighter.facing == SIDE_LEFT,
        });
        return;
    }
    append_vector_fighter(commands, fighter, x, feet_y);
}

fn append_vector_fighter(
    commands: &mut Vec<DrawCommand>,
    fighter: &FighterPresentation,
    x: i32,
    feet_y: i32,
) {
    // Animated pixel-art stand-in while original local sprite sheets are unavailable.
    let (shirt, shorts, accessory) = fighter_palette(fighter.character_id);
    let phase = (fighter.action_frame % 8) as i32;
    let walking = fighter.action_id == 2;
    let punch = matches!(fighter.action_id, 3 | 5 | 8 | 10 | 13 | 15);
    let kick = matches!(fighter.action_id, 4 | 6 | 9 | 11 | 14 | 16);
    let crouching = matches!(fighter.action_id, 7 | 8 | 9 | 10 | 11 | 21 | 22);
    let airborne = fighter.in_air
        || fighter.action_id == 12
        || (13..=16).contains(&fighter.action_id);
    let bob = if walking { phase % 4 / 2 } else { 0 };
    let body_y = feet_y - if airborne { 38 } else if crouching { 23 } else { 34 } + bob;
    let face_dir = if fighter.facing == SIDE_LEFT { -1 } else { 1 };

    commands.push(DrawCommand::Rect {
        rect: Rect::new(x - 18, feet_y - 2, 36, 4),
        rgba: [0, 0, 0, 88],
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x - 8, body_y + 12, 16, if crouching { 12 } else { 17 }),
        rgba: shirt,
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x - 7, body_y + 1, 14, 13),
        rgba: [236, 190, 146, 255],
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x - 8, body_y - 1, 16, 4),
        rgba: accessory,
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x + face_dir * 2, body_y + 6, 2, 2),
        rgba: [24, 28, 36, 255],
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x - 7, body_y + 27, 14, 7),
        rgba: shorts,
    });

    let stride = if walking {
        if phase < 4 { 3 } else { -3 }
    } else {
        0
    };
    if kick {
        let extension = if fighter.action_frame % 4 < 2 { 8 } else { 13 };
        commands.push(DrawCommand::Rect {
            rect: Rect::new(
                x + face_dir * 4,
                feet_y - 12,
                (extension + 1) as u32,
                5,
            ),
            rgba: [30, 34, 44, 255],
        });
    } else {
        commands.push(DrawCommand::Rect {
            rect: Rect::new(x - 6 - stride, feet_y - 10, 5, 10),
            rgba: [35, 39, 52, 255],
        });
        commands.push(DrawCommand::Rect {
            rect: Rect::new(x + 2 + stride, feet_y - 10, 5, 10),
            rgba: [35, 39, 52, 255],
        });
    }

    let arm_y = body_y + 16;
    let arm_length = if punch {
        15
    } else if fighter.defending {
        7
    } else {
        5
    };
    commands.push(DrawCommand::Rect {
        rect: Rect::new(
            x + face_dir * 7,
            arm_y + if phase % 2 == 0 { 0 } else { 1 },
            arm_length as u32,
            5,
        ),
        rgba: shirt,
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x - 10, arm_y + 2, 4, 7),
        rgba: shirt,
    });
}

fn fighter_palette(character_id: u8) -> ([u8; 4], [u8; 4], [u8; 4]) {
    match character_id {
        1 => ([76, 166, 104, 255], [194, 52, 48, 255], [218, 190, 128, 255]),
        2 => ([112, 122, 146, 255], [74, 82, 104, 255], [92, 78, 54, 255]),
        4 => ([210, 66, 56, 255], [46, 90, 170, 255], [242, 226, 184, 255]),
        7 => ([154, 86, 172, 255], [64, 64, 86, 255], [190, 150, 210, 255]),
        _ => ([76, 128, 188, 255], [54, 72, 104, 255], [180, 192, 214, 255]),
    }
}

fn append_training_stage(
    commands: &mut Vec<DrawCommand>,
    width: u32,
    height: u32,
    camera_x: f64,
) {
    let camera = camera_x.round() as i32;
    commands.push(DrawCommand::Rect {
        rect: Rect::new(0, 0, width, height),
        rgba: [105, 166, 206, 255],
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(0, 194, width, height.saturating_sub(194)),
        rgba: [111, 154, 96, 255],
    });
    for index in 0..8 {
        let x = (index * 126 - camera / 3).rem_euclid(width as i32 + 126) - 80;
        let building_height = 36 + (index * 17 % 44);
        commands.push(DrawCommand::Rect {
            rect: Rect::new(x, 194 - building_height, 74, building_height as u32),
            rgba: [76, 104, 116, 255],
        });
        for window in 0..3 {
            commands.push(DrawCommand::Rect {
                rect: Rect::new(x + 10 + window * 20, 166 - building_height / 2, 7, 8),
                rgba: [224, 210, 148, 255],
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
    let bar_width = 142_i32;
    commands.push(DrawCommand::Rect {
        rect: Rect::new(0, 0, width, 43),
        rgba: [10, 13, 21, 205],
    });
    for player in 0..2 {
        let left = if player == 0 {
            12
        } else {
            width as i32 - 12 - bar_width
        };
        let fighter = &frame.fighters[player];
        draw_text(commands, left, 8, &names[player], [244, 238, 217, 255], 1);
        commands.push(DrawCommand::Rect {
            rect: Rect::new(left, 18, bar_width as u32, 9),
            rgba: [46, 48, 58, 255],
        });
        let health = fighter.health.clamp(0.0, 100.0);
        commands.push(DrawCommand::Rect {
            rect: Rect::new(left, 18, (bar_width as f64 * health / 100.0) as u32, 9),
            rgba: if health > 35.0 {
                [70, 196, 116, 255]
            } else {
                [218, 72, 62, 255]
            },
        });
        commands.push(DrawCommand::Rect {
            rect: Rect::new(left, 31, 86, 4),
            rgba: [46, 48, 58, 255],
        });
        let charge = fighter.charge_power.clamp(0.0, 100.0);
        commands.push(DrawCommand::Rect {
            rect: Rect::new(left, 31, (86.0 * charge / 100.0) as u32, 4),
            rgba: [70, 164, 232, 255],
        });
        draw_text(
            commands,
            left + 92,
            29,
            &format!("P{}", fighter.power),
            [170, 205, 235, 255],
            1,
        );
    }

    draw_text(
        commands,
        width as i32 / 2 - 28,
        8,
        "STREET CHAVES",
        [255, 218, 126, 255],
        1,
    );
    draw_text(
        commands,
        width as i32 / 2 - 17,
        23,
        &format!("TICK {}", frame.tick_index),
        [210, 216, 224, 255],
        1,
    );
    commands.push(DrawCommand::Rect {
        rect: Rect::new(0, height.saturating_sub(16), width, 16),
        rgba: [10, 13, 21, 220],
    });
    draw_text(
        commands,
        8,
        height as i32 - 12,
        "P1 ARROWS + Z X C V   P2 WASD + R T Y U   ESC QUIT",
        [224, 228, 236, 255],
        1,
    );
}

fn outline_rect(
    commands: &mut Vec<DrawCommand>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    rgba: [u8; 4],
) {
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x, y, width, 2),
        rgba,
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x, y + height as i32 - 2, width, 2),
        rgba,
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x, y, 2, height),
        rgba,
    });
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x + width as i32 - 2, y, 2, height),
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
    for character in text.chars() {
        let rows = glyph(character.to_ascii_uppercase());
        for (row, pattern) in rows.iter().enumerate() {
            for column in 0..5 {
                if pattern & (1 << (4 - column)) != 0 {
                    commands.push(DrawCommand::Rect {
                        rect: Rect::new(
                            cursor + column as i32 * scale as i32,
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

fn glyph(character: char) -> [u8; 7] {
    match character {
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
        '.' => [0, 0, 0, 0, 0, 12, 12],
        ' ' => [0; 7],
        _ => [31, 17, 5, 4, 4, 0, 4],
    }
}

fn fighter_display_name(character_id: u8) -> &'static str {
    match character_id {
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

fn apply_white_color_key(image: &mut RgbaImage) {
    for pixel in image.pixels.chunks_exact_mut(4) {
        if pixel[0] == 255 && pixel[1] == 255 && pixel[2] == 255 {
            pixel[3] = 0;
        }
    }
}

fn split_pascal_sprite_sheet(image: &RgbaImage) -> Result<Vec<(u32, u32)>, RuntimeError> {
    if image.width == 0 || image.height == 0 {
        return Err(RuntimeError::InvalidConfig("empty fighter sprite sheet"));
    }
    let stride = image.width as usize * 4;
    let is_dark = |x: u32, y: u32| {
        let offset = y as usize * stride + x as usize * 4;
        let pixel = &image.pixels[offset..offset + 3];
        pixel[0] <= 100 && pixel[1] <= 100 && pixel[2] <= 100
    };
    let mut frames = Vec::new();
    let mut start = 0_u32;
    for column in 0..image.width {
        if is_dark(column, 0) && is_dark(column, image.height - 1) {
            if column > start {
                frames.push((start, column - start));
            }
            start = column.saturating_add(1);
        }
    }
    if start < image.width {
        frames.push((start, image.width - start));
    }
    Ok(frames)
}

fn crop_rgba(source: &RgbaImage, x: u32, width: u32) -> Result<RgbaImage, RuntimeError> {
    if width == 0 || x.saturating_add(width) > source.width {
        return Err(RuntimeError::InvalidConfig("sprite frame crop is outside the sheet"));
    }
    let row_bytes = width as usize * 4;
    let source_stride = source.width as usize * 4;
    let mut pixels = vec![0_u8; row_bytes * source.height as usize];
    for row in 0..source.height as usize {
        let start = row * source_stride + x as usize * 4;
        let target = row * row_bytes;
        pixels[target..target + row_bytes]
            .copy_from_slice(&source.pixels[start..start + row_bytes]);
    }
    let mut image = RgbaImage::new(width, source.height, pixels)?;
    apply_white_color_key(&mut image);
    Ok(image)
}

fn rooted_path(root: &Path, resource: &str) -> PathBuf {
    let path = Path::new(resource);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn numbered_path(pattern: &str, number: usize) -> String {
    let Some(start) = pattern.find('#') else {
        return pattern.to_owned();
    };
    let width = pattern[start..]
        .chars()
        .take_while(|character| *character == '#')
        .count();
    let end = start + width;
    format!("{}{:0width$}{}", &pattern[..start], number, &pattern[end..])
}

#[derive(Debug, Default)]
struct IniDocument {
    entries: HashMap<(String, String), String>,
}

impl IniDocument {
    fn read(path: &Path) -> Self {
        fs::read_to_string(path)
            .map(|contents| Self::parse(&contents))
            .unwrap_or_default()
    }

    fn parse(contents: &str) -> Self {
        let mut document = Self::default();
        let mut section = String::new();
        for raw_line in contents.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_ascii_lowercase();
            } else if let Some((key, value)) = line.split_once('=') {
                document.entries.insert(
                    (section.clone(), key.trim().to_ascii_lowercase()),
                    value.trim().to_owned(),
                );
            }
        }
        document
    }

    fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.entries
            .get(&(section.to_ascii_lowercase(), key.to_ascii_lowercase()))
            .map(String::as_str)
    }

    fn section(&self, name: &str) -> Vec<(&str, &str)> {
        let name = name.to_ascii_lowercase();
        self.entries
            .iter()
            .filter(|((section, _), _)| section == &name)
            .map(|((_, key), value)| (key.as_str(), value.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_sheet() -> RgbaImage {
        // 3 frames separated by dark top-and-bottom columns.
        let width = 8;
        let height = 2;
        let mut pixels = vec![0_u8; width * height * 4];
        for y in 0..height {
            for x in 0..width {
                let offset = (y * width + x) * 4;
                let color = if x == 2 || x == 5 {
                    [12, 12, 12, 255]
                } else {
                    [120, 80, 60, 255]
                };
                pixels[offset..offset + 4].copy_from_slice(&color);
            }
        }
        RgbaImage::new(width as u32, height as u32, pixels).unwrap()
    }

    #[test]
    fn pascal_sprite_sheet_splits_on_dark_columns_instead_of_assuming_uniform_cells() {
        assert_eq!(
            split_pascal_sprite_sheet(&sample_sheet()).unwrap(),
            vec![(0, 2), (3, 2), (6, 2)]
        );
    }

    #[test]
    fn white_color_key_only_changes_white_pixels_to_transparent() {
        let mut image = RgbaImage::new(
            2,
            1,
            vec![255, 255, 255, 255, 10, 20, 30, 255],
        )
        .unwrap();
        apply_white_color_key(&mut image);
        assert_eq!(&image.pixels[0..4], &[255, 255, 255, 0]);
        assert_eq!(&image.pixels[4..8], &[10, 20, 30, 255]);
    }

    #[test]
    fn numbered_resource_paths_keep_pascal_zero_padding() {
        assert_eq!(numbered_path("Cenarios/Cenario-##.bmp", 1), "Cenarios/Cenario-01.bmp");
        assert_eq!(numbered_path("Efeitos/Efeito-###.bmp", 7), "Efeitos/Efeito-007.bmp");
    }

    #[test]
    fn hud_has_distinct_player_panels_and_a_bottom_control_strip() {
        let fighter = |slot, character_id| FighterPresentation {
            slot,
            character_id,
            sprite_frame: 1,
            action_id: 1,
            action_frame: 0,
            facing: SIDE_RIGHT,
            x: 100.0,
            y: 300.0,
            health: 75.0,
            power: 1,
            charge_power: 50.0,
            in_air: false,
            defending: false,
            is_kick: false,
            force_strength: 0,
            invulnerable: false,
        };
        let frame = FightPresentationFrame {
            tick_index: 10,
            fighters: [fighter(0, 1), fighter(1, 4)],
            camera_x: 0.0,
            pending_sound: None,
            screen_flash: 0,
            screen_shake: 0,
        };
        let mut commands = Vec::new();
        append_hud(&mut commands, 400, 300, &frame, &["CHAVES".into(), "QUICO".into()]);
        assert!(commands.iter().any(|command| matches!(command, DrawCommand::Rect { rect, .. } if rect.y == 18)));
        assert!(commands.iter().any(|command| matches!(command, DrawCommand::Rect { rect, .. } if rect.y == 284)));
    }
}
