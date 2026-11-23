//! Platform-neutral input sampling primitives for native SDL and browser hosts.
//!
//! This module intentionally knows no scancodes, SDL types, browser key names,
//! controller IDs, or game-specific command combinations. Host adapters normalize
//! physical events to logical directions/buttons before submitting them here.
//! Command recognition remains a separate, source-audited simulation concern.

/// Logical direction emitted by a platform adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Neutral,
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

/// A logical button slot. Host mappings should be configured outside simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Button(pub u8);

/// Normalized event delivered by a host adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    DirectionChanged(Direction),
    ButtonChanged {
        button: Button,
        down: bool,
    },
    /// Clear held state when focus is lost or a device disconnects.
    Reset,
}

/// Immutable input sample for one logical simulation tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InputFrame {
    pub direction: Direction,
    pub held_buttons: u32,
    pub pressed_buttons: u32,
    pub released_buttons: u32,
    /// Saturating per-slot hold counters. A zero value means not held.
    pub hold_ticks: [u16; 32],
}

impl InputFrame {
    /// Query a button slot without exposing mask arithmetic to consumers.
    pub fn is_held(self, button: Button) -> bool {
        button.0 < 32 && (self.held_buttons & (1u32 << button.0)) != 0
    }

    pub fn was_pressed(self, button: Button) -> bool {
        button.0 < 32 && (self.pressed_buttons & (1u32 << button.0)) != 0
    }

    pub fn was_released(self, button: Button) -> bool {
        button.0 < 32 && (self.released_buttons & (1u32 << button.0)) != 0
    }

    pub fn held_for(self, button: Button) -> u16 {
        self.hold_ticks.get(button.0 as usize).copied().unwrap_or(0)
    }
}

/// Stateful host-to-simulation sampler.
///
/// Events update physical-to-logical state between ticks. sample_tick publishes
/// edge flags exactly once and advances hold counters once per simulation tick,
/// independent of render FPS or OS key-repeat events.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputSampler {
    direction: Direction,
    held_buttons: u32,
    hold_ticks: [u16; 32],
    pending_pressed: u32,
    pending_released: u32,
}

impl InputSampler {
    /// Apply normalized input from keyboard, gamepad, or another host.
    pub fn apply_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::DirectionChanged(direction) => self.direction = direction,
            InputEvent::ButtonChanged { button, down } if button.0 < 32 => {
                let bit = 1u32 << button.0;
                let was_down = self.held_buttons & bit != 0;
                if down && !was_down {
                    self.held_buttons |= bit;
                    self.pending_pressed |= bit;
                } else if !down && was_down {
                    self.held_buttons &= !bit;
                    self.hold_ticks[button.0 as usize] = 0;
                    self.pending_released |= bit;
                }
                // Duplicate down/up events are ignored, including OS repeat.
            }
            InputEvent::ButtonChanged { .. } => {}
            InputEvent::Reset => self.reset(),
        }
    }

    /// Produce the state consumed by one simulation tick.
    pub fn sample_tick(&mut self) -> InputFrame {
        for slot in 0..32 {
            let bit = 1u32 << slot;
            if self.held_buttons & bit != 0 {
                self.hold_ticks[slot] = self.hold_ticks[slot].saturating_add(1);
            }
        }

        let frame = InputFrame {
            direction: self.direction,
            held_buttons: self.held_buttons,
            pressed_buttons: self.pending_pressed,
            released_buttons: self.pending_released,
            hold_ticks: self.hold_ticks,
        };
        self.pending_pressed = 0;
        self.pending_released = 0;
        frame
    }

    /// Clear all state to prevent stuck input after focus loss/disconnect.
    pub fn reset(&mut self) {
        self.direction = Direction::Neutral;
        self.held_buttons = 0;
        self.hold_ticks = [0; 32];
        self.pending_pressed = 0;
        self.pending_released = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_are_emitted_once_and_hold_counter_advances_per_tick() {
        let mut sampler = InputSampler::default();
        let attack = Button(2);
        sampler.apply_event(InputEvent::ButtonChanged {
            button: attack,
            down: true,
        });
        // Repeated host down events (including OS key repeat) do not create edges.
        sampler.apply_event(InputEvent::ButtonChanged {
            button: attack,
            down: true,
        });

        let first = sampler.sample_tick();
        assert!(first.was_pressed(attack));
        assert!(first.is_held(attack));
        assert_eq!(first.held_for(attack), 1);

        let second = sampler.sample_tick();
        assert!(!second.was_pressed(attack));
        assert_eq!(second.held_for(attack), 2);

        sampler.apply_event(InputEvent::ButtonChanged {
            button: attack,
            down: false,
        });
        let third = sampler.sample_tick();
        assert!(third.was_released(attack));
        assert!(!third.is_held(attack));
        assert_eq!(third.held_for(attack), 0);
    }

    #[test]
    fn reset_clears_direction_buttons_and_edges() {
        let mut sampler = InputSampler::default();
        let button = Button(31);
        sampler.apply_event(InputEvent::DirectionChanged(Direction::DownLeft));
        sampler.apply_event(InputEvent::ButtonChanged { button, down: true });
        sampler.apply_event(InputEvent::Reset);

        let frame = sampler.sample_tick();
        assert_eq!(frame.direction, Direction::Neutral);
        assert_eq!(frame.held_buttons, 0);
        assert_eq!(frame.pressed_buttons, 0);
        assert_eq!(frame.released_buttons, 0);
        assert_eq!(frame.held_for(button), 0);
    }

    #[test]
    fn out_of_range_button_slots_are_ignored() {
        let mut sampler = InputSampler::default();
        sampler.apply_event(InputEvent::ButtonChanged {
            button: Button(32),
            down: true,
        });
        assert_eq!(sampler.sample_tick().held_buttons, 0);
    }
}
