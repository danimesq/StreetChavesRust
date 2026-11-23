//! First concrete, renderer-free fight orchestrator.
use crate::chaves_specials::{ChavesSpecialCommand, ChavesSpecialContext, ChavesSpecialOutcome};
use crate::command_recognition::{AttackButton, CommandRecognizer};
use crate::combat_runtime::{
    C01FrameResult, CombatState, VictoryAnimationContext, ACTION_POSE, ACTION_SPECIAL1,
    ACTION_SPECIAL2, ACTION_SPECIAL3, ACTION_SPECIAL4, ACTION_SPECIAL_FINAL, ACTION_VICTORY,
};
use crate::effect_runtime::{EffectTickContext, EffectTickResult};
use crate::fighter_command_data::{embedded_character_command_rules, CharacterCommandRules};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandMatch {
    pub fighter_index: usize,
    pub button: AttackButton,
    pub command_code: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnhandledCommandSource {
    CharacterButtonRule,
    BasicButtonAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnhandledCommand {
    pub fighter_index: usize,
    pub button: AttackButton,
    pub command_code: u8,
    pub source: UnhandledCommandSource,
}

/// One headless simulation tick's observable work.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FightTickReport {
    pub tick_index: u64,
    /// Backwards-compatible shorthand for the player-one input sample.
    pub input: InputFrame,
    pub fighter_inputs: [InputFrame; 2],
    pub command_matches: Vec<CommandMatch>,
    /// Commands requiring common/non-C01 action implementations not yet ported.
    pub unhandled_commands: Vec<UnhandledCommand>,
    pub sequence_transitions: Vec<(usize, u8, u8)>,
    pub special_outcomes: Vec<(usize, ChavesSpecialOutcome)>,
    pub c01_frames: Vec<(usize, C01FrameResult)>,
    pub effects: EffectTickResult,
    pub errors: Vec<String>,
}

/// Concrete SimulationTick implementation over the existing combat/effect state.
#[derive(Debug, Clone)]
pub struct FightSimulation {
    pub state: CombatState,
    config: FightSimulationConfig,
    rng: Fpc322Rng,
    pending_specials: Vec<(usize, ChavesSpecialCommand)>,
    command_rules: [Option<CharacterCommandRules>; 2],
    command_recognizers: [CommandRecognizer; 2],
    command_data_errors: Vec<String>,
    last_report: FightTickReport,
}

impl FightSimulation {
    /// Build a deterministic headless fight and load real per-character command tables.
    pub fn new(state: CombatState, config: FightSimulationConfig, rng_seed: i32) -> Self {
        let mut command_rules: [Option<CharacterCommandRules>; 2] = [None, None];
        let mut command_data_errors = Vec::new();
        for fighter_index in 0..2 {
            let character_id = state.fighters[fighter_index].character_id;
            match embedded_character_command_rules(character_id) {
                Ok(rules) => command_rules[fighter_index] = Some(rules),
                Err(error) => command_data_errors.push(error.to_string()),
            }
        }
        Self {
            state,
            config,
            rng: Fpc322Rng::new(rng_seed),
            pending_specials: Vec::new(),
            command_rules,
            command_recognizers: [CommandRecognizer::default(); 2],
            command_data_errors,
            last_report: FightTickReport::default(),
        }
    }

    /// Queue an explicit C01 command for the next non-stopped simulation tick.
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

    pub fn last_report(&self) -> &FightTickReport {
        &self.last_report
    }

    pub fn rng_calls_consumed(&self) -> u64 {
        self.rng.calls_consumed()
    }

    /// Run with independent normalized input frames for both fighters.
    /// The SimulationTick trait currently supplies player one and neutral player two.
    pub fn tick_with_inputs(&mut self, inputs: [InputFrame; 2]) {
        self.run_tick(inputs);
    }

    fn dispatch_character_command(
        &mut self,
        fighter_index: usize,
        button: AttackButton,
        command_code: u8,
        report: &mut FightTickReport,
    ) {
        report.command_matches.push(CommandMatch {
            fighter_index,
            button,
            command_code,
        });

        // This is the only per-character action dispatcher currently implemented:
        // Chaves (C01), commands cmEsp1..cmEspFinal (16..24).
        if self.state.fighters[fighter_index].character_id == 1
            && ChavesSpecialCommand::from_pascal_code(command_code).is_some()
        {
            match self.state.dispatch_chaves_special_code(
                fighter_index,
                command_code,
                self.config.special,
            ) {
                Ok(outcome) => report.special_outcomes.push((fighter_index, outcome)),
                Err(error) => report.errors.push(format!(
                    "C01 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else {
            report.unhandled_commands.push(UnhandledCommand {
                fighter_index,
                button,
                command_code,
                source: UnhandledCommandSource::CharacterButtonRule,
            });
        }
    }

    fn dispatch_input_commands(
        &mut self,
        inputs: [InputFrame; 2],
        report: &mut FightTickReport,
    ) {
        // Pascal L7_Entrada handles fighter 1, then fighter 2. Special-table rows
        // for each button are all dispatched in file order before the basic action.
        for fighter_index in 0..2 {
            let input = inputs[fighter_index];
            let Some(button_rules) = self.command_rules[fighter_index]
                .as_ref()
                .map(|rules| rules.buttons.clone())
            else {
                continue;
            };
            for button in [
                AttackButton::WeakPunch,
                AttackButton::WeakKick,
                AttackButton::StrongPunch,
                AttackButton::StrongKick,
            ] {
                let commands = self.command_recognizers[fighter_index].commands_for_button(
                    &self.state.fighters[fighter_index],
                    input,
                    button,
                    &button_rules,
                    self.config.special.ground_y,
                );
                for command_code in commands {
                    self.dispatch_character_command(
                        fighter_index,
                        button,
                        command_code,
                        report,
                    );
                }
            }

            // Pascal always sends the corresponding ordinary punch/kick after
            // processing [Botoes]. The common action pipeline is not ported yet,
            // so report these IDs rather than silently pretending to execute them.
            for (button, command_code) in [
                (AttackButton::WeakPunch, 3),
                (AttackButton::WeakKick, 4),
                (AttackButton::StrongPunch, 5),
                (AttackButton::StrongKick, 6),
            ] {
                if input.was_pressed(button.input_button()) {
                    report.unhandled_commands.push(UnhandledCommand {
                        fighter_index,
                        button,
                        command_code,
                        source: UnhandledCommandSource::BasicButtonAction,
                    });
                }
            }
        }
    }

    fn update_command_sequences(
        &mut self,
        inputs: [InputFrame; 2],
        report: &mut FightTickReport,
    ) {
        for fighter_index in 0..2 {
            let input = inputs[fighter_index];
            let Some(rules) = self.command_rules[fighter_index].as_ref() else {
                continue;
            };
            let recognizer = self.command_recognizers[fighter_index];
            recognizer.update_counters(&mut self.state.fighters[fighter_index], input);
            let transitions = recognizer.apply_sequences(
                &mut self.state.fighters[fighter_index],
                input,
                &rules.sequences,
                self.config.special.ground_y,
            );
            report.sequence_transitions.extend(
                transitions
                    .into_iter()
                    .map(|(from, to)| (fighter_index, from, to)),
            );
            recognizer.tick_sequence_timeout(
                &mut self.state.fighters[fighter_index],
                self.state.screen_flash != 0 || self.state.clock_stopped,
            );
        }
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

    fn run_tick(&mut self, inputs: [InputFrame; 2]) {
        let mut report = FightTickReport {
            tick_index: self.state.tick_index,
            input: inputs[0],
            fighter_inputs: inputs,
            errors: self.command_data_errors.clone(),
            ..FightTickReport::default()
        };

        // L7_Entrada is gated by hit-stop and screen flash. Sequence tracking still
        // runs afterwards, matching sequenciagolpe's placement in the Pascal loop.
        if !self.state.clock_stopped && self.state.screen_flash == 0 {
            self.dispatch_input_commands(inputs, &mut report);
        }
        self.update_command_sequences(inputs, &mut report);

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
        self.run_tick([input, InputFrame::default()]);
    }
}
