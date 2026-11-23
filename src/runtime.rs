//! Shared headless execution boundary for native and browser hosts.
//!
//! SDL, browser and standalone-WASM adapters normalize their physical input
//! events, feed them to this runner, and pass elapsed monotonic time to
//! advance(). The runner samples input once per due logical tick and invokes
//! the same simulation interface for every platform.

use std::time::Duration;

use crate::fixed_timestep::{FixedTimestepClock, TickBatch};
use crate::input::{InputEvent, InputFrame, InputSampler};

/// One authoritative simulation update for one logical 28 ms tick.
///
/// Implement this trait on the game state/orchestrator. Implementations must
/// advance exactly one logical simulation step per call and must not derive
/// gameplay time from rendering cadence or read a platform clock.
pub trait SimulationTick {
    /// Advance the authoritative simulation using one sampled logical input.
    fn tick(&mut self, input: InputFrame);
}

/// Headless bridge between host events, the 28 ms clock and the game simulation.
///
/// A host applies every normalized input event as it drains its platform event
/// queue, then calls advance() with elapsed monotonic time. Rendering remains
/// outside this type and may run independently after each host callback.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FixedStepRunner {
    clock: FixedTimestepClock,
    input: InputSampler,
}

impl FixedStepRunner {
    /// Create an idle runner with a fresh clock and neutral input state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply one normalized event from keyboard, controller, or browser input.
    ///
    /// Physical key names, scancodes, DOM events and controller IDs must be
    /// translated by the host before they cross this boundary.
    pub fn handle_input_event(&mut self, event: InputEvent) {
        self.input.apply_event(event);
    }

    /// Advance the simulation once for each complete 28 ms interval due.
    ///
    /// Input events should be applied before this call. If no tick is due,
    /// pending pressed/released edges remain in the sampler until the next
    /// simulation tick. During catch-up, the first tick receives those edges
    /// and later ticks receive the current held state with advanced hold counts.
    ///
    /// The returned interpolation fraction is for presentation only. This
    /// method never renders, sleeps, reads a clock, or accesses platform APIs.
    pub fn advance<S: SimulationTick>(
        &mut self,
        elapsed: Duration,
        simulation: &mut S,
    ) -> TickBatch {
        let batch = self.clock.advance(elapsed);

        for _ in 0..batch.ticks_due {
            let input = self.input.sample_tick();
            simulation.tick(input);
        }

        batch
    }

    /// Number of logical simulation ticks scheduled since construction.
    pub fn scheduled_ticks(&self) -> u128 {
        self.clock.scheduled_ticks()
    }

    /// Fractional time retained until the next logical tick is due.
    pub fn remainder(&self) -> Duration {
        self.clock.remainder()
    }
}
