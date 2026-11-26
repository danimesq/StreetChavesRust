//! StreetChaves consumer of the shared Foundation SDL3 host.
//! Foundation owns the window, events, textures, WAV decoding, device and PCM mixer.
//! StreetChaves owns its two-player input mapping, 28 ms clock, game presentation and assets.

use crate::audio_foundation::{FoundationAudioOutput, PascalAudioLayout};
use crate::combat_runtime::SIDE_LEFT;
use crate::fight_simulation::{FightPresentationFrame, FightSimulation, FighterPresentation};
use crate::fixed_timestep::{FixedTimestepClock, TickBatch};
use crate::input::{Button, Direction, InputEvent, InputSampler};
use foundation_runtime::native_sdl::Sdl3Host;
use foundation_runtime::{
    DrawCommand, FrameOutput, PadDirection, PlayerInputSnapshot, Rect, RuntimeConfig,
    RuntimeError, RuntimeHost, TextureId,
};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

/// Audited sprite-sheet mapping; P.Quadro is never treated as an atlas offset automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteFrameMapping {
    pub texture: TextureId,
    pub source: Rect,
    pub destination_width: u32,
    pub destination_height: u32,
    pub anchor_x: i32,
    pub anchor_y: i32,
}

/// Native Foundation-backed host for the two-player deterministic simulation.
pub struct NativeFightHost {
    host: Sdl3Host,
    clock: FixedTimestepClock,
    input: [InputSampler; 2],
    last_host_time: Duration,
    quit_requested: bool,
    logical_width: u32,
    logical_height: u32,
    sprite_frames: HashMap<(u8, i32), SpriteFrameMapping>,
    audio: FoundationAudioOutput,
    audio_layout: Option<PascalAudioLayout>,
}

impl NativeFightHost {
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
            audio: FoundationAudioOutput::new(),
            audio_layout: None,
        })
    }

    pub fn should_quit(&self) -> bool { self.quit_requested }
    pub fn host(&self) -> &Sdl3Host { &self.host }
    pub fn host_mut(&mut self) -> &mut Sdl3Host { &mut self.host }

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
        if mapping.source.width == 0 || mapping.source.height == 0
            || mapping.destination_width == 0 || mapping.destination_height == 0 {
            return Err(RuntimeError::InvalidConfig("sprite dimensions must be non-zero"));
        }
        self.sprite_frames.insert((character_id, sprite_frame), mapping);
        Ok(())
    }

    pub fn load_pascal_audio(&mut self, game_root: impl AsRef<Path>) -> PascalAudioLayout {
        let layout = self.audio.load_pascal_resources(&mut self.host, game_root);
        self.audio_layout = Some(layout.clone());
        layout
    }

    pub fn audio_layout(&self) -> Option<&PascalAudioLayout> { self.audio_layout.as_ref() }

    /// Drain host input and execute all due 28 ms logical ticks for both fighters.
    pub fn poll_and_tick(&mut self, fight: &mut FightSimulation) -> Result<TickBatch, RuntimeError> {
        let snapshot = self.host.poll_input()?;
        self.quit_requested = snapshot.quit_requested;
        for player in 0..2 {
            apply_player_snapshot(&mut self.input[player], snapshot.players[player]);
        }

        let now = self.host.now();
        let elapsed = now.saturating_sub(self.last_host_time);
        self.last_host_time = now;
        let batch = self.clock.advance(elapsed);

        for _ in 0..batch.ticks_due {
            let frames = std::array::from_fn(|player| self.input[player].sample_tick());
            fight.tick_with_inputs(frames);
            if let Some(layout) = self.audio_layout.as_ref() {
                let _ = self.audio.play_pending_fight_sound(&mut self.host, layout, fight);
            }
        }
        Ok(batch)
    }

    /// Render an immutable fight snapshot. Stage/background commands are supplied by the game.
    pub fn render(&mut self, frame: &FightPresentationFrame, scene: &[DrawCommand]) -> Result<(), RuntimeError> {
        let mut commands = Vec::with_capacity(scene.len() + 12);
        commands.push(DrawCommand::Clear { rgba: [12, 16, 28, 255] });
        commands.extend_from_slice(scene);
        for fighter in &frame.fighters {
            append_fighter_commands(&mut commands, fighter, frame.camera_x, &self.sprite_frames);
        }
        if frame.screen_flash > 0 {
            commands.push(DrawCommand::Rect {
                rect: Rect::new(0, 0, self.logical_width, self.logical_height),
                rgba: [255, 255, 255, frame.screen_flash.saturating_mul(8)],
            });
        }
        self.host.present(&FrameOutput { draw_commands: commands, audio: Vec::new() })
    }

    pub fn render_fight(&mut self, fight: &FightSimulation, scene: &[DrawCommand]) -> Result<(), RuntimeError> {
        self.render(&fight.presentation_frame(), scene)
    }

    pub fn load_fight_music(&mut self, number: usize) -> bool {
        let Some(layout) = self.audio_layout.as_ref() else { return false };
        self.audio.load_fight_music(&mut self.host, layout, number)
    }

    pub fn play_music(&mut self) -> bool {
        let Some(layout) = self.audio_layout.as_ref() else { return false };
        self.audio.play_music(&mut self.host, layout)
    }

    pub fn stop_music(&mut self) -> bool {
        let Some(layout) = self.audio_layout.as_ref() else { return false };
        self.audio.stop_music(&mut self.host, layout)
    }

    pub fn play_victory_voice(&mut self, index: usize) -> bool {
        let Some(layout) = self.audio_layout.as_ref() else { return false };
        self.audio.play_victory_voice(&mut self.host, layout, index)
    }
}

fn apply_player_snapshot(sampler: &mut InputSampler, player: PlayerInputSnapshot) {
    sampler.apply_event(InputEvent::DirectionChanged(map_direction(player.direction)));
    for slot in 0..32 {
        let bit = 1_u32 << slot;
        if player.pressed_buttons & bit != 0 {
            sampler.apply_event(InputEvent::ButtonChanged { button: Button(slot as u8), down: true });
        }
        if player.released_buttons & bit != 0 {
            sampler.apply_event(InputEvent::ButtonChanged { button: Button(slot as u8), down: false });
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
    let y = fighter.y.round() as i32;
    if let Some(mapping) = mappings.get(&(fighter.character_id, fighter.sprite_frame)) {
        commands.push(DrawCommand::Texture {
            texture: mapping.texture,
            source: Some(mapping.source),
            destination: Rect::new(
                x.saturating_sub(mapping.anchor_x),
                y.saturating_sub(mapping.anchor_y),
                mapping.destination_width,
                mapping.destination_height,
            ),
            flip_horizontal: fighter.facing == SIDE_LEFT,
        });
        return;
    }

    // Visible diagnostic silhouette until the relevant original frame is mapped.
    let rgba = if fighter.slot == 0 { [224, 92, 92, 255] } else { [88, 150, 238, 255] };
    commands.push(DrawCommand::Rect {
        rect: Rect::new(x.saturating_sub(8), y.saturating_sub(24), 16, 24),
        rgba,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foundation_direction_is_normalized_to_the_existing_game_input_type() {
        assert_eq!(map_direction(PadDirection::UpLeft), Direction::UpLeft);
        assert_eq!(map_direction(PadDirection::Neutral), Direction::Neutral);
    }

    #[test]
    fn players_keep_special_button_edges_in_separate_samplers() {
        let mut p1 = InputSampler::default();
        let mut p2 = InputSampler::default();
        apply_player_snapshot(&mut p1, PlayerInputSnapshot {
            held_buttons: 1 << 8, pressed_buttons: 1 << 8, ..Default::default()
        });
        apply_player_snapshot(&mut p2, PlayerInputSnapshot {
            held_buttons: 1 << 12, pressed_buttons: 1 << 12, ..Default::default()
        });
        let one = p1.sample_tick();
        let two = p2.sample_tick();
        assert!(one.was_pressed(Button(8)));
        assert!(two.was_pressed(Button(12)));
        assert_eq!(one.held_buttons, 1 << 8);
        assert_eq!(two.held_buttons, 1 << 12);
    }

    #[test]
    fn sprite_frame_lookup_uses_registered_metadata_not_the_frame_number_as_an_offset() {
        let mut map = HashMap::new();
        map.insert((1, 42), SpriteFrameMapping {
            texture: TextureId(7), source: Rect::new(96, 32, 24, 40),
            destination_width: 48, destination_height: 80, anchor_x: 12, anchor_y: 72,
        });
        let fighter = FighterPresentation {
            slot: 0, character_id: 1, sprite_frame: 42, action_id: 1, action_frame: 0,
            facing: SIDE_LEFT, x: 120.0, y: 180.0, health: 100.0, power: 0, invulnerable: false,
        };
        let mut commands = Vec::new();
        append_fighter_commands(&mut commands, &fighter, 20.0, &map);
        assert_eq!(commands, vec![DrawCommand::Texture {
            texture: TextureId(7), source: Some(Rect::new(96, 32, 24, 40)),
            destination: Rect::new(88, 108, 48, 80), flip_horizontal: true,
        }]);
    }
}
