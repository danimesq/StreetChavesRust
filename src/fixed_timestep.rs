//! Platform-neutral fixed-step scheduling for the game simulation.
//!
//! The host measures monotonic wall-clock time and feeds elapsed durations into
//! this module. The clock decides how many 28 ms simulation ticks are due; it
//! does not sleep, read a platform clock, collect input, or render frames.

use std::time::Duration;

/// Evidence-based logical simulation interval used by the original game.
pub const LOGICAL_TICK_DURATION: Duration = Duration::from_millis(28);

/// Integer nanoseconds keep the accumulator exact and avoid floating-point drift.
const LOGICAL_TICK_NANOS: u128 = 28_000_000;

/// Work made due by one elapsed-time update.
///
/// The host must execute every due tick before using interpolation_alpha to
/// render the newest simulation state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickBatch {
    /// Number of complete logical simulation ticks due.
    pub ticks_due: u128,
    /// Fraction of the next 28 ms tick currently accumulated, in [0, 1).
    ///
    /// This is a presentation hint only. Never feed it back into simulation.
    pub interpolation_alpha: f64,
}

/// Accumulates host elapsed time and schedules deterministic 28 ms updates.
///
/// Call advance with elapsed time from a monotonic host clock. A 144 Hz
/// renderer will therefore schedule fewer than one simulation tick per frame;
/// it cannot make the game run 144 times per second.
///
/// This clock deliberately has no catch-up cap. Every complete interval in
/// the supplied elapsed time becomes due work. Host adapters must handle
/// lifecycle pauses explicitly instead of feeding time spent suspended or
/// hidden into the simulation clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FixedTimestepClock {
    accumulated_nanos: u128,
    scheduled_ticks: u128,
}

impl FixedTimestepClock {
    /// Construct a clock with no accumulated time or scheduled ticks.
    pub const fn new() -> Self {
        Self {
            accumulated_nanos: 0,
            scheduled_ticks: 0,
        }
    }

    /// Add elapsed host time and report every complete simulation tick due.
    ///
    /// Hosts normally call this once per outer-loop/render callback, after
    /// draining and normalizing the available platform input events. For each
    /// returned tick, sample logical input once and advance the simulation once.
    ///
    /// Complete ticks are removed from the accumulator immediately. The
    /// fractional remainder is retained for the next call. No time is rounded
    /// to a display-frame interval and no tick is discarded.
    pub fn advance(&mut self, elapsed: Duration) -> TickBatch {
        self.accumulated_nanos = self.accumulated_nanos.saturating_add(elapsed.as_nanos());

        let ticks_due = self.accumulated_nanos / LOGICAL_TICK_NANOS;
        self.accumulated_nanos %= LOGICAL_TICK_NANOS;
        self.scheduled_ticks = self.scheduled_ticks.saturating_add(ticks_due);

        TickBatch {
            ticks_due,
            interpolation_alpha: self.interpolation_alpha(),
        }
    }

    /// Number of logical ticks scheduled since this clock was created.
    pub const fn scheduled_ticks(&self) -> u128 {
        self.scheduled_ticks
    }

    /// Unconsumed fraction of one logical tick.
    pub fn remainder(&self) -> Duration {
        // The accumulator invariant guarantees this value is below 28 ms.
        Duration::from_nanos(self.accumulated_nanos as u64)
    }

    /// Presentation-only fraction of the next logical tick, always in [0, 1).
    pub fn interpolation_alpha(&self) -> f64 {
        self.accumulated_nanos as f64 / LOGICAL_TICK_NANOS as f64
    }
}
