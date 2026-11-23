//! Evidence-driven contracts for the Street Chaves simulation core.
//!
//! These types preserve source order and source data. The crate now models the
//! pinned Pascal/C box-update rules and Delphi 5 integer RNG recurrence. The
//! original executable's exact selector choice, initial RandSeed, and complete
//! random-call stream remain unverified.

use std::fmt;

/// Parse and conservatively reconcile concrete Pascal JSONL traces with static symbolic sites.
pub mod trace_join;

/// Strict replay of concrete RNG requests from Pascal schema-v2 captures.
pub mod pascal_replay;

/// Differential support for concrete Pascal tick checkpoints.
#[rustfmt::skip]
pub mod pascal_differential;

/// Bounded Pascal-derived AI runtime: target selection, movement, close-range attacks and child RNG.
pub mod combat_runtime;

/// Fixed-capacity live effect instances and source-ordered update snapshots.
pub mod effect_runtime;

/// C01 special [Acao] entry and character-specific [IA] dispatch.
pub mod chaves_specials;
/// Pascal-style direction/button command recognition primitives.
pub mod command_recognition;

/// Pinned per-character [Botoes]/[Sequencias] data and INI rule loader.
pub mod fighter_command_data;

/// Host-independent logical input sampling with edge and hold-state tracking.
pub mod input;

/// Platform-neutral fixed-step scheduling at the evidence-based 28 ms interval.
pub mod fixed_timestep;

/// Shared headless bridge from normalized host input to logical simulation ticks.
pub mod runtime;

/// Concrete renderer-free fight orchestrator implementing SimulationTick.
pub mod fight_simulation;

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

impl RawBounds {
    /// Mirror the horizontal axis using the endpoint swap used by both ports.
    pub fn mirrored_horizontally(self) -> Self {
        Self {
            x1: -self.x2,
            y1: self.y1,
            x2: -self.x1,
            y2: self.y2,
        }
    }
}

/// Attack-box row. `selector` is the Pascal action key or the C state key;
/// keep the source namespace alongside provenance when storing extracted rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitboxRecord {
    pub selector: u16,
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

/// World-space axis-aligned bounds using the strict comparisons observed in
/// both reference combat paths. Edge contact alone is not an overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Aabb {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Aabb {
    pub fn overlaps_strict(self, other: Self) -> bool {
        self.left < other.right
            && self.right > other.left
            && self.top < other.bottom
            && self.bottom > other.top
    }
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxState<T> {
    rows: Vec<T>,
}

impl<T> Default for BoxState<T> {
    fn default() -> Self {
        Self { rows: Vec::new() }
    }
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

    /// Reproduce a source lookup that scans all matching rows and writes one
    /// field set for each match: the last match wins; no match leaves old data.
    ///
    /// This matches the observed C selector's table scan. Callers must pass
    /// rows in their original table order and must not pre-sort them.
    pub fn apply_last_match_or_retain(
        &mut self,
        matching_rows: impl IntoIterator<Item = T>,
    ) -> bool {
        let mut last = None;
        for row in matching_rows {
            last = Some(row);
        }

        match last {
            Some(row) => {
                self.rows.clear();
                self.rows.push(row);
                true
            }
            None => false,
        }
    }
}

impl BoxState<HitboxRecord> {
    /// Mirror the explicit active-flag clear used after guard/hit/state changes.
    /// Geometry, force, damage, and guard-damage metadata remain stored.
    pub fn deactivate_attack(&mut self) {
        for row in &mut self.rows {
            row.active = false;
        }
    }
}

/// One Pascal [Ataque] row, retaining its two distinct strength concepts:
/// the optional strength selector condition and the row's optional force assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PascalAttackDefinition {
    pub action: ActionId,
    pub action_timer: u16,
    /// None represents a generic row with no strength condition.
    pub strength_condition: Option<u8>,
    pub bounds: RawBounds,
    pub active: bool,
    /// Zero means "leave the fighter's current force unchanged".
    pub assigned_strength: u8,
    pub damage: u16,
    pub guard_damage: u16,
}

/// The mutable fields touched by Pascal's ordered DefineCaixaAtaque calls.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PascalAttackState {
    pub current_force: u8,
    pub current_box: Option<HitboxRecord>,
}

impl PascalAttackState {
    /// Apply matching rows in INI order, exactly as AplicarCaixas does.
    ///
    /// The strength condition is checked against the current force at that
    /// point in the loop. A previous generic row can therefore change which
    /// later strength-specific row matches. No matching row means no write.
    pub fn apply_ordered_rows(
        &mut self,
        action: ActionId,
        action_timer: u16,
        rows: &[PascalAttackDefinition],
    ) -> usize {
        let mut applied = 0;

        for row in rows {
            if row.action != action || row.action_timer != action_timer {
                continue;
            }
            if row
                .strength_condition
                .is_some_and(|required| required != self.current_force)
            {
                continue;
            }

            self.current_box = Some(HitboxRecord {
                selector: row.action.0,
                timer: row.action_timer,
                bounds: row.bounds,
                active: row.active,
                strength: row.assigned_strength,
                damage: row.damage,
                guard_damage: row.guard_damage,
            });

            if row.assigned_strength != 0 {
                self.current_force = row.assigned_strength;
            }
            applied += 1;
        }

        applied
    }
}

/// Ordered event emitted by one action on one logical tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatEvent {
    FrameChanged {
        frame: u16,
    },
    SoundQueued {
        sound_id: u16,
    },
    EffectSpawned {
        effect_id: u16,
        variant: Option<char>,
    },
    /// Generic until the final typed player-state schema is evidence-backed.
    StateMutation {
        field: String,
        value: i32,
    },
    RandomDraw {
        upper_exclusive: u32,
        value: u32,
        draw_index: u64,
    },
    /// Explicitly disables a stored attack without erasing its coordinates.
    AttackDeactivated,
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
        Self {
            tick,
            events: Vec::new(),
        }
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
        Self {
            fighter,
            action,
            ticks: Vec::new(),
        }
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
                f,
                "timeline tick must increase: {} then {}",
                previous.0, next.0
            ),
        }
    }
}

impl std::error::Error for TimelineError {}

/// Common interface for deterministic test sequences and compatibility RNGs.

pub trait RandomSource {
    type Error;
    fn draw(&mut self, upper_exclusive: u32) -> Result<RandomSample, Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomSample {
    pub draw_index: u64,
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
        Self {
            values: values.into(),
            cursor: 0,
        }
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
    Exhausted {
        draw_index: u64,
        upper_exclusive: u32,
    },
    OutOfRange {
        draw_index: u64,
        value: u32,
        upper_exclusive: u32,
    },
}

impl fmt::Display for FixtureRngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exhausted {
                draw_index,
                upper_exclusive,
            } => write!(
                f,
                "fixture RNG exhausted at draw {} (requested [0, {}))",
                draw_index, upper_exclusive
            ),
            Self::OutOfRange {
                draw_index,
                value,
                upper_exclusive,
            } => write!(
                f,
                "fixture RNG value {} at draw {} is outside [0, {})",
                value, draw_index, upper_exclusive
            ),
        }
    }
}

impl std::error::Error for FixtureRngError {}

impl RandomSource for FixtureRng {
    type Error = FixtureRngError;

    fn draw(&mut self, upper_exclusive: u32) -> Result<RandomSample, Self::Error> {
        let draw_index = u64::try_from(self.cursor).unwrap_or(u64::MAX);
        let value = *self
            .values
            .get(self.cursor)
            .ok_or(FixtureRngError::Exhausted {
                draw_index,
                upper_exclusive,
            })?;

        if value >= upper_exclusive {
            return Err(FixtureRngError::OutOfRange {
                draw_index,
                value,
                upper_exclusive,
            });
        }

        self.cursor += 1;
        Ok(RandomSample {
            draw_index,
            upper_exclusive,
            value,
        })
    }
}

/// FPC 3.2.2 System.Random compatibility layer used by the normative Pascal source.
///
/// The pinned FPC 3.2.2 RTL implements MTwist (MT19937, 624-word state).
/// Random(LongInt) consumes one 32-bit MT word and applies multiply-high range
/// reduction. Random(Int64) consumes two MT words, composes a non-negative
/// 63-bit value, then applies modulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PascalRandomWidth {
    LongInt,
    Int64,
}

/// One Pascal System.Random call, including how many raw MT words it consumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PascalRandomSample {
    pub call_index: u64,
    pub first_word_index: u64,
    pub words_consumed: u8,
    pub width: PascalRandomWidth,
    pub range: i64,
    pub value: i64,
}

/// Free Pascal 3.2.2 MTwist implementation with injectable RandSeed.
///
/// Randomize's platform-specific seed source is deliberately kept outside this
/// type. The FPC RTL complements RandSeed after detecting a reseed; this
/// implementation preserves that behavior to match subsequent comparisons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fpc322Rng {
    state: [u32; 624],
    index: usize,
    rand_seed: u32,
    old_rand_seed: u32,
    calls: u64,
    words: u64,
}

impl Fpc322Rng {
    const N: usize = 624;
    const M: usize = 397;
    const MATRIX_A: u32 = 0x9908_B0DF;
    const UPPER_MASK: u32 = 0x8000_0000;
    const LOWER_MASK: u32 = 0x7FFF_FFFF;

    /// Match a fresh FPC process with RandSeed set to seed.
    pub fn new(seed: i32) -> Self {
        Self {
            state: [0; Self::N],
            index: Self::N + 1,
            rand_seed: seed as u32,
            old_rand_seed: 0,
            calls: 0,
            words: 0,
        }
    }

    pub fn rand_seed(&self) -> u32 {
        self.rand_seed
    }

    /// Number of System.Random calls (not raw MT words).
    pub fn calls_consumed(&self) -> u64 {
        self.calls
    }

    /// Number of underlying 32-bit MT words consumed.
    pub fn words_consumed(&self) -> u64 {
        self.words
    }

    /// Assign RandSeed as Pascal code can. FPC notices a changed value on the
    /// next random call, except for the RTL's documented sentinel edge case.
    pub fn set_rand_seed(&mut self, seed: i32) {
        self.rand_seed = seed as u32;
    }

    fn init_state(&mut self, seed: u32) {
        self.state[0] = seed;
        for i in 1..Self::N {
            let previous = self.state[i - 1];
            self.state[i] = 1_812_433_253u32
                .wrapping_mul(previous ^ (previous >> 30))
                .wrapping_add(i as u32);
        }
        self.index = Self::N;
    }

    #[inline]
    fn twist_word(a: u32, b: u32) -> u32 {
        let mixed = (a & Self::UPPER_MASK) | (b & Self::LOWER_MASK);
        (mixed >> 1) ^ if mixed & 1 != 0 { Self::MATRIX_A } else { 0 }
    }

    /// Preserve the in-place loop order in FPC 3.2.2's mtwist_update_state.
    fn update_state(&mut self) {
        for i in 0..(Self::N - Self::M) {
            self.state[i] =
                self.state[i + Self::M] ^ Self::twist_word(self.state[i], self.state[i + 1]);
        }
        for i in (Self::N - Self::M)..(Self::N - 1) {
            self.state[i] = self.state[i - (Self::N - Self::M)]
                ^ Self::twist_word(self.state[i], self.state[i + 1]);
        }
        self.state[Self::N - 1] =
            self.state[Self::M - 1] ^ Self::twist_word(self.state[Self::N - 1], self.state[0]);
        self.index = 0;
    }

    /// One raw MT word, following FPC's RandSeed/OldRandSeed logic.
    pub fn next_u32(&mut self) -> u32 {
        let mut word_index = self.index;
        self.index = self.index.saturating_add(1);

        if self.rand_seed != self.old_rand_seed || word_index >= Self::N + 1 {
            self.init_state(self.rand_seed);
            self.rand_seed = !self.rand_seed;
            self.old_rand_seed = self.rand_seed;
            word_index = Self::N;
        }

        if word_index == Self::N {
            self.update_state();
            word_index = 0;
            self.index = 1;
        }

        let mut value = self.state[word_index];
        value ^= value >> 11;
        value ^= (value << 7) & 0x9D2C_5680;
        value ^= (value << 15) & 0xEFC6_0000;
        value ^= value >> 18;

        self.words = self.words.saturating_add(1);
        value
    }

    /// FPC 3.2.2 Random(LongInt): one MT word and multiply-high reduction.
    pub fn random_longint(&mut self, range: i32) -> PascalRandomSample {
        let call_index = self.calls;
        let first_word_index = self.words;
        let word = self.next_u32();
        let adjusted_range = if range < 0 {
            range.wrapping_add(1)
        } else {
            range
        };
        let product_bits = (word as i64).wrapping_mul(adjusted_range as i64) as u64;
        let value = (product_bits >> 32) as i32 as i64;
        self.calls = self.calls.saturating_add(1);
        PascalRandomSample {
            call_index,
            first_word_index,
            words_consumed: 1,
            width: PascalRandomWidth::LongInt,
            range: range as i64,
            value,
        }
    }

    /// FPC 3.2.2 Random(Int64): two MT words, 63-bit assembly, then modulo.
    pub fn random_int64(&mut self, range: i64) -> PascalRandomSample {
        let call_index = self.calls;
        let first_word_index = self.words;
        let low = self.next_u32() as u64;
        let high = self.next_u32() as u64;
        let assembled = low | ((high << 32) & i64::MAX as u64);
        let value = if range == 0 {
            0
        } else {
            (assembled as i64) % range
        };
        self.calls = self.calls.saturating_add(1);
        PascalRandomSample {
            call_index,
            first_word_index,
            words_consumed: 2,
            width: PascalRandomWidth::Int64,
            range,
            value,
        }
    }
}

impl RandomSource for Fpc322Rng {
    type Error = std::convert::Infallible;

    /// The shared trait exposes the 32-bit Pascal overload. Call random_int64
    /// explicitly for expressions that ChavesRoteiro marks as Largo=true.
    fn draw(&mut self, upper_exclusive: u32) -> Result<RandomSample, Self::Error> {
        assert!(
            upper_exclusive <= i32::MAX as u32,
            "FPC Random(LongInt) range must fit in LongInt"
        );
        let sample = self.random_longint(upper_exclusive as i32);
        Ok(RandomSample {
            draw_index: sample.first_word_index,
            upper_exclusive,
            value: sample.value as u32,
        })
    }
}

/// Experimental Delphi 5 LCG candidate reconstructed from the C comparison port.
/// This is not the normative RNG for StreetChavesRust; Pascal/FPC 3.2.2 is normative.
///
/// The state transition is modulo 2^32:
/// RandSeed := RandSeed * $08088405 + 1
/// and Random(N) returns the high 32 bits of next_seed * N.
/// Seed initialization is deliberately separate: Randomize's time-derived
/// seed is not equivalent to supplying a reproducible captured RandSeed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DelphiRng {
    rand_seed: u32,
    draws: u64,
}

impl DelphiRng {
    pub const MULTIPLIER: u32 = 0x0808_8405;

    pub fn new(rand_seed: u32) -> Self {
        Self {
            rand_seed,
            draws: 0,
        }
    }

    pub fn rand_seed(&self) -> u32 {
        self.rand_seed
    }

    pub fn draws_consumed(&self) -> u64 {
        self.draws
    }

    /// Advance the 32-bit Delphi LCG once and return the new seed bits.
    pub fn next_u32(&mut self) -> u32 {
        self.rand_seed = self
            .rand_seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(1);
        self.draws = self.draws.saturating_add(1);
        self.rand_seed
    }

    /// Delphi's Random(N) for the valid non-negative signed LongInt range.
    /// The source runtime accepts 0..=i32::MAX; game callers use small ranges.
    ///
    /// A zero range still advances RandSeed, matching the Delphi 5 assembly:
    /// the seed update occurs before the multiply-high result is produced.
    pub fn random_range(&mut self, upper_exclusive: u32) -> u32 {
        assert!(
            upper_exclusive <= i32::MAX as u32,
            "Delphi 5 Random range must fit in a non-negative LongInt"
        );
        let next = self.next_u32() as u64;
        ((next * upper_exclusive as u64) >> 32) as u32
    }
}

impl RandomSource for DelphiRng {
    type Error = std::convert::Infallible;

    fn draw(&mut self, upper_exclusive: u32) -> Result<RandomSample, Self::Error> {
        let draw_index = self.draws;
        let value = self.random_range(upper_exclusive);
        Ok(RandomSample {
            draw_index,
            upper_exclusive,
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fpc_322_rng_matches_mt19937_reference_words() {
        let mut rng = Fpc322Rng::new(5489);
        assert_eq!(rng.next_u32(), 3_499_211_612);
        assert_eq!(rng.next_u32(), 581_869_302);
        assert_eq!(rng.next_u32(), 3_890_346_734);
        assert_eq!(rng.words_consumed(), 3);
    }

    #[test]
    fn fpc_322_random_longint_consumes_one_word_with_multiply_high() {
        let mut rng = Fpc322Rng::new(5489);
        let sample = rng.random_longint(8);
        assert_eq!(sample.value, 6);
        assert_eq!(sample.words_consumed, 1);
        assert_eq!(sample.first_word_index, 0);
        assert_eq!(rng.words_consumed(), 1);
        assert_eq!(rng.calls_consumed(), 1);
    }

    #[test]
    fn fpc_322_random_int64_consumes_two_words_then_uses_modulo() {
        let mut rng = Fpc322Rng::new(5489);
        let sample = rng.random_int64(1000);
        assert_eq!(sample.value, 4);
        assert_eq!(sample.words_consumed, 2);
        assert_eq!(sample.first_word_index, 0);
        assert_eq!(rng.words_consumed(), 2);
        assert_eq!(rng.calls_consumed(), 1);
    }

    #[test]
    fn fpc_322_zero_ranges_still_consume_the_overload_word_count() {
        let mut longint_rng = Fpc322Rng::new(5489);
        assert_eq!(longint_rng.random_longint(0).value, 0);
        assert_eq!(longint_rng.words_consumed(), 1);

        let mut int64_rng = Fpc322Rng::new(5489);
        assert_eq!(int64_rng.random_int64(0).value, 0);
        assert_eq!(int64_rng.words_consumed(), 2);
    }

    #[test]
    fn fpc_322_reseed_restarts_the_reference_sequence() {
        let mut rng = Fpc322Rng::new(5489);
        let first = rng.next_u32();
        rng.set_rand_seed(5489);
        assert_eq!(rng.next_u32(), first);
    }

    #[test]
    fn delphi_rng_matches_delphi_5_range_vectors() {
        let mut rng = DelphiRng::new(0);
        assert_eq!(rng.draw(10).unwrap().value, 0);
        assert_eq!(rng.rand_seed(), 0x0000_0001);
        assert_eq!(rng.draw(100).unwrap().value, 3);
        assert_eq!(rng.rand_seed(), 0x0808_8406);
        assert_eq!(rng.draw(2).unwrap().value, 1);
        assert_eq!(rng.rand_seed(), 0xDC6D_AC1F);
        assert_eq!(rng.draw(120).unwrap().value, 24);
        assert_eq!(rng.rand_seed(), 0x33DC_589C);
        assert_eq!(rng.draws_consumed(), 4);
    }

    #[test]
    fn delphi_rng_zero_range_still_advances_the_seed() {
        let mut rng = DelphiRng::new(0);
        assert_eq!(rng.random_range(0), 0);
        assert_eq!(rng.rand_seed(), 1);
        assert_eq!(rng.draws_consumed(), 1);
    }

    #[test]
    fn explicit_attack_deactivation_preserves_geometry_and_damage_metadata() {
        let active = HitboxRecord {
            selector: 27,
            timer: 1,
            bounds: RawBounds {
                x1: 10,
                y1: 130,
                x2: 82,
                y2: 158,
            },
            active: true,
            strength: 2,
            damage: 18,
            guard_damage: 3,
        };
        let deactivated = HitboxRecord {
            active: false,
            ..active
        };
        let mut state = BoxState::default();

        assert!(state.apply_last_match_or_retain([active]));
        state.deactivate_attack();

        assert_eq!(state.rows(), &[deactivated]);
        assert_eq!(state.rows()[0].bounds, active.bounds);
        assert_eq!(state.rows()[0].damage, active.damage);
        assert_eq!(state.rows()[0].guard_damage, active.guard_damage);
        assert_eq!(state.rows()[0].strength, active.strength);
    }

    #[test]
    fn last_matching_box_row_wins_and_no_match_retains_previous_value() {
        let first = HitboxRecord {
            selector: 27,
            timer: 1,
            bounds: RawBounds {
                x1: 10,
                y1: 130,
                x2: 82,
                y2: 158,
            },
            active: true,
            strength: 0,
            damage: 18,
            guard_damage: 3,
        };
        let second = HitboxRecord {
            bounds: RawBounds {
                x1: 10,
                y1: 25,
                x2: 70,
                y2: 55,
            },
            ..first
        };
        let mut state = BoxState::default();

        assert!(state.apply_last_match_or_retain([first, second]));
        assert_eq!(state.rows(), &[second]);
        assert!(!state.apply_last_match_or_retain(Vec::<HitboxRecord>::new()));
        assert_eq!(state.rows(), &[second]);
    }

    #[test]
    fn pascal_strength_variants_are_filtered_in_ini_order() {
        let bounds_weak = RawBounds {
            x1: 10,
            y1: 130,
            x2: 82,
            y2: 158,
        };
        let bounds_strong = RawBounds {
            x1: 10,
            y1: 25,
            x2: 70,
            y2: 55,
        };
        let rows = [
            PascalAttackDefinition {
                action: ActionId(27),
                action_timer: 1,
                strength_condition: Some(1),
                bounds: bounds_weak,
                active: true,
                assigned_strength: 0,
                damage: 18,
                guard_damage: 3,
            },
            PascalAttackDefinition {
                action: ActionId(27),
                action_timer: 1,
                strength_condition: Some(2),
                bounds: bounds_strong,
                active: true,
                assigned_strength: 0,
                damage: 18,
                guard_damage: 3,
            },
        ];

        let mut weak = PascalAttackState {
            current_force: 1,
            current_box: None,
        };
        assert_eq!(weak.apply_ordered_rows(ActionId(27), 1, &rows), 1);
        assert_eq!(weak.current_box.as_ref().unwrap().bounds, bounds_weak);

        let mut strong = PascalAttackState {
            current_force: 2,
            current_box: None,
        };
        assert_eq!(strong.apply_ordered_rows(ActionId(27), 1, &rows), 1);
        assert_eq!(strong.current_box.as_ref().unwrap().bounds, bounds_strong);
    }

    #[test]
    fn earlier_pascal_row_force_assignment_changes_later_variant_match() {
        let generic = PascalAttackDefinition {
            action: ActionId(9),
            action_timer: 4,
            strength_condition: None,
            bounds: RawBounds {
                x1: 0,
                y1: 1,
                x2: 2,
                y2: 3,
            },
            active: false,
            assigned_strength: 2,
            damage: 0,
            guard_damage: 0,
        };
        let strong_variant = PascalAttackDefinition {
            strength_condition: Some(2),
            bounds: RawBounds {
                x1: 4,
                y1: 5,
                x2: 6,
                y2: 7,
            },
            active: true,
            assigned_strength: 0,
            damage: 9,
            guard_damage: 1,
            ..generic
        };
        let mut state = PascalAttackState {
            current_force: 1,
            current_box: None,
        };

        assert_eq!(
            state.apply_ordered_rows(ActionId(9), 4, &[generic, strong_variant]),
            2
        );
        assert_eq!(state.current_force, 2);
        assert_eq!(
            state.current_box.as_ref().unwrap().bounds,
            strong_variant.bounds
        );
        assert!(state.current_box.as_ref().unwrap().active);
    }

    #[test]
    fn pascal_no_matching_attack_row_retains_the_previous_hitbox() {
        let previous = HitboxRecord {
            selector: 27,
            timer: 1,
            bounds: RawBounds {
                x1: 10,
                y1: 130,
                x2: 82,
                y2: 158,
            },
            active: true,
            strength: 0,
            damage: 18,
            guard_damage: 3,
        };
        let mut state = PascalAttackState {
            current_force: 1,
            current_box: Some(previous),
        };
        assert_eq!(state.apply_ordered_rows(ActionId(4), 2, &[]), 0);
        assert_eq!(state.current_box, Some(previous));
    }

    #[test]
    fn raw_bounds_mirror_by_swapping_and_negating_horizontal_endpoints() {
        let bounds = RawBounds {
            x1: 10,
            y1: 25,
            x2: 70,
            y2: 55,
        };
        assert_eq!(
            bounds.mirrored_horizontally(),
            RawBounds {
                x1: -70,
                y1: 25,
                x2: -10,
                y2: 55
            }
        );
    }

    #[test]
    fn tick_events_preserve_insertion_order() {
        let mut tick = TickEvents::new(Tick(4));
        assert_eq!(tick.push(CombatEvent::SoundQueued { sound_id: 14 }), Ok(0));
        assert_eq!(tick.push(CombatEvent::FrameChanged { frame: 37 }), Ok(1));
        assert_eq!(
            tick.events()[0].event,
            CombatEvent::SoundQueued { sound_id: 14 }
        );
        assert_eq!(
            tick.events()[1].event,
            CombatEvent::FrameChanged { frame: 37 }
        );
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
        assert_eq!(
            rng.draw(4).unwrap(),
            RandomSample {
                draw_index: 0,
                upper_exclusive: 4,
                value: 2
            }
        );
        assert_eq!(
            rng.draw(2).unwrap(),
            RandomSample {
                draw_index: 1,
                upper_exclusive: 2,
                value: 0
            }
        );
        assert_eq!(
            rng.draw(5).unwrap(),
            RandomSample {
                draw_index: 2,
                upper_exclusive: 5,
                value: 3
            }
        );
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
    fn strict_aabb_overlap_rejects_edge_contact_and_accepts_one_unit_overlap() {
        let first = Aabb {
            left: 0,
            top: 0,
            right: 10,
            bottom: 10,
        };
        let touching = Aabb {
            left: 10,
            top: 0,
            right: 20,
            bottom: 10,
        };
        let overlap = Aabb {
            left: 9,
            top: 0,
            right: 20,
            bottom: 10,
        };
        assert!(!first.overlaps_strict(touching));
        assert!(first.overlaps_strict(overlap));
    }

    #[test]
    fn box_state_changes_only_when_an_explicit_update_is_supplied() {
        let first = HitboxRecord {
            selector: 10,
            timer: 3,
            bounds: RawBounds {
                x1: 1,
                y1: 2,
                x2: 3,
                y2: 4,
            },
            active: true,
            strength: 1,
            damage: 7,
            guard_damage: 2,
        };
        let replacement = HitboxRecord {
            active: false,
            ..first
        };
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
