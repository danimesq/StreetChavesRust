//! Source-derived CHS_ADVERSARIOS campaign order and progression.
//!
//! The Pascal source creates seven random, non-repeating opponents, excluding
//! the selected fighter, the configured boss and each fighter's NaoEnfrenta list.
//! The eighth entry is the selected fighter's character-specific boss.

use crate::pascal_replay::PascalRandomSource;
use std::fmt::Display;

pub const CAMPAIGN_PHASES: u8 = 8;
pub const RANDOM_OPPONENTS: usize = 7;
pub const ORIGINAL_ADVERSARY_SCREEN_READY_TICK: u16 = 150;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignProfile {
    pub selected_character: u8,
    pub boss_character: u8,
    /// IDs from the selected fighter's Dados/Lutadores/CNN.ini NaoEnfrenta list.
    pub never_face: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignPlan {
    pub selected_character: u8,
    /// Seven randomized opponents followed by the configured boss.
    pub opponents: [u8; CAMPAIGN_PHASES as usize],
}

impl CampaignPlan {
    /// Reproduce criaadversario from Estado4_Selecao.inc.
    ///
    /// Each candidate is Random(FQtdLutadores) + 1 using the FPC LongInt
    /// overload. The original attempts up to 2,001 draws per opponent before
    /// accepting the last result, even when metadata makes the roster impossible.
    pub fn generate<R>(
        profile: &CampaignProfile,
        roster_size: u8,
        rng: &mut R,
    ) -> Result<Self, String>
    where
        R: PascalRandomSource,
        R::Error: Display,
    {
        if roster_size == 0 {
            return Err("campaign roster cannot be empty".to_owned());
        }
        if !(1..=roster_size).contains(&profile.selected_character) {
            return Err(format!(
                "selected character {} is outside the 1..={roster_size} roster",
                profile.selected_character
            ));
        }
        if !(1..=roster_size).contains(&profile.boss_character) {
            return Err(format!(
                "boss character {} is outside the 1..={roster_size} roster",
                profile.boss_character
            ));
        }

        let mut available = vec![true; usize::from(roster_size) + 1];
        available[0] = false;
        for excluded in &profile.never_face {
            if *excluded <= roster_size {
                available[usize::from(*excluded)] = false;
            }
        }
        available[usize::from(profile.boss_character)] = false;

        let mut opponents = [0; CAMPAIGN_PHASES as usize];
        for phase_index in 0..RANDOM_OPPONENTS {
            let mut attempts = 0usize;
            let chosen = loop {
                let draw = rng
                    .draw_longint(i32::from(roster_size))
                    .map_err(|error| format!("campaign RNG failed: {error}"))?;
                if draw < 0 || draw >= i32::from(roster_size) {
                    return Err(format!(
                        "campaign RNG returned {draw} outside 0..{roster_size}"
                    ));
                }
                let candidate = draw as u8 + 1;
                attempts += 1;
                if (available[usize::from(candidate)]
                    && candidate != profile.selected_character)
                    || attempts > 2_000
                {
                    break candidate;
                }
            };
            opponents[phase_index] = chosen;
            available[usize::from(chosen)] = false;
        }
        opponents[RANDOM_OPPONENTS] = profile.boss_character;

        Ok(Self {
            selected_character: profile.selected_character,
            opponents,
        })
    }

    /// Return the source campaign opponent for phase 1..=8.
    pub fn opponent(&self, phase: u8) -> Option<u8> {
        if !(1..=CAMPAIGN_PHASES).contains(&phase) {
            return None;
        }
        Some(self.opponents[usize::from(phase - 1)])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampaignAdvance {
    NextPhase(u8),
    Completed,
    Defeated,
    Inactive,
}

/// Progression state independent of SDL, Canvas or a particular game host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CampaignProgress {
    phase: u8,
    completed: bool,
    defeated: bool,
}

impl CampaignProgress {
    pub const fn new() -> Self {
        Self {
            phase: 1,
            completed: false,
            defeated: false,
        }
    }

    pub fn phase(&self) -> Option<u8> {
        (!self.completed && !self.defeated).then_some(self.phase)
    }

    /// Advance only after the current fight has been won.
    pub fn record_victory(&mut self) -> CampaignAdvance {
        if self.completed || self.defeated {
            return CampaignAdvance::Inactive;
        }
        if self.phase == CAMPAIGN_PHASES {
            self.completed = true;
            CampaignAdvance::Completed
        } else {
            self.phase += 1;
            CampaignAdvance::NextPhase(self.phase)
        }
    }

    pub fn record_defeat(&mut self) -> CampaignAdvance {
        if self.completed || self.defeated {
            return CampaignAdvance::Inactive;
        }
        self.defeated = true;
        CampaignAdvance::Defeated
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn is_defeated(&self) -> bool {
        self.defeated
    }
}

impl Default for CampaignProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CampaignStageProfile {
    pub primary_stage: u8,
    pub alternative_stage: u8,
    pub rare_stage: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampaignStageSource {
    Primary,
    Alternative,
    Rare,
    Secret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CampaignStageChoice {
    pub stage: u8,
    pub source: CampaignStageSource,
}

/// Reproduce the alternative/rare stage rolls from Estado5_Adversarios.inc.
/// A nonzero secret stage is applied last and clamped to the stage range.
pub fn choose_campaign_stage<R>(
    profile: CampaignStageProfile,
    phase: u8,
    stage_count: u8,
    secret_stage: Option<u8>,
    rng: &mut R,
) -> Result<CampaignStageChoice, String>
where
    R: PascalRandomSource,
    R::Error: Display,
{
    if stage_count == 0 {
        return Err("campaign stage list cannot be empty".to_owned());
    }
    if !(1..=CAMPAIGN_PHASES).contains(&phase) {
        return Err(format!("campaign phase {phase} is outside 1..=8"));
    }

    // The two source rolls are independent and retain their original order.
    let alternative_roll = rng
        .draw_longint(5)
        .map_err(|error| format!("alternative-stage RNG failed: {error}"))?;
    let rare_roll = rng
        .draw_longint(6)
        .map_err(|error| format!("rare-stage RNG failed: {error}"))?;

    let mut choice = CampaignStageChoice {
        stage: profile.primary_stage.clamp(1, stage_count),
        source: CampaignStageSource::Primary,
    };
    if phase < CAMPAIGN_PHASES && profile.alternative_stage != 0 && alternative_roll <= 1 {
        choice = CampaignStageChoice {
            stage: profile.alternative_stage.clamp(1, stage_count),
            source: CampaignStageSource::Alternative,
        };
    }
    if phase < CAMPAIGN_PHASES && profile.rare_stage != 0 && rare_roll <= 1 {
        choice = CampaignStageChoice {
            stage: profile.rare_stage.clamp(1, stage_count),
            source: CampaignStageSource::Rare,
        };
    }
    if let Some(secret) = secret_stage.filter(|stage| *stage != 0) {
        choice = CampaignStageChoice {
            stage: secret.clamp(1, stage_count),
            source: CampaignStageSource::Secret,
        };
    }
    Ok(choice)
}

/// CHS_ADVERSARIOS presentation clock: the original enters preparation after
/// TempoEstado exceeds 150; an active player's skip edge adds five ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AdversariesPresentation {
    elapsed_ticks: u16,
}

impl AdversariesPresentation {
    pub fn tick(&mut self, player_skipped: bool) -> bool {
        self.elapsed_ticks = self.elapsed_ticks.saturating_add(1);
        if player_skipped && self.elapsed_ticks > 0 {
            self.elapsed_ticks = self.elapsed_ticks.saturating_add(5);
        }
        self.is_ready_for_preparation()
    }

    pub fn elapsed_ticks(&self) -> u16 {
        self.elapsed_ticks
    }

    pub fn is_ready_for_preparation(&self) -> bool {
        self.elapsed_ticks > ORIGINAL_ADVERSARY_SCREEN_READY_TICK
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    struct SequenceRng {
        values: VecDeque<i32>,
    }

    impl SequenceRng {
        fn new(values: impl IntoIterator<Item = i32>) -> Self {
            Self { values: values.into_iter().collect() }
        }
    }

    impl PascalRandomSource for SequenceRng {
        type Error = &'static str;

        fn draw_longint(&mut self, range: i32) -> Result<i32, Self::Error> {
            let value = self.values.pop_front().ok_or("no scripted RNG value")?;
            Ok(if range <= 0 { 0 } else { value.rem_euclid(range) })
        }

        fn draw_int64(&mut self, range: i64) -> Result<i64, Self::Error> {
            let value = self.values.pop_front().ok_or("no scripted RNG value")?;
            Ok(if range <= 0 { 0 } else { i64::from(value).rem_euclid(range) })
        }
    }

    #[test]
    fn order_excludes_player_forbidden_fighters_and_boss_without_repeats() {
        let profile = CampaignProfile {
            selected_character: 2,
            boss_character: 15,
            never_face: vec![3],
        };
        let mut rng = SequenceRng::new([0, 1, 2, 3, 4, 5, 6, 7, 8]);
        let plan = CampaignPlan::generate(&profile, 15, &mut rng).unwrap();
        assert_eq!(plan.opponents, [1, 4, 5, 6, 7, 8, 9, 15]);
        assert!(!plan.opponents[..7].contains(&2));
        assert!(!plan.opponents[..7].contains(&3));
        assert!(!plan.opponents[..7].contains(&15));
    }

    #[test]
    fn boss_is_phase_eight_and_campaign_finishes_after_its_victory() {
        let plan = CampaignPlan {
            selected_character: 1,
            opponents: [2, 3, 4, 5, 6, 7, 8, 15],
        };
        let mut progress = CampaignProgress::new();
        for phase in 2..=8 {
            assert_eq!(progress.record_victory(), CampaignAdvance::NextPhase(phase));
        }
        assert_eq!(plan.opponent(progress.phase().unwrap()), Some(15));
        assert_eq!(progress.record_victory(), CampaignAdvance::Completed);
        assert_eq!(progress.phase(), None);
        assert_eq!(progress.record_victory(), CampaignAdvance::Inactive);
    }

    #[test]
    fn defeat_ends_campaign_without_advancing() {
        let mut progress = CampaignProgress::new();
        assert_eq!(progress.record_defeat(), CampaignAdvance::Defeated);
        assert_eq!(progress.phase(), None);
        assert!(progress.is_defeated());
    }

    #[test]
    fn secret_stage_overrides_the_ordered_alternative_and_rare_rolls() {
        let profile = CampaignStageProfile {
            primary_stage: 2,
            alternative_stage: 3,
            rare_stage: 4,
        };
        let mut rng = SequenceRng::new([1, 1]);
        let choice = choose_campaign_stage(profile, 4, 12, Some(99), &mut rng).unwrap();
        assert_eq!(choice.stage, 12);
        assert_eq!(choice.source, CampaignStageSource::Secret);
    }

    #[test]
    fn presentation_clock_uses_the_original_151_tick_boundary_and_skip_step() {
        let mut view = AdversariesPresentation::default();
        assert!(!view.tick(false));
        assert_eq!(view.elapsed_ticks(), 1);
        assert!(!view.tick(true));
        assert_eq!(view.elapsed_ticks(), 7);
        for _ in 0..144 {
            view.tick(false);
        }
        assert!(view.is_ready_for_preparation());
    }
}
