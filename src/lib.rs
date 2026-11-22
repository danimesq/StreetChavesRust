//! Evidence-driven contracts for the Street Chaves simulation core.
//!
//! These types preserve source order and source data. They intentionally do
//! not decide unresolved original-game semantics such as collision edge rules,
//! box retention between frames, or Delphi's exact random-number algorithm.

use std::fmt;

/// Stable fighter identifier from the researched roster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FighterId(pub u8);

/// Character action identifier. The numeric mapping remains source-specific.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionId(pub u16);

/// Zero-based logical tick within an action timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tick(pub u32);

/// Raw source coordinates. Do not reorder or normalize until the source
/// coordinate convention is established for the specific data table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawBounds {
    pub x1: i16,
    pub y1: i16,
    pub x2: i16,
    pub y2: i16,
}

/// Attack-box row, retaining every field proven to match between Pascal and C.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitboxRecord {
    pub frame: u16,
    pub timer: u16,
    pub bounds: RawBounds,
    pub active: bool,
    pub strength: u8,
    pub damage: u16,
    pub guard_damage: u16,
}

/// Hurtbox row as recorded in the fighter frame tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HurtboxRecord {
    pub frame: u16,
    pub bounds: RawBounds,
}

/// Explicit update to a box collection. Retain is representable, not a claim
/// that the original game retains boxes when a frame has no table row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoxUpdate<T> {
    Replace(Vec<T>),
    Clear,
    Retain,
}

/// Current box state for a simulation adapter or deterministic fixture.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BoxState<T> {
    rows: Vec<T>,
}

impl<T: Clone> BoxState<T> {
    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    /// Apply only an explicit operation. Missing data must not imply Retain.
    pub fn apply(&mut self, update: &BoxUpdate<T>) {
        match update {
            BoxUpdate::Replace(rows) => self.rows.clone_from(rows),
            BoxUpdate::Clear => self.rows.clear(),
            BoxUpdate::Retain => {}
        }
    }
}

/// Ordered event emitted by one action on one logical tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatEvent {
    FrameChanged { frame: u16 },
    SoundQueued { sound_id: u16 },
    EffectSpawned { effect_id: u16, variant: Option<char> },
    /// Generic until the final typed player-state schema is evidence-backed.
    StateMutation { field: String, value: i32 },
    RandomDraw { upper_exclusive: u32, value: u32, draw_index: u32 },
    Hitboxes(BoxUpdate<HitboxRecord>),
    Hurtboxes(BoxUpdate<HurtboxRecord>),
}

/// One event with its exact order inside a tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedEvent {
    pub ordinal: u16,
    pub event: CombatEvent,
}

/// All observed events for one action tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickEvents {
    pub tick: Tick,
    events: Vec<OrderedEvent>,
}

impl TickEvents {
    pub fn new(tick: Tick) -> Self {
        Self { tick, events: Vec::new() }
    }

    pub fn events(&self) -> &[OrderedEvent] {
        &self.events
    }

    /// Preserve insertion/source order instead of sorting by event category.
    pub fn push(&mut self, event: CombatEvent) -> Result<u16, TimelineError> {
        let ordinal = u16::try_from(self.events.len())
            .map_err(|_| TimelineError::TooManyEvents(self.tick))?;
        self.events.push(OrderedEvent { ordinal, event });
        Ok(ordinal)
    }
}

/// Source-ordered timeline for one fighter action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionTimeline {
    pub fighter: FighterId,
    pub action: ActionId,
    ticks: Vec<TickEvents>,
}

impl ActionTimeline {
    pub fn new(fighter: FighterId, action: ActionId) -> Self {
        Self { fighter, action, ticks: Vec::new() }
    }

    pub fn ticks(&self) -> &[TickEvents] {
        &self.ticks
    }

    /// Ticks must increase strictly. Same-tick order lives inside TickEvents.
    pub fn push_tick(&mut self, tick: TickEvents) -> Result<(), TimelineError> {
        if let Some(previous) = self.ticks.last() {
            if tick.tick <= previous.tick {
                return Err(TimelineError::NonIncreasingTick {
                    previous: previous.tick,
                    next: tick.tick,
                });
            }
        }
        self.ticks.push(tick);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineError {
    TooManyEvents(Tick),
    NonIncreasingTick { previous: Tick, next: Tick },
}

impl fmt::Display for TimelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyEvents(tick) => write!(f, "too many events on tick {}", tick.0),
            Self::NonIncreasingTick { previous, next } => write!(
                f, "timeline tick must increase: {} then {}", previous.0, next.0
            ),
        }
    }
}

impl std::error::Error for TimelineError {}

/// Deterministic sequence source for fixtures. This is not Delphi-compatible
/// RNG; it pins random-call order until the original algorithm is recovered.
pub trait RandomSource {
    type Error;
    fn draw(&mut self, upper_exclusive: u32) -> Result<RandomSample, Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomSample {
    pub draw_index: u32,
    pub upper_exclusive: u32,
    pub value: u32,
}

/// Fixed sequence RNG for deterministic tests and replay fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureRng {
    values: Vec<u32>,
    cursor: usize,
}

impl FixtureRng {
    pub fn new(values: impl Into<Vec<u32>>) -> Self {
        Self { values: values.into(), cursor: 0 }
    }

    pub fn draws_consumed(&self) -> usize {
        self.cursor
    }

    pub fn remaining(&self) -> usize {
        self.values.len().saturating_sub(self.cursor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureRngError {
    Exhausted { draw_index: u32, upper_exclusive: u32 },
    OutOfRange { draw_index: u32, value: u32, upper_exclusive: u32 },
}

impl fmt::Display for FixtureRngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exhausted { draw_index, upper_exclusive } => write!(
                f, "fixture RNG exhausted at draw {} (requested [0, {}))",
                draw_index, upper_exclusive
            ),
            Self::OutOfRange { draw_index, value, upper_exclusive } => write!(
                f, "fixture RNG value {} at draw {} is outside [0, {})",
                value, draw_index, upper_exclusive
            ),
        }
    }
}

impl std::error::Error for FixtureRngError {}

impl RandomSource for FixtureRng {
    type Error = FixtureRngError;

    fn draw(&mut self, upper_exclusive: u32) -> Result<RandomSample, Self::Error> {
        let draw_index = u32::try_from(self.cursor).unwrap_or(u32::MAX);
        let value = *self.values.get(self.cursor).ok_or(
            FixtureRngError::Exhausted { draw_index, upper_exclusive }
        )?;

        if value >= upper_exclusive {
            return Err(FixtureRngError::OutOfRange {
                draw_index, value, upper_exclusive
            });
        }

        self.cursor += 1;
        Ok(RandomSample { draw_index, upper_exclusive, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_events_preserve_insertion_order() {
        let mut tick = TickEvents::new(Tick(4));
        assert_eq!(tick.push(CombatEvent::SoundQueued { sound_id: 14 }), Ok(0));
        assert_eq!(tick.push(CombatEvent::FrameChanged { frame: 37 }), Ok(1));
        assert_eq!(tick.events()[0].event, CombatEvent::SoundQueued { sound_id: 14 });
        assert_eq!(tick.events()[1].event, CombatEvent::FrameChanged { frame: 37 });
    }

    #[test]
    fn action_timeline_rejects_duplicate_or_reversed_ticks() {
        let mut timeline = ActionTimeline::new(FighterId(1), ActionId(3));
        timeline.push_tick(TickEvents::new(Tick(0))).unwrap();
        timeline.push_tick(TickEvents::new(Tick(1))).unwrap();
        assert!(matches!(
            timeline.push_tick(TickEvents::new(Tick(1))),
            Err(TimelineError::NonIncreasingTick { .. })
        ));
    }

    #[test]
    fn fixture_rng_consumes_values_sequentially() {
        let mut rng = FixtureRng::new(vec![2, 0, 3]);
        assert_eq!(rng.draw(4).unwrap(), RandomSample {
            draw_index: 0, upper_exclusive: 4, value: 2
        });
        assert_eq!(rng.draw(2).unwrap(), RandomSample {
            draw_index: 1, upper_exclusive: 2, value: 0
        });
        assert_eq!(rng.draw(5).unwrap(), RandomSample {
            draw_index: 2, upper_exclusive: 5, value: 3
        });
        assert_eq!(rng.draws_consumed(), 3);
        assert_eq!(rng.remaining(), 0);
    }

    #[test]
    fn fixture_rng_rejects_exhaustion_and_out_of_range_values() {
        let mut rng = FixtureRng::new(vec![5]);
        assert!(matches!(
            rng.draw(5),
            Err(FixtureRngError::OutOfRange { draw_index: 0, .. })
        ));
        assert_eq!(rng.draws_consumed(), 0);
        let mut empty = FixtureRng::new(Vec::<u32>::new());
        assert!(matches!(
            empty.draw(10),
            Err(FixtureRngError::Exhausted { draw_index: 0, .. })
        ));
    }

    #[test]
    fn box_state_changes_only_when_an_explicit_update_is_supplied() {
        let first = HitboxRecord {
            frame: 10,
            timer: 3,
            bounds: RawBounds { x1: 1, y1: 2, x2: 3, y2: 4 },
            active: true,
            strength: 1,
            damage: 7,
            guard_damage: 2,
        };
        let replacement = HitboxRecord { active: false, ..first };
        let mut state = BoxState::default();
        state.apply(&BoxUpdate::Replace(vec![first]));
        assert_eq!(state.rows(), &[first]);
        state.apply(&BoxUpdate::Retain);
        assert_eq!(state.rows(), &[first]);
        state.apply(&BoxUpdate::Replace(vec![replacement]));
        assert_eq!(state.rows(), &[replacement]);
        state.apply(&BoxUpdate::Clear);
        assert!(state.rows().is_empty());
    }
}
