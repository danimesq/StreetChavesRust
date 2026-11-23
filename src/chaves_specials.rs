//! Source-derived C01 special-command entry and CPU decision logic.
//!
//! This module owns the character-specific [Acao] and [IA] portions only.
//! The existing update_c01_character_frame method remains the separate [Quadro] executor.

use crate::combat_runtime::{
    AiCommand, AiTickInputs, CharacterActionScript, CombatState, FighterState, RuntimeError,
    ACTION_CROUCHING, ACTION_DEFENDING, ACTION_IDLE, ACTION_JUMPING, ACTION_LOW_DEFENDING,
    ACTION_SPECIAL1, ACTION_SPECIAL2, ACTION_SPECIAL3, ACTION_SPECIAL4, ACTION_SPECIAL_FINAL,
    ACTION_STRONG_PUNCH, ACTION_WALKING, FORCE_SPECIAL, FORCE_STRONG, FORCE_WEAK,
};
use crate::pascal_replay::PascalRandomSource;
use std::fmt;

pub const CHARACTER_CHAVES: u8 = 1;
pub const SOUND_FINAL_SPECIAL: u16 = 19;

/// The nine C01 special command IDs declared in ChavesTipos.pas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChavesSpecialCommand {
    Esp1Weak,
    Esp1Strong,
    Esp2Weak,
    Esp2Strong,
    Esp3Weak,
    Esp3Strong,
    Esp4Weak,
    Esp4Strong,
    Final,
}

impl ChavesSpecialCommand {
    pub const fn pascal_code(self) -> u8 {
        match self {
            Self::Esp1Weak => 16,
            Self::Esp1Strong => 17,
            Self::Esp2Weak => 18,
            Self::Esp2Strong => 19,
            Self::Esp3Weak => 20,
            Self::Esp3Strong => 21,
            Self::Esp4Weak => 22,
            Self::Esp4Strong => 23,
            Self::Final => 24,
        }
    }

    pub const fn from_pascal_code(code: u8) -> Option<Self> {
        match code {
            16 => Some(Self::Esp1Weak),
            17 => Some(Self::Esp1Strong),
            18 => Some(Self::Esp2Weak),
            19 => Some(Self::Esp2Strong),
            20 => Some(Self::Esp3Weak),
            21 => Some(Self::Esp3Strong),
            22 => Some(Self::Esp4Weak),
            23 => Some(Self::Esp4Strong),
            24 => Some(Self::Final),
            _ => None,
        }
    }

    const fn is_weak(self) -> bool {
        matches!(
            self,
            Self::Esp1Weak | Self::Esp2Weak | Self::Esp3Weak | Self::Esp4Weak
        )
    }

    const fn is_special_one(self) -> bool {
        matches!(self, Self::Esp1Weak | Self::Esp1Strong)
    }

    const fn is_special_three(self) -> bool {
        matches!(self, Self::Esp3Weak | Self::Esp3Strong)
    }
}

/// Pascal globals consumed by the C01 [Acao] and [IA] sections.
///
/// Effect presence is not injected here: the Special 1 guard queries the current
/// live CombatState.effects slots at dispatch time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChavesSpecialContext {
    pub charge_gain: f64,
    pub charge_max: f64,
    pub ground_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChavesSpecialRejection {
    IneligibleAction,
    Effect7AlreadyExists,
    TooFarAboveGround,
    FinalUnavailable,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChavesSpecialEvent {
    CommonDefendingCleared,
    ActionChanged { value: u16 },
    ActionFrameChanged { value: u16 },
    ActionDurationChanged { value: u16 },
    ChargePowerChanged { value: f64 },
    PositionYChanged { value: f64 },
    InAirChanged { value: bool },
    VerticalVelocityChanged { value: i8 },
    JumpDirectionChanged { value: u8 },
    LowTimeChanged { value: u8 },
    SequenceStateChanged { value: u8 },
    PowerChanged { value: u16 },
    SpriteFrameChanged { value: i32 },
    ForceStrengthChanged { value: u8 },
    ScreenFlashChanged { value: u8 },
    PendingSoundChanged { value: u16 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChavesSpecialOutcome {
    /// True only when C01 [Acao] accepted the command and entered the special.
    pub applied: bool,
    pub rejection: Option<ChavesSpecialRejection>,
    /// Ordered writes from Common [Acao] followed by C01 [Acao].
    pub events: Vec<ChavesSpecialEvent>,
}

impl ChavesSpecialOutcome {
    fn rejected(reason: ChavesSpecialRejection) -> Self {
        Self {
            applied: false,
            rejection: Some(reason),
            events: Vec::new(),
        }
    }

    fn applied(events: Vec<ChavesSpecialEvent>) -> Self {
        Self {
            applied: true,
            rejection: None,
            events,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChavesSpecialError {
    FighterIndexOutOfRange(usize),
    NotChaves { actual_character_id: u8 },
    UnknownCommandCode(u8),
}

impl fmt::Display for ChavesSpecialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FighterIndexOutOfRange(index) => {
                write!(f, "fighter index {index} is outside the two-player state")
            }
            Self::NotChaves {
                actual_character_id,
            } => write!(
                f,
                "C01 special script cannot run for character id {actual_character_id}"
            ),
            Self::UnknownCommandCode(code) => {
                write!(f, "unknown C01 special command code {code}")
            }
        }
    }
}

impl std::error::Error for ChavesSpecialError {}

impl CombatState {
    /// Run only the character-specific C01 [Acao] branch.
    ///
    /// The caller is responsible for the preceding Common [Acao] and following
    /// Common [AcaoFim] stages. This method does not touch [Quadro].
    pub fn start_chaves_special(
        &mut self,
        fighter_index: usize,
        command: ChavesSpecialCommand,
        context: ChavesSpecialContext,
    ) -> Result<ChavesSpecialOutcome, ChavesSpecialError> {
        let sprite_offset = match fighter_index {
            0 => 0_i32,
            1 => 50_i32,
            index => return Err(ChavesSpecialError::FighterIndexOutOfRange(index)),
        };

        let fighter = self
            .fighters
            .get_mut(fighter_index)
            .ok_or(ChavesSpecialError::FighterIndexOutOfRange(fighter_index))?;

        if fighter.character_id != CHARACTER_CHAVES {
            return Err(ChavesSpecialError::NotChaves {
                actual_character_id: fighter.character_id,
            });
        }

        let current_action = fighter.action_id;
        let allowed = match command {
            ChavesSpecialCommand::Esp1Weak | ChavesSpecialCommand::Esp1Strong => matches!(
                current_action,
                ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING | ACTION_LOW_DEFENDING
            ),
            ChavesSpecialCommand::Esp2Weak | ChavesSpecialCommand::Esp2Strong => matches!(
                current_action,
                ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING | ACTION_LOW_DEFENDING
            ),
            ChavesSpecialCommand::Esp3Weak | ChavesSpecialCommand::Esp3Strong => matches!(
                current_action,
                ACTION_IDLE
                    | ACTION_WALKING
                    | ACTION_JUMPING
                    | ACTION_LOW_DEFENDING
                    | ACTION_SPECIAL2
            ),
            ChavesSpecialCommand::Esp4Weak | ChavesSpecialCommand::Esp4Strong => matches!(
                current_action,
                ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING | ACTION_LOW_DEFENDING
            ),
            ChavesSpecialCommand::Final => matches!(
                current_action,
                ACTION_IDLE
                    | ACTION_WALKING
                    | ACTION_STRONG_PUNCH
                    | ACTION_CROUCHING
                    | ACTION_DEFENDING
                    | ACTION_LOW_DEFENDING
            ),
        };

        if !allowed {
            return Ok(ChavesSpecialOutcome::rejected(
                ChavesSpecialRejection::IneligibleAction,
            ));
        }

        let owner = (fighter_index + 1) as u8;
        if command.is_special_one() && self.effects.contains_type_for_owner(7, owner) {
            return Ok(ChavesSpecialOutcome::rejected(
                ChavesSpecialRejection::Effect7AlreadyExists,
            ));
        }

        if command.is_special_three() && !(context.ground_y - 90.0 <= fighter.y) {
            return Ok(ChavesSpecialOutcome::rejected(
                ChavesSpecialRejection::TooFarAboveGround,
            ));
        }

        if command == ChavesSpecialCommand::Final
            && !(fighter.health <= 15.0 || fighter.power > 0)
        {
            return Ok(ChavesSpecialOutcome::rejected(
                ChavesSpecialRejection::FinalUnavailable,
            ));
        }

        let mut events = Vec::new();
        match command {
            ChavesSpecialCommand::Esp1Weak | ChavesSpecialCommand::Esp1Strong => {
                set_action(fighter, ACTION_SPECIAL1, &mut events);
                set_action_frame(fighter, 0, &mut events);
                set_action_duration(fighter, 28, &mut events);
                gain_charge(fighter, 5.0, context, &mut events);
                set_force(
                    fighter,
                    if command.is_weak() {
                        FORCE_WEAK
                    } else {
                        FORCE_STRONG
                    },
                    &mut events,
                );
            }
            ChavesSpecialCommand::Esp2Weak | ChavesSpecialCommand::Esp2Strong => {
                set_action(fighter, ACTION_SPECIAL2, &mut events);
                set_action_frame(fighter, 0, &mut events);
                gain_charge(fighter, 5.0, context, &mut events);
                set_action_duration(
                    fighter,
                    if command.is_weak() { 16 } else { 27 },
                    &mut events,
                );
                set_force(
                    fighter,
                    if command.is_weak() {
                        FORCE_WEAK
                    } else {
                        FORCE_STRONG
                    },
                    &mut events,
                );
            }
            ChavesSpecialCommand::Esp3Weak | ChavesSpecialCommand::Esp3Strong => {
                set_action(fighter, ACTION_SPECIAL3, &mut events);
                gain_charge(fighter, 6.0, context, &mut events);
                fighter.y = context.ground_y;
                events.push(ChavesSpecialEvent::PositionYChanged { value: fighter.y });
                fighter.in_air = false;
                events.push(ChavesSpecialEvent::InAirChanged { value: false });
                set_action_frame(fighter, 0, &mut events);
                fighter.vertical_velocity = 0;
                events.push(ChavesSpecialEvent::VerticalVelocityChanged { value: 0 });
                fighter.jump_direction = 0;
                events.push(ChavesSpecialEvent::JumpDirectionChanged { value: 0 });
                fighter.low_time = 0;
                events.push(ChavesSpecialEvent::LowTimeChanged { value: 0 });
                fighter.sequence_state = 0;
                events.push(ChavesSpecialEvent::SequenceStateChanged { value: 0 });
                set_action_duration(
                    fighter,
                    if command.is_weak() { 10 } else { 16 },
                    &mut events,
                );
                set_force(
                    fighter,
                    if command.is_weak() {
                        FORCE_WEAK
                    } else {
                        FORCE_STRONG
                    },
                    &mut events,
                );
            }
            ChavesSpecialCommand::Esp4Weak | ChavesSpecialCommand::Esp4Strong => {
                set_action(fighter, ACTION_SPECIAL4, &mut events);
                set_action_frame(fighter, 0, &mut events);
                gain_charge(fighter, 5.0, context, &mut events);
                set_action_duration(fighter, 22, &mut events);
                set_force(
                    fighter,
                    if command.is_weak() {
                        FORCE_WEAK
                    } else {
                        FORCE_STRONG
                    },
                    &mut events,
                );
            }
            ChavesSpecialCommand::Final => {
                set_action(fighter, ACTION_SPECIAL_FINAL, &mut events);
                set_action_frame(fighter, 0, &mut events);
                set_action_duration(fighter, 38, &mut events);
                if fighter.power == 3 {
                    fighter.charge_power = 0.0;
                    events.push(ChavesSpecialEvent::ChargePowerChanged { value: 0.0 });
                }
                // Pascal Poder is Word, so the assignment wraps at 16 bits.
                fighter.power = fighter.power.wrapping_sub(1);
                events.push(ChavesSpecialEvent::PowerChanged {
                    value: fighter.power,
                });
                fighter.sprite_frame = sprite_offset + 36;
                events.push(ChavesSpecialEvent::SpriteFrameChanged {
                    value: fighter.sprite_frame,
                });
                set_force(fighter, FORCE_SPECIAL, &mut events);
                self.screen_flash = 22;
                events.push(ChavesSpecialEvent::ScreenFlashChanged { value: 22 });
                self.pending_sound = Some(SOUND_FINAL_SPECIAL);
                events.push(ChavesSpecialEvent::PendingSoundChanged {
                    value: SOUND_FINAL_SPECIAL,
                });
            }
        }

        Ok(ChavesSpecialOutcome::applied(events))
    }

    /// Dispatch a Pascal Codigo through Common [Acao] and then C01 [Acao].
    ///
    /// The common section always clears P.Defendendo before the character branch,
    /// even when the requested special is subsequently rejected.
    pub fn dispatch_chaves_special(
        &mut self,
        fighter_index: usize,
        command: ChavesSpecialCommand,
        context: ChavesSpecialContext,
    ) -> Result<ChavesSpecialOutcome, ChavesSpecialError> {
        let fighter = self
            .fighters
            .get_mut(fighter_index)
            .ok_or(ChavesSpecialError::FighterIndexOutOfRange(fighter_index))?;
        fighter.defending = false;

        let mut outcome = self.start_chaves_special(fighter_index, command, context)?;
        outcome.events.insert(0, ChavesSpecialEvent::CommonDefendingCleared);
        Ok(outcome)
    }

    /// Numeric command entry point for controller/input adapters and ROT dispatchers.
    pub fn dispatch_chaves_special_code(
        &mut self,
        fighter_index: usize,
        command_code: u8,
        context: ChavesSpecialContext,
    ) -> Result<ChavesSpecialOutcome, ChavesSpecialError> {
        let command = ChavesSpecialCommand::from_pascal_code(command_code)
            .ok_or(ChavesSpecialError::UnknownCommandCode(command_code))?;
        self.dispatch_chaves_special(fighter_index, command, context)
    }
}

fn set_action(fighter: &mut FighterState, value: u16, events: &mut Vec<ChavesSpecialEvent>) {
    fighter.action_id = value;
    events.push(ChavesSpecialEvent::ActionChanged { value });
}

fn set_action_frame(
    fighter: &mut FighterState,
    value: u16,
    events: &mut Vec<ChavesSpecialEvent>,
) {
    fighter.action_frame = value;
    events.push(ChavesSpecialEvent::ActionFrameChanged { value });
}

fn set_action_duration(
    fighter: &mut FighterState,
    value: u16,
    events: &mut Vec<ChavesSpecialEvent>,
) {
    fighter.action_duration = value;
    events.push(ChavesSpecialEvent::ActionDurationChanged { value });
}

fn set_force(fighter: &mut FighterState, value: u8, events: &mut Vec<ChavesSpecialEvent>) {
    fighter.force_strength = value;
    events.push(ChavesSpecialEvent::ForceStrengthChanged { value });
}

fn gain_charge(
    fighter: &mut FighterState,
    amount: f64,
    context: ChavesSpecialContext,
    events: &mut Vec<ChavesSpecialEvent>,
) {
    // The Pascal guard checks the old value only and does not clamp the new value.
    if fighter.charge_power < context.charge_max {
        fighter.charge_power += amount * context.charge_gain;
        events.push(ChavesSpecialEvent::ChargePowerChanged {
            value: fighter.charge_power,
        });
    }
}

/// Stateful adapter for the C01 [IA] section.
#[derive(Debug, Clone, Copy)]
pub struct ChavesCharacterActionScript {
    context: ChavesSpecialContext,
}

impl ChavesCharacterActionScript {
    pub const fn new(context: ChavesSpecialContext) -> Self {
        Self { context }
    }

    fn dispatch_ai_special<R: PascalRandomSource>(
        &mut self,
        state: &mut CombatState,
        fighter_index: usize,
        command: ChavesSpecialCommand,
        commands: &mut Vec<AiCommand>,
    ) -> Result<(), RuntimeError<R::Error>> {
        // AcaoJogador is called even when the character [Acao] gate rejects the move.
        commands.push(AiCommand::ChavesSpecial(command));
        state
            .dispatch_chaves_special(fighter_index, command, self.context)
            .map_err(|_| RuntimeError::UnsupportedBranch("C01 [IA] special dispatcher failed"))?;
        Ok(())
    }

    fn roll_special_strength<R: PascalRandomSource>(
        &mut self,
        state: &mut CombatState,
        fighter_index: usize,
        weak: ChavesSpecialCommand,
        strong: ChavesSpecialCommand,
        rng: &mut R,
        commands: &mut Vec<AiCommand>,
    ) -> Result<(), RuntimeError<R::Error>> {
        let strength = rng.draw_longint(2).map_err(RuntimeError::Rng)?;
        let command = match strength {
            0 => weak,
            1 => strong,
            _ => return Err(RuntimeError::UnsupportedBranch("C01 [IA] Random(2) out of range")),
        };
        self.dispatch_ai_special::<R>(state, fighter_index, command, commands)
    }
}

impl CharacterActionScript for ChavesCharacterActionScript {
    fn execute<R: PascalRandomSource>(
        &mut self,
        _state: &mut CombatState,
        _fighter_index: usize,
        _command: AiCommand,
        _rng: &mut R,
    ) -> Result<(), RuntimeError<R::Error>> {
        // Common movement/attack commands have no additional C01 [Acao] writes here.
        Ok(())
    }

    fn execute_character_ai<R: PascalRandomSource>(
        &mut self,
        state: &mut CombatState,
        fighter_index: usize,
        input: AiTickInputs,
        rng: &mut R,
        commands: &mut Vec<AiCommand>,
    ) -> Result<(), RuntimeError<R::Error>> {
        let Some(fighter) = state.fighters.get(fighter_index) else {
            return Err(RuntimeError::UnsupportedBranch("C01 [IA] invalid fighter index"));
        };
        if fighter.character_id != CHARACTER_CHAVES || fighter.defending {
            return Ok(());
        }

        // C01.rot [IA] line 188: projectile threat, then Random(2) strength choice.
        if input.threat == 1 && input.projectile_distance > 140 {
            if random_zero_longint(rng, 7.0, input.difficulty_factor, 188)? {
                self.roll_special_strength(
                    state,
                    fighter_index,
                    ChavesSpecialCommand::Esp1Weak,
                    ChavesSpecialCommand::Esp1Strong,
                    rng,
                    commands,
                )?;
            }
        }

        // C01.rot [IA] line 190: long range without a threat.
        if input.threat == 0 && input.distance > 120 {
            if random_zero_longint(rng, 30.0, input.difficulty_factor, 190)? {
                self.roll_special_strength(
                    state,
                    fighter_index,
                    ChavesSpecialCommand::Esp1Weak,
                    ChavesSpecialCommand::Esp1Strong,
                    rng,
                    commands,
                )?;
            }
        }

        // C01.rot [IA] line 192: close-range Special 2.
        if input.threat == 0 && input.distance < 200 {
            if random_zero_longint(rng, 40.0, input.difficulty_factor, 192)? {
                self.roll_special_strength(
                    state,
                    fighter_index,
                    ChavesSpecialCommand::Esp2Weak,
                    ChavesSpecialCommand::Esp2Strong,
                    rng,
                    commands,
                )?;
            }
        }

        // C01.rot [IA] line 194: Special 3 when the opponent is above ground.
        if input.distance < 150 && self.context.ground_y > f64::from(input.opponent_y) {
            if random_zero_longint(rng, 50.0, input.difficulty_factor, 194)? {
                self.roll_special_strength(
                    state,
                    fighter_index,
                    ChavesSpecialCommand::Esp3Weak,
                    ChavesSpecialCommand::Esp3Strong,
                    rng,
                    commands,
                )?;
            }
        }

        // C01.rot [IA] line 196: independent Special 4 probability check.
        if input.threat == 0 && input.distance < 200 {
            if random_zero_longint(rng, 40.0, input.difficulty_factor, 196)? {
                self.roll_special_strength(
                    state,
                    fighter_index,
                    ChavesSpecialCommand::Esp4Weak,
                    ChavesSpecialCommand::Esp4Strong,
                    rng,
                    commands,
                )?;
            }
        }

        // C01.rot [IA] line 197: the final move has no Random(2) strength selection.
        if input.distance < 150 && self.context.ground_y > f64::from(input.opponent_y) {
            if random_zero_longint(rng, 50.0, input.difficulty_factor, 197)? {
                self.dispatch_ai_special::<R>(
                    state,
                    fighter_index,
                    ChavesSpecialCommand::Final,
                    commands,
                )?;
            }
        }

        Ok(())
    }
}

/// Free Pascal Round is ties-to-even. Each Random(Round(...)) uses the LongInt overload.
fn random_zero_longint<R: PascalRandomSource>(
    rng: &mut R,
    multiplier: f64,
    factor: f64,
    source_line: u32,
) -> Result<bool, RuntimeError<R::Error>> {
    let value = multiplier * factor;
    if !value.is_finite() || value < i32::MIN as f64 || value > i32::MAX as f64 {
        return Err(RuntimeError::InvalidProbabilityRange {
            source_line,
            range: 0,
        });
    }

    let floor = value.floor();
    let fraction = value - floor;
    let rounded = if fraction < 0.5 {
        floor
    } else if fraction > 0.5 {
        floor + 1.0
    } else if (floor as i64).rem_euclid(2) == 0 {
        floor
    } else {
        floor + 1.0
    };
    let range = rounded as i32;
    if range <= 0 {
        return Err(RuntimeError::InvalidProbabilityRange {
            source_line,
            range: i64::from(range),
        });
    }

    Ok(rng.draw_longint(range).map_err(RuntimeError::Rng)? == 0)
}
