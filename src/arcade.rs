//! Arcade cabinet settings, coin accounting and shared-keyboard preferences.
//!
//! The menu mirrors the twelve options exposed by the original F1 screen.
//! Settings are shared by native and browser runtimes, including round, HUD and audio controls.

use foundation_runtime::{PhysicalKey, PlayerOneKeyMap};

/// Arcade options displayed by the F1 configuration screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcadeSettings {
    /// Original game level, 1 (easiest) through 8 (hardest).
    pub difficulty: u8,
    /// 1 = 30s, 2 = 60s, 3 = 90s, 4 = unlimited.
    pub round_time: u8,
    /// 1 = normal, 2 = P1 max, 3 = P2 max, 4 = both max.
    pub power_mode: u8,
    /// Damage multiplier, bounded to the original 0.3..=2.0 range.
    pub damage_level: f64,
    pub blood: bool,
    pub rounds: u8,
    /// 1 = Portuguese, 2 = Spanish.
    pub language: u8,
    /// Selected sound-effect test slot, 0 means no preview.
    pub sound_test: u8,
    /// Selected music track, 0 means silence.
    pub music_track: u8,
    pub flash: bool,
    pub show_level: bool,
    pub show_credits: bool,
}

impl Default for ArcadeSettings {
    fn default() -> Self {
        Self {
            difficulty: 4,
            round_time: 3,
            power_mode: 1,
            damage_level: 0.9,
            blood: true,
            rounds: 2,
            language: 1,
            sound_test: 0,
            music_track: 1,
            flash: true,
            show_level: true,
            show_credits: true,
        }
    }
}

impl ArcadeSettings {
    pub const OPTION_COUNT: usize = 12;

    /// Move one F1 setting left (-1) or right (+1), applying source bounds.
    pub fn adjust(&mut self, option: usize, delta: i8) {
        let direction = i16::from(delta.signum());
        match option {
            0 => self.difficulty = (i16::from(self.difficulty) + direction).clamp(1, 8) as u8,
            1 => self.round_time = (i16::from(self.round_time) + direction).clamp(1, 4) as u8,
            2 => self.power_mode = (i16::from(self.power_mode) + direction).clamp(1, 4) as u8,
            3 => {
                self.damage_level = ((self.damage_level * 10.0 + f64::from(direction)).round()
                    / 10.0)
                    .clamp(0.3, 2.0);
            }
            4 => self.blood = if delta != 0 { !self.blood } else { self.blood },
            5 => self.rounds = (i16::from(self.rounds) + direction).clamp(1, 3) as u8,
            6 => self.language = (i16::from(self.language) + direction).clamp(1, 2) as u8,
            7 => self.sound_test = (i16::from(self.sound_test) + direction).clamp(0, 71) as u8,
            8 => self.music_track = (i16::from(self.music_track) + direction).clamp(0, 6) as u8,
            9 => self.flash = if delta != 0 { !self.flash } else { self.flash },
            10 => {
                self.show_level = if delta != 0 {
                    !self.show_level
                } else {
                    self.show_level
                }
            }
            11 => {
                self.show_credits = if delta != 0 {
                    !self.show_credits
                } else {
                    self.show_credits
                }
            }
            _ => {}
        }
    }

    /// Localized-independent value shown in the compact F1 options list.
    pub fn value_label(self, option: usize) -> String {
        match option {
            0 => self.difficulty.to_string(),
            1 => ["", "30 SEC", "60 SEC", "90 SEC", "NO LIMIT"][usize::from(self.round_time)]
                .to_owned(),
            2 => ["", "NORMAL", "P1 MAX", "P2 MAX", "BOTH MAX"][usize::from(self.power_mode)]
                .to_owned(),
            3 => format!("{:.1}x", self.damage_level),
            4 => enabled(self.blood),
            5 => self.rounds.to_string(),
            6 => if self.language == 1 {
                "PORTUGUESE"
            } else {
                "SPANISH"
            }
            .to_owned(),
            7 => {
                if self.sound_test == 0 {
                    "OFF".to_owned()
                } else {
                    self.sound_test.to_string()
                }
            }
            8 => {
                if self.music_track == 0 {
                    "OFF".to_owned()
                } else {
                    self.music_track.to_string()
                }
            }
            9 => enabled(self.flash),
            10 => enabled(self.show_level),
            11 => enabled(self.show_credits),
            _ => String::new(),
        }
    }

    /// Language-aware option captions used by the cabinet's original F1 page.
    pub const fn localized_option_name(self, option: usize) -> &'static str {
        if self.language == 2 {
            match option {
                0 => "NIVEL DE JUEGO",
                1 => "TIEMPO DE RONDA",
                2 => "PODER",
                3 => "DANO",
                4 => "SANGRE",
                5 => "RONDA",
                6 => "IDIOMA",
                7 => "PROBAR SONIDO",
                8 => "MUSICA",
                9 => "DESTELLO",
                10 => "MOSTRAR NIVEL",
                11 => "MOSTRAR CREDITOS",
                _ => "",
            }
        } else {
            match option {
                0 => "NIVEL DE JOGO",
                1 => "TEMPO DE ROUND",
                2 => "PODER",
                3 => "DANO",
                4 => "SANGUE",
                5 => "ROUNDS",
                6 => "IDIOMA",
                7 => "TESTE DE SOM",
                8 => "MUSICA",
                9 => "FLASH",
                10 => "MOSTRAR NIVEL",
                11 => "MOSTRAR CREDITOS",
                _ => "",
            }
        }
    }

    /// Localize value labels without mutating the stored numeric F1 settings.
    pub fn localized_value_label(self, option: usize) -> String {
        let enabled_label = |value| {
            if self.language == 2 {
                if value {
                    "SI"
                } else {
                    "NO"
                }
            } else if value {
                "SIM"
            } else {
                "NAO"
            }
        };
        match option {
            1 => match self.round_time {
                1 => "30 SEG",
                2 => "60 SEG",
                3 => "90 SEG",
                _ => {
                    if self.language == 2 {
                        "ILIMITADO"
                    } else {
                        "ILIMITADO"
                    }
                }
            }
            .to_owned(),
            2 => match (self.language, self.power_mode) {
                (_, 1) => "NORMAL",
                (1, 2) => "P1 MAX",
                (1, 3) => "P2 MAX",
                (_, 2) => "J1 MAX",
                (_, 3) => "J2 MAX",
                _ => "AMBOS MAX",
            }
            .to_owned(),
            4 => enabled_label(self.blood).to_owned(),
            6 => if self.language == 2 {
                "ESPANOL"
            } else {
                "PORTUGUES"
            }
            .to_owned(),
            7 => {
                if self.sound_test == 0 {
                    if self.language == 2 {
                        "APAGADO"
                    } else {
                        "DESLIGADO"
                    }
                    .to_owned()
                } else {
                    self.sound_test.to_string()
                }
            }
            8 => {
                if self.music_track == 0 {
                    if self.language == 2 {
                        "SILENCIO"
                    } else {
                        "SILENCIO"
                    }
                    .to_owned()
                } else {
                    self.music_track.to_string()
                }
            }
            9 => enabled_label(self.flash).to_owned(),
            10 => enabled_label(self.show_level).to_owned(),
            11 => enabled_label(self.show_credits).to_owned(),
            _ => self.value_label(option),
        }
    }

    /// Stable row name for the F1 screen.
    pub const fn option_name(option: usize) -> &'static str {
        match option {
            0 => "GAME LEVEL",
            1 => "ROUND TIME",
            2 => "POWER",
            3 => "DAMAGE",
            4 => "BLOOD",
            5 => "ROUNDS",
            6 => "LANGUAGE",
            7 => "SOUND TEST",
            8 => "MUSIC TRACK",
            9 => "FLASH",
            10 => "SHOW LEVEL",
            11 => "SHOW CREDITS",
            _ => "",
        }
    }
}

/// Result of resolving one round in a best-of-N cabinet match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundAdvance {
    Continue { round: u8, winner: usize },
    MatchWon { winner: usize },
    Draw { next_round: u8 },
    Inactive,
}

/// Per-match clock and round counter shared by native and browser hosts.
/// The authoritative simulation cadence is 28 ms, matching the Pascal timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArcadeRoundProgress {
    round_limit_ticks: Option<u32>,
    elapsed_ticks: u32,
    rounds_to_win: u8,
    rounds_won: [u8; 2],
    round: u8,
    finished: bool,
}
impl ArcadeRoundProgress {
    pub fn new(settings: ArcadeSettings) -> Self {
        let seconds = match settings.round_time {
            1 => Some(30_u32),
            2 => Some(60),
            3 => Some(90),
            _ => None,
        };
        Self {
            round_limit_ticks: seconds
                .map(|value| value.saturating_mul(1000).saturating_add(27) / 28),
            elapsed_ticks: 0,
            rounds_to_win: settings.rounds.clamp(1, 3),
            rounds_won: [0; 2],
            round: 1,
            finished: false,
        }
    }
    pub fn tick(&mut self) -> bool {
        if self.finished {
            return false;
        }
        self.elapsed_ticks = self.elapsed_ticks.saturating_add(1);
        self.round_limit_ticks
            .is_some_and(|limit| self.elapsed_ticks >= limit)
    }
    pub fn elapsed_ticks(&self) -> u32 {
        self.elapsed_ticks
    }
    pub fn round(&self) -> u8 {
        self.round
    }
    pub fn rounds_won(&self) -> [u8; 2] {
        self.rounds_won
    }
    pub fn rounds_to_win(&self) -> u8 {
        self.rounds_to_win
    }
    pub fn remaining_ticks(&self) -> Option<u32> {
        self.round_limit_ticks
            .map(|limit| limit.saturating_sub(self.elapsed_ticks))
    }
    pub fn is_finished(&self) -> bool {
        self.finished
    }
    pub fn record_result(&mut self, winner: Option<usize>) -> RoundAdvance {
        if self.finished {
            return RoundAdvance::Inactive;
        }
        match winner {
            Some(player) if player < 2 => {
                self.rounds_won[player] = self.rounds_won[player].saturating_add(1);
                if self.rounds_won[player] >= self.rounds_to_win {
                    self.finished = true;
                    RoundAdvance::MatchWon { winner: player }
                } else {
                    self.round = self.round.saturating_add(1);
                    self.elapsed_ticks = 0;
                    RoundAdvance::Continue {
                        round: self.round,
                        winner: player,
                    }
                }
            }
            _ => {
                self.round = self.round.saturating_add(1);
                self.elapsed_ticks = 0;
                RoundAdvance::Draw {
                    next_round: self.round,
                }
            }
        }
    }
}

fn enabled(value: bool) -> String {
    if value { "ON" } else { "OFF" }.to_owned()
}

/// Persisted settings plus the eight physical P1 bindings configured in F2.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcadePreferences {
    pub settings: ArcadeSettings,
    pub player_one_keys: PlayerOneKeyMap,
}

impl Default for ArcadePreferences {
    fn default() -> Self {
        Self {
            settings: ArcadeSettings::default(),
            player_one_keys: PlayerOneKeyMap::default(),
        }
    }
}

impl ArcadePreferences {
    /// Encode preferences as stable JSON for native files and browser localStorage.
    pub fn to_json(self) -> String {
        let keys: Vec<&str> = self
            .player_one_keys
            .keys
            .iter()
            .map(|key| key.label())
            .collect();
        serde_json::json!({
            "schema_version": 1,
            "settings": {
                "difficulty": self.settings.difficulty,
                "round_time": self.settings.round_time,
                "power_mode": self.settings.power_mode,
                "damage_level": self.settings.damage_level,
                "blood": self.settings.blood,
                "rounds": self.settings.rounds,
                "language": self.settings.language,
                "sound_test": self.settings.sound_test,
                "music_track": self.settings.music_track,
                "flash": self.settings.flash,
                "show_level": self.settings.show_level,
                "show_credits": self.settings.show_credits
            },
            "player_one_keys": keys
        })
        .to_string()
    }

    /// Decode preferences, falling back field-by-field to original game defaults.
    /// A malformed or duplicate key map is rejected without discarding good settings.
    pub fn from_json(source: &str) -> Option<Self> {
        let value: serde_json::Value = serde_json::from_str(source).ok()?;
        if value.get("schema_version")?.as_u64()? != 1 {
            return None;
        }

        let defaults = Self::default();
        let settings_value = value.get("settings")?;
        let mut settings = defaults.settings;
        settings.difficulty = bounded_u8(settings_value, "difficulty", 1, 8, settings.difficulty);
        settings.round_time = bounded_u8(settings_value, "round_time", 1, 4, settings.round_time);
        settings.power_mode = bounded_u8(settings_value, "power_mode", 1, 4, settings.power_mode);
        settings.damage_level = settings_value
            .get("damage_level")
            .and_then(serde_json::Value::as_f64)
            .filter(|number| number.is_finite())
            .unwrap_or(settings.damage_level)
            .clamp(0.3, 2.0);
        settings.blood = settings_value
            .get("blood")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(settings.blood);
        settings.rounds = bounded_u8(settings_value, "rounds", 1, 3, settings.rounds);
        settings.language = bounded_u8(settings_value, "language", 1, 2, settings.language);
        settings.sound_test = bounded_u8(settings_value, "sound_test", 0, 71, settings.sound_test);
        settings.music_track =
            bounded_u8(settings_value, "music_track", 0, 6, settings.music_track);
        settings.flash = settings_value
            .get("flash")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(settings.flash);
        settings.show_level = settings_value
            .get("show_level")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(settings.show_level);
        settings.show_credits = settings_value
            .get("show_credits")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(settings.show_credits);

        let keys = value.get("player_one_keys")?.as_array()?;
        if keys.len() != 8 {
            return None;
        }
        let mut parsed = Vec::with_capacity(8);
        for key in keys {
            parsed.push(parse_physical_key(key.as_str()?)?);
        }
        let keys: [PhysicalKey; 8] = parsed.try_into().ok()?;
        let player_one_keys = PlayerOneKeyMap { keys };
        if !player_one_keys.has_unique_keys() {
            return None;
        }
        Some(Self {
            settings,
            player_one_keys,
        })
    }
}

fn bounded_u8(object: &serde_json::Value, field: &str, min: u8, max: u8, fallback: u8) -> u8 {
    object
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
        .filter(|value| (min..=max).contains(value))
        .unwrap_or(fallback)
}

fn parse_physical_key(label: &str) -> Option<PhysicalKey> {
    use PhysicalKey::*;
    Some(match label {
        "A" => A,
        "B" => B,
        "C" => C,
        "D" => D,
        "E" => E,
        "F" => F,
        "G" => G,
        "H" => H,
        "I" => I,
        "J" => J,
        "K" => K,
        "L" => L,
        "M" => M,
        "N" => N,
        "O" => O,
        "P" => P,
        "Q" => Q,
        "R" => R,
        "S" => S,
        "T" => T,
        "U" => U,
        "V" => V,
        "W" => W,
        "X" => X,
        "Y" => Y,
        "Z" => Z,
        "0" => Digit0,
        "1" => Digit1,
        "2" => Digit2,
        "3" => Digit3,
        "4" => Digit4,
        "5" => Digit5,
        "6" => Digit6,
        "7" => Digit7,
        "8" => Digit8,
        "9" => Digit9,
        "UP" => ArrowUp,
        "DOWN" => ArrowDown,
        "LEFT" => ArrowLeft,
        "RIGHT" => ArrowRight,
        "NUM 0" => Numpad0,
        "NUM 1" => Numpad1,
        "NUM 2" => Numpad2,
        "NUM 3" => Numpad3,
        "NUM 4" => Numpad4,
        "NUM 5" => Numpad5,
        "NUM 6" => Numpad6,
        "NUM 7" => Numpad7,
        "NUM 8" => Numpad8,
        "NUM 9" => Numpad9,
        "F1" => F1,
        "F2" => F2,
        "F3" => F3,
        "F4" => F4,
        "F5" => F5,
        "F6" => F6,
        "F7" => F7,
        "F8" => F8,
        "F9" => F9,
        "F10" => F10,
        "F11" => F11,
        "F12" => F12,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_rules_apply_timer_and_first_to_series() {
        let mut settings = ArcadeSettings::default();
        settings.round_time = 1;
        settings.rounds = 2;
        let mut progress = ArcadeRoundProgress::new(settings);
        assert_eq!(progress.rounds_to_win(), 2);
        for _ in 0..(30_000 / 28) {
            progress.tick();
        }
        assert!(progress.tick());
        assert_eq!(
            progress.record_result(Some(0)),
            RoundAdvance::Continue {
                round: 2,
                winner: 0
            }
        );
        assert_eq!(
            progress.record_result(Some(0)),
            RoundAdvance::MatchWon { winner: 0 }
        );
        assert_eq!(progress.record_result(Some(1)), RoundAdvance::Inactive);

        settings.rounds = 3;
        let first_to_three = ArcadeRoundProgress::new(settings);
        assert_eq!(first_to_three.rounds_to_win(), 3);
    }

    #[test]
    fn unlimited_round_setting_has_no_clock_deadline() {
        let mut settings = ArcadeSettings::default();
        settings.round_time = 4;
        let mut progress = ArcadeRoundProgress::new(settings);
        for _ in 0..100_000 {
            assert!(!progress.tick());
        }
        assert_eq!(progress.remaining_ticks(), None);
    }

    #[test]
    fn original_game_defaults_are_preserved() {
        let settings = ArcadeSettings::default();
        assert_eq!(settings.difficulty, 4);
        assert_eq!(settings.round_time, 3);
        assert_eq!(settings.power_mode, 1);
        assert_eq!(settings.damage_level, 0.9);
        assert!(settings.blood);
        assert_eq!(settings.rounds, 2);
        assert_eq!(settings.language, 1);
    }

    #[test]
    fn f1_values_respect_original_bounds() {
        let mut settings = ArcadeSettings::default();
        for _ in 0..20 {
            settings.adjust(0, 1);
        }
        assert_eq!(settings.difficulty, 8);
        for _ in 0..30 {
            settings.adjust(3, 1);
        }
        assert_eq!(settings.damage_level, 2.0);
        settings.adjust(3, -1);
        assert_eq!(settings.damage_level, 1.9);
    }

    #[test]
    fn settings_and_key_map_round_trip() {
        let mut preferences = ArcadePreferences::default();
        preferences.settings.adjust(0, 1);
        preferences.player_one_keys.keys[4] = PhysicalKey::J;
        let encoded = preferences.to_json();
        assert_eq!(ArcadePreferences::from_json(&encoded), Some(preferences));
    }

    #[test]
    fn duplicate_keys_reject_the_saved_map() {
        let json = ArcadePreferences::default()
            .to_json()
            .replace("\"Z\"", "\"UP\"");
        assert!(ArcadePreferences::from_json(&json).is_none());
    }
}
