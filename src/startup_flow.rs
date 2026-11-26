//! Platform-neutral boot, title and local fighter-selection flow.

use crate::input::{Button, Direction, InputFrame};

/// Initial native application phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupPhase {
    /// Short original-game-inspired opening animation before the title screen.
    Opening,
    /// Start prompt.
    Title,
    /// Two independent local fighter cursors and confirmation states.
    CharacterSelect,
    /// The main fight loop owns the application from this point on.
    Fighting,
}

/// Immutable screen state for the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartupScreen {
    pub phase: StartupPhase,
    pub ticks: u64,
    pub selected: [u8; 2],
    pub confirmed: [bool; 2],
    pub p2_active: bool,
}

/// An event emitted by the state machine when startup is complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupEvent {
    None,
    StartFight { characters: [u8; 2] },
}

/// Stateful startup flow. It does not own rendering, timing, SDL or game simulation.
#[derive(Debug, Clone)]
pub struct StartupFlow {
    phase: StartupPhase,
    ticks: u64,
    selected: [u8; 2],
    confirmed: [bool; 2],
    p2_active: bool,
}

impl StartupFlow {
    pub const INTRO_TICKS: u64 = 210;

    /// Use command-line character IDs as the initial fighter cursors.
    pub fn new(default_characters: [u8; 2]) -> Self {
        let selected = default_characters.map(|id| id.clamp(1, 15));
        Self {
            phase: StartupPhase::Opening,
            ticks: 0,
            selected,
            confirmed: [false; 2],
            p2_active: false,
        }
    }

    pub fn phase(&self) -> StartupPhase {
        self.phase
    }

    pub fn screen(&self) -> StartupScreen {
        StartupScreen {
            phase: self.phase,
            ticks: self.ticks,
            selected: self.selected,
            confirmed: self.confirmed,
            p2_active: self.p2_active,
        }
    }

    /// Advance the startup flow by one authoritative input tick.
    ///
    /// P1 uses arrows + Z. P2 joins by using WASD/R. If P2 does not join,
    /// the selected second fighter remains a stationary training opponent.
    pub fn tick(&mut self, inputs: [InputFrame; 2]) -> StartupEvent {
        if self.phase == StartupPhase::Fighting {
            return StartupEvent::None;
        }

        self.ticks = self.ticks.saturating_add(1);
        let p1_confirm = inputs[0].was_pressed(Button(0));
        let p2_confirm = inputs[1].was_pressed(Button(0));

        match self.phase {
            StartupPhase::Opening => {
                if p1_confirm || p2_confirm || self.ticks >= Self::INTRO_TICKS {
                    self.phase = StartupPhase::Title;
                    self.ticks = 0;
                }
            }
            StartupPhase::Title => {
                if p1_confirm || p2_confirm {
                    self.phase = StartupPhase::CharacterSelect;
                    self.ticks = 0;
                    self.confirmed = [false; 2];
                    self.p2_active = p2_confirm || inputs[1].direction != Direction::Neutral;
                }
            }
            StartupPhase::CharacterSelect => {
                if inputs[1].direction != Direction::Neutral
                    || inputs[1].held_buttons != 0
                    || inputs[1].pressed_buttons != 0
                {
                    self.p2_active = true;
                }

                // Direction is level-triggered by the host. Repeat at a measured menu cadence.
                if self.ticks % 5 == 0 {
                    if !self.confirmed[0] {
                        move_selection(&mut self.selected[0], inputs[0].direction);
                    }
                    if self.p2_active && !self.confirmed[1] {
                        move_selection(&mut self.selected[1], inputs[1].direction);
                    }
                }

                if p1_confirm {
                    self.confirmed[0] = true;
                }
                if p2_confirm {
                    self.confirmed[1] = true;
                }
                if self.confirmed[0] && !self.p2_active {
                    self.confirmed[1] = true;
                }

                if self.confirmed[0] && self.confirmed[1] {
                    self.phase = StartupPhase::Fighting;
                    return StartupEvent::StartFight {
                        characters: self.selected,
                    };
                }
            }
            StartupPhase::Fighting => {}
        }
        StartupEvent::None
    }
}

impl Default for StartupFlow {
    fn default() -> Self {
        Self::new([1, 4])
    }
}

fn move_selection(selected: &mut u8, direction: Direction) {
    let offset = match direction {
        Direction::Left => -1_i16,
        Direction::Right => 1,
        Direction::Up => -5,
        Direction::Down => 5,
        Direction::UpLeft => -6,
        Direction::UpRight => -4,
        Direction::DownLeft => 4,
        Direction::DownRight => 6,
        Direction::Neutral => 0,
    };
    if offset == 0 {
        return;
    }
    let next = (i16::from(*selected) - 1 + offset).rem_euclid(15) + 1;
    *selected = next as u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press_start() -> [InputFrame; 2] {
        [
            InputFrame {
                pressed_buttons: 1,
                ..Default::default()
            },
            InputFrame::default(),
        ]
    }

    #[test]
    fn intro_advances_to_title_on_its_own_clock() {
        let mut flow = StartupFlow::default();
        for _ in 0..StartupFlow::INTRO_TICKS - 1 {
            assert_eq!(flow.tick([InputFrame::default(); 2]), StartupEvent::None);
        }
        assert_eq!(flow.phase(), StartupPhase::Title);
    }

    #[test]
    fn intro_can_be_skipped_and_title_requires_a_second_start_press() {
        let mut flow = StartupFlow::default();
        flow.tick(press_start());
        assert_eq!(flow.phase(), StartupPhase::Title);
        assert_eq!(flow.tick(press_start()), StartupEvent::None);
        assert_eq!(flow.phase(), StartupPhase::CharacterSelect);
    }

    #[test]
    fn one_player_confirmation_starts_with_the_default_second_fighter() {
        let mut flow = StartupFlow::default();
        flow.tick(press_start());
        flow.tick(press_start());
        assert_eq!(
            flow.tick(press_start()),
            StartupEvent::StartFight {
                characters: [1, 4]
            }
        );
    }

    #[test]
    fn second_player_must_confirm_after_joining() {
        let mut flow = StartupFlow::default();
        flow.tick(press_start());
        flow.tick(press_start());
        let p2_move = [
            InputFrame::default(),
            InputFrame {
                direction: Direction::Left,
                ..Default::default()
            },
        ];
        for _ in 0..5 {
            flow.tick(p2_move);
        }
        assert_eq!(flow.tick(press_start()), StartupEvent::None);
        let both_confirm = [
            InputFrame {
                pressed_buttons: 1,
                ..Default::default()
            },
            InputFrame {
                pressed_buttons: 1,
                ..Default::default()
            },
        ];
        assert_eq!(
            flow.tick(both_confirm),
            StartupEvent::StartFight {
                characters: [1, 3]
            }
        );
    }


    #[test]
    fn player_two_can_join_from_the_title_screen() {
        let mut flow = StartupFlow::default();
        let both_start = [
            InputFrame {
                pressed_buttons: 1,
                ..Default::default()
            },
            InputFrame {
                pressed_buttons: 1,
                ..Default::default()
            },
        ];
        flow.tick(both_start);
        flow.tick(press_start());
        assert!(flow.screen().p2_active);
    }

    #[test]
    fn character_selection_wraps_around_the_fifteen_character_roster() {
        let mut selected = 1;
        move_selection(&mut selected, Direction::Left);
        assert_eq!(selected, 15);
        move_selection(&mut selected, Direction::Down);
        assert_eq!(selected, 5);
    }
}
