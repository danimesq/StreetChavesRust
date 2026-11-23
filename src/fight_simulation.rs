//! First concrete, renderer-free fight orchestrator.
//!
//! This slice composes existing source-backed C01 frame logic and the live
//! effect arena behind the shared 28 ms SimulationTick interface. It does not
//! claim a complete Pascal fight loop: unsupported common physics, action-end,
//! collision, command-recognition and non-C01 frame paths remain explicit gates.

use crate::chaves_specials::{ChavesSpecialCommand, ChavesSpecialContext, ChavesSpecialOutcome};
use crate::combat_runtime::{
    C01FrameResult, CombatState, VictoryAnimationContext, ACTION_POSE, ACTION_SPECIAL1,
    ACTION_SPECIAL2, ACTION_SPECIAL3, ACTION_SPECIAL4, ACTION_SPECIAL_FINAL, ACTION_VICTORY,
};
use crate::effect_runtime::{EffectTickContext, EffectTickResult};
use crate::input::InputFrame;
use crate::runtime::SimulationTick;
use crate::Fpc322Rng;

/// Configuration for the already-audited C01 entry and frame executors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FightSimulationConfig {
    pub special: ChavesSpecialContext,
    pub frame: VictoryAnimationContext,
    /// Camera Y used by the existing effect culling contract.
    pub effect_camera_y: f64,
}

/// One headless simulation tick's observable work.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FightTickReport {
    pub tick_index: u64,
    pub input: InputFrame,
    pub special_outcomes: Vec<(usize, ChavesSpecialOutcome)>,
    pub c01_frames: Vec<(usize, C01FrameResult)>,
    pub effects: EffectTickResult,
    /// Runtime errors are retained for the host/test harness; no panic or fake fallback.
    pub errors: Vec<String>,
}

/// Concrete SimulationTick implementation over the existing combat/effect state.
#[derive(Debug, Clone)]
pub struct FightSimulation {
    pub state: CombatState,
    config: FightSimulationConfig,
    rng: Fpc322Rng,
    pending_specials: Vec<(usize, ChavesSpecialCommand)>,
    last_report: FightTickReport,
}

impl FightSimulation {
    /// Build a deterministic headless fight around an existing two-fighter state.
    pub fn new(state: CombatState, config: FightSimulationConfig, rng_seed: i32) -> Self {
        Self {
            state,
            config,
            rng: Fpc322Rng::new(rng_seed),
            pending_specials: Vec::new(),
            last_report: FightTickReport::default(),
        }
    }

    /// Queue an explicit C01 command for the next non-stopped simulation tick.
    ///
    /// This is intentionally not a physical-button mapping. The source-audited
    /// command recognizer and host key/controller mapping are separate work.
    pub fn queue_chaves_special(
        &mut self,
        fighter_index: usize,
        command: ChavesSpecialCommand,
    ) -> bool {
        if fighter_index >= self.state.fighters.len()
            || self.state.fighters[fighter_index].character_id != 1
        {
            return false;
        }
        self.pending_specials.push((fighter_index, command));
        true
    }

    /// Last completed tick report, including frame/effect snapshots and errors.
    pub fn last_report(&self) -> &FightTickReport {
        &self.last_report
    }

    /// Number of FPC Random API calls consumed by the current deterministic stream.
    pub fn rng_calls_consumed(&self) -> u64 {
        self.rng.calls_consumed()
    }

    fn advance_c01_frame(&mut self, fighter_index: usize, report: &mut FightTickReport) {
        if self.state.fighters[fighter_index].character_id != 1 {
            return;
        }

        let action_id = self.state.fighters[fighter_index].action_id;
        let advances_frame = matches!(
            action_id,
            ACTION_SPECIAL1
                | ACTION_SPECIAL2
                | ACTION_SPECIAL3
                | ACTION_SPECIAL4
                | ACTION_SPECIAL_FINAL
                | ACTION_VICTORY
                | ACTION_POSE
        );
        if !advances_frame {
            return;
        }

        let mut context = self.config.frame;
        context.sprite_offset = (fighter_index as i32) * 50;
        context.camera_x = self.state.camera_x;
        let frame_before = self.state.fighters[fighter_index].action_frame;

        match self
            .state
            .update_c01_character_frame(fighter_index, context, &mut self.rng)
        {
            Ok(frame_result) => {
                report.c01_frames.push((fighter_index, frame_result));
                // The source [Quadro] executor owns frame writes; the outer
                // logical tick advances the timer once, unless the script reset it.
                let fighter = &mut self.state.fighters[fighter_index];
                if fighter.action_frame == frame_before
                    && fighter.action_frame < fighter.action_duration
                {
                    fighter.action_frame = fighter.action_frame.saturating_add(1);
                }
            }
            Err(error) => report
                .errors
                .push(format!("C01 [Quadro] fighter {fighter_index}: {error}")),
        }
    }

    fn run_tick(&mut self, input: InputFrame) {
        let mut report = FightTickReport {
            tick_index: self.state.tick_index,
            input,
            ..FightTickReport::default()
        };

        // A stopped fight does not consume queued commands or advance fighter frames.
        if !self.state.clock_stopped {
            for (fighter_index, command) in std::mem::take(&mut self.pending_specials) {
                match self.state.dispatch_chaves_special(
                    fighter_index,
                    command,
                    self.config.special,
                ) {
                    Ok(outcome) => report.special_outcomes.push((fighter_index, outcome)),
                    Err(error) => report
                        .errors
                        .push(format!("C01 [Acao] fighter {fighter_index}: {error}")),
                }
            }

            for fighter_index in 0..self.state.fighters.len() {
                self.advance_c01_frame(fighter_index, &mut report);
            }
        }

        // The live arena is advanced exactly once after fighter [Quadro] work.
        // Collision consequences are deliberately not fabricated here.
        let effect_context = EffectTickContext {
            flash_active: self.state.screen_flash != 0,
            hit_stop_active: self.state.clock_stopped,
            camera_x: self.state.camera_x,
            camera_y: self.config.effect_camera_y,
        };
        report.effects = self.state.tick_effects(effect_context);

        self.state.tick_index = self.state.tick_index.saturating_add(1);
        self.last_report = report;
    }
}

impl SimulationTick for FightSimulation {
    fn tick(&mut self, input: InputFrame) {
        self.run_tick(input);
    }
}
