//! Platform-neutral arcade opening, coin, configuration and fighter-selection flow.

use crate::arcade::{ArcadePreferences, ArcadeSettings};
use crate::input::{Button, Direction, InputFrame};
use foundation_runtime::{PhysicalKey, PlayerOneKeyMap};

/// Initial and in-game menu phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupPhase {
    Opening,
    Title,
    Config,
    KeyRemap,
    CharacterSelect,
    Fighting,
}

/// Immutable screen state for the renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StartupScreen {
    pub phase: StartupPhase,
    pub ticks: u64,
    pub selected: [u8; 2],
    pub confirmed: [bool; 2],
    pub p2_active: bool,
    pub credits: u8,
    pub settings: ArcadeSettings,
    pub menu_cursor: usize,
    pub player_one_keys: PlayerOneKeyMap,
    pub remap_cursor: usize,
    pub remap_conflict: bool,
}

/// Event emitted when startup or a live arcade cabinet action changes ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupEvent {
    None,
    PreferencesChanged,
    StartFight {
        characters: [u8; 2],
        cpu_opponent: bool,
    },
    JoinPlayer {
        player: u8,
    },
}

/// Stateful cabinet flow. Credits and menus are game state, not renderer state.
#[derive(Debug, Clone)]
pub struct StartupFlow {
    phase: StartupPhase,
    ticks: u64,
    selected: [u8; 2],
    confirmed: [bool; 2],
    p1_active: bool,
    p2_active: bool,
    credits: u8,
    preferences: ArcadePreferences,
    menu_cursor: usize,
    remap_cursor: usize,
    remap_conflict: bool,
}

impl StartupFlow {
    pub const INTRO_TICKS: u64 = 210;
    pub const MAX_CREDITS: u8 = 99;
    const MENU_REPEAT_TICKS: u64 = 5;

    /// Use command-line character IDs as the initial fighter cursors.
    pub fn new(default_characters: [u8; 2]) -> Self {
        Self::with_preferences(default_characters, ArcadePreferences::default())
    }

    /// Restore persisted settings and P1 keyboard bindings.
    pub fn with_preferences(default_characters: [u8; 2], preferences: ArcadePreferences) -> Self {
        let selected = default_characters.map(|id| id.clamp(1, 15));
        Self {
            phase: StartupPhase::Opening,
            ticks: 0,
            selected,
            confirmed: [false; 2],
            p1_active: false,
            p2_active: false,
            credits: 0,
            preferences,
            menu_cursor: 0,
            remap_cursor: 0,
            remap_conflict: false,
        }
    }

    pub fn phase(&self) -> StartupPhase {
        self.phase
    }

    pub fn credits(&self) -> u8 {
        self.credits
    }

    pub fn settings(&self) -> ArcadeSettings {
        self.preferences.settings
    }

    pub fn preferences(&self) -> ArcadePreferences {
        self.preferences
    }

    pub fn set_preferences(&mut self, preferences: ArcadePreferences) {
        self.preferences = preferences;
    }

    pub fn screen(&self) -> StartupScreen {
        StartupScreen {
            phase: self.phase,
            ticks: self.ticks,
            selected: self.selected,
            confirmed: self.confirmed,
            p2_active: self.p2_active,
            credits: self.credits,
            settings: self.preferences.settings,
            menu_cursor: self.menu_cursor,
            player_one_keys: self.preferences.player_one_keys,
            remap_cursor: self.remap_cursor,
            remap_conflict: self.remap_conflict,
        }
    }

    /// Advance the cabinet state by one authoritative input tick.
    ///
    /// Digit 5 inserts a credit (maximum 99); Digit 1 and Digit 2 spend a credit
    /// to enter P1/P2. The absent second player becomes the CPU when a fight starts.
    pub fn tick(&mut self, inputs: [InputFrame; 2]) -> StartupEvent {
        self.ticks = self.ticks.saturating_add(1);
        let physical_key = inputs[0]
            .physical_key_pressed
            .or(inputs[1].physical_key_pressed);

        if self.phase == StartupPhase::Fighting {
            return self.tick_fighting(physical_key);
        }

        if matches!(self.phase, StartupPhase::Opening | StartupPhase::Title) {
            if physical_key == Some(PhysicalKey::F1) {
                self.phase = StartupPhase::Config;
                self.menu_cursor = 0;
                self.ticks = 0;
                return StartupEvent::None;
            }
            if physical_key == Some(PhysicalKey::F2) {
                self.phase = StartupPhase::KeyRemap;
                self.remap_cursor = 0;
                self.remap_conflict = false;
                self.ticks = 0;
                return StartupEvent::None;
            }
        }

        match self.phase {
            StartupPhase::Opening | StartupPhase::Title => {
                if physical_key == Some(PhysicalKey::Digit5)
                    || physical_key == Some(PhysicalKey::Numpad5)
                {
                    self.insert_credit();
                    return StartupEvent::None;
                }

                if physical_key == Some(PhysicalKey::Digit1)
                    || physical_key == Some(PhysicalKey::Numpad1)
                {
                    self.enter_player_one();
                    return StartupEvent::None;
                }

                if physical_key == Some(PhysicalKey::Digit2)
                    || physical_key == Some(PhysicalKey::Numpad2)
                {
                    self.enter_player_two();
                    return StartupEvent::None;
                }

                if self.phase == StartupPhase::Opening {
                    if inputs[0].was_pressed(Button(0))
                        || inputs[1].was_pressed(Button(0))
                        || self.ticks >= Self::INTRO_TICKS
                    {
                        self.phase = StartupPhase::Title;
                        self.ticks = 0;
                    }
                }
            }
            StartupPhase::Config => return self.tick_config(inputs, physical_key),
            StartupPhase::KeyRemap => return self.tick_key_remap(physical_key),
            StartupPhase::CharacterSelect => {
                if physical_key == Some(PhysicalKey::Digit5)
                    || physical_key == Some(PhysicalKey::Numpad5)
                {
                    self.insert_credit();
                    return StartupEvent::None;
                }
                if physical_key == Some(PhysicalKey::Digit2)
                    || physical_key == Some(PhysicalKey::Numpad2)
                {
                    self.enter_player_two();
                }

                // Direction is level-triggered by the host. Repeat at a measured menu cadence.
                if self.ticks % Self::MENU_REPEAT_TICKS == 0 {
                    if !self.confirmed[0] {
                        move_selection(&mut self.selected[0], inputs[0].direction);
                    }
                    if self.p2_active && !self.confirmed[1] {
                        move_selection(&mut self.selected[1], inputs[1].direction);
                    }
                }

                if inputs[0].was_pressed(Button(0)) {
                    self.confirmed[0] = true;
                }
                if self.p2_active && inputs[1].was_pressed(Button(0)) {
                    self.confirmed[1] = true;
                }
                if self.confirmed[0] && !self.p2_active {
                    self.confirmed[1] = true;
                }

                if self.confirmed[0] && self.confirmed[1] {
                    self.phase = StartupPhase::Fighting;
                    return StartupEvent::StartFight {
                        characters: self.selected,
                        cpu_opponent: !self.p2_active,
                    };
                }
            }
            StartupPhase::Fighting => {}
        }
        StartupEvent::None
    }

    fn tick_fighting(&mut self, key: Option<PhysicalKey>) -> StartupEvent {
        match key {
            Some(PhysicalKey::Digit5 | PhysicalKey::Numpad5) => {
                self.insert_credit();
                StartupEvent::None
            }
            Some(PhysicalKey::Digit2 | PhysicalKey::Numpad2)
                if !self.p2_active && self.credits > 0 =>
            {
                self.credits -= 1;
                self.p2_active = true;
                self.confirmed[1] = false;
                StartupEvent::JoinPlayer { player: 2 }
            }
            _ => StartupEvent::None,
        }
    }

    fn insert_credit(&mut self) {
        self.credits = self.credits.saturating_add(1).min(Self::MAX_CREDITS);
    }

    fn enter_player_one(&mut self) {
        if self.p1_active || self.credits == 0 {
            return;
        }
        self.credits -= 1;
        self.p1_active = true;
        self.confirmed = [false; 2];
        self.phase = StartupPhase::CharacterSelect;
        self.ticks = 0;
    }

    fn enter_player_two(&mut self) {
        if self.p2_active || self.credits == 0 {
            return;
        }
        self.credits -= 1;
        self.p2_active = true;
        self.confirmed[1] = false;
    }

    fn tick_config(
        &mut self,
        inputs: [InputFrame; 2],
        physical_key: Option<PhysicalKey>,
    ) -> StartupEvent {
        if physical_key == Some(PhysicalKey::F1) {
            self.phase = StartupPhase::Title;
            self.ticks = 0;
            return StartupEvent::PreferencesChanged;
        }

        if self.ticks % Self::MENU_REPEAT_TICKS != 0 {
            return StartupEvent::None;
        }

        match inputs[0].direction {
            Direction::Up => self.menu_cursor = self.menu_cursor.saturating_sub(1),
            Direction::Down => {
                self.menu_cursor = (self.menu_cursor + 1).min(ArcadeSettings::OPTION_COUNT - 1)
            }
            Direction::Left => {
                self.preferences.settings.adjust(self.menu_cursor, -1);
                return StartupEvent::PreferencesChanged;
            }
            Direction::Right => {
                self.preferences.settings.adjust(self.menu_cursor, 1);
                return StartupEvent::PreferencesChanged;
            }
            _ => {}
        }
        StartupEvent::None
    }

    fn tick_key_remap(&mut self, physical_key: Option<PhysicalKey>) -> StartupEvent {
        let Some(key) = physical_key else {
            return StartupEvent::None;
        };
        if key == PhysicalKey::F2 {
            self.phase = StartupPhase::Title;
            self.ticks = 0;
            return StartupEvent::PreferencesChanged;
        }
        if self.remap_cursor >= 8 {
            self.phase = StartupPhase::Title;
            return StartupEvent::PreferencesChanged;
        }

        let mut candidate = self.preferences.player_one_keys;
        candidate.keys[self.remap_cursor] = key;
        self.remap_conflict = !candidate.has_unique_keys();
        if self.remap_conflict {
            return StartupEvent::None;
        }

        self.preferences.player_one_keys = candidate;
        self.remap_cursor += 1;
        if self.remap_cursor >= 8 {
            self.phase = StartupPhase::Title;
            self.ticks = 0;
        }
        StartupEvent::PreferencesChanged
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

    fn press_button(player: usize, button: u8) -> [InputFrame; 2] {
        let mut inputs = [InputFrame::default(); 2];
        inputs[player].pressed_buttons = 1 << button;
        inputs
    }

    fn press_key(key: PhysicalKey) -> [InputFrame; 2] {
        let mut inputs = [InputFrame::default(); 2];
        inputs[0].physical_key_pressed = Some(key);
        inputs
    }

    fn start_one_player() -> StartupFlow {
        let mut flow = StartupFlow::default();
        flow.tick(press_key(PhysicalKey::Digit5));
        flow.tick(press_key(PhysicalKey::Digit1));
        flow
    }

    #[test]
    fn intro_advances_to_title_on_its_own_clock() {
        let mut flow = StartupFlow::default();
        for _ in 0..StartupFlow::INTRO_TICKS {
            flow.tick([InputFrame::default(); 2]);
        }
        assert_eq!(flow.phase(), StartupPhase::Title);
    }

    #[test]
    fn inserting_a_coin_and_pressing_one_spends_exactly_one_credit() {
        let mut flow = StartupFlow::default();
        flow.tick(press_key(PhysicalKey::Digit5));
        assert_eq!(flow.credits(), 1);
        flow.tick(press_key(PhysicalKey::Digit1));
        assert_eq!(flow.credits(), 0);
        assert_eq!(flow.phase(), StartupPhase::CharacterSelect);
    }

    #[test]
    fn start_without_a_credit_does_not_start_the_fight() {
        let mut flow = StartupFlow::default();
        flow.tick(press_key(PhysicalKey::Digit1));
        assert_eq!(flow.credits(), 0);
        assert_eq!(flow.phase(), StartupPhase::Opening);
    }

    #[test]
    fn credits_are_capped_at_ninety_nine() {
        let mut flow = StartupFlow::default();
        for _ in 0..120 {
            flow.tick(press_key(PhysicalKey::Digit5));
        }
        assert_eq!(flow.credits(), StartupFlow::MAX_CREDITS);
    }

    #[test]
    fn one_player_confirmation_starts_with_a_cpu_opponent() {
        let mut flow = start_one_player();
        let event = flow.tick(press_button(0, 0));
        assert_eq!(
            event,
            StartupEvent::StartFight {
                characters: [1, 4],
                cpu_opponent: true,
            }
        );
    }

    #[test]
    fn player_two_spends_a_credit_and_disables_the_cpu() {
        let mut flow = StartupFlow::default();
        flow.tick(press_key(PhysicalKey::Digit5));
        flow.tick(press_key(PhysicalKey::Digit5));
        flow.tick(press_key(PhysicalKey::Digit1));
        flow.tick(press_key(PhysicalKey::Digit2));
        assert_eq!(flow.credits(), 0);
        flow.tick(press_button(0, 0));
        let event = flow.tick(press_button(1, 0));
        assert_eq!(
            event,
            StartupEvent::StartFight {
                characters: [1, 4],
                cpu_opponent: false,
            }
        );
    }

    #[test]
    fn player_two_can_join_a_live_fight_using_a_credit() {
        let mut flow = start_one_player();
        flow.tick(press_button(0, 0));
        flow.tick(press_key(PhysicalKey::Digit5));
        assert_eq!(
            flow.tick(press_key(PhysicalKey::Digit2)),
            StartupEvent::JoinPlayer { player: 2 }
        );
        assert_eq!(flow.credits(), 0);
    }

    #[test]
    fn f1_changes_a_setting_and_f1_saves_and_returns() {
        let mut flow = StartupFlow::default();
        flow.tick(press_key(PhysicalKey::F1));
        assert_eq!(flow.phase(), StartupPhase::Config);
        let down = {
            let mut inputs = [InputFrame::default(); 2];
            inputs[0].direction = Direction::Right;
            inputs
        };
        for _ in 0..5 {
            flow.tick(down);
        }
        assert_eq!(flow.settings().difficulty, 5);
        assert_eq!(
            flow.tick(press_key(PhysicalKey::F1)),
            StartupEvent::PreferencesChanged
        );
        assert_eq!(flow.phase(), StartupPhase::Title);
    }

    #[test]
    fn f2_captures_eight_unique_keys_and_rejects_collisions() {
        let mut flow = StartupFlow::default();
        flow.tick(press_key(PhysicalKey::F2));
        assert_eq!(flow.phase(), StartupPhase::KeyRemap);
        flow.tick(press_key(PhysicalKey::Z));
        assert!(flow.screen().remap_conflict);
        flow.tick(press_key(PhysicalKey::I));
        assert_eq!(flow.preferences().player_one_keys.keys[0], PhysicalKey::I);
        for key in [
            PhysicalKey::K,
            PhysicalKey::J,
            PhysicalKey::L,
            PhysicalKey::U,
            PhysicalKey::O,
            PhysicalKey::P,
            PhysicalKey::M,
        ] {
            flow.tick(press_key(key));
        }
        assert_eq!(flow.phase(), StartupPhase::Title);
        assert!(flow.preferences().player_one_keys.has_unique_keys());
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
