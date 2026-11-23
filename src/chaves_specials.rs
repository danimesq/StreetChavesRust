//! C01 (Chaves) special-command entry behavior from the pinned Pascal `[Acao]` script.
//!
//! This module deliberately stops at the command-entry boundary. Per-frame `[Quadro]`
//! behavior, real effect-slot lookup/creation, hit resolution and audio playback are
//! separate systems and are not simulated here.

use crate::combat_runtime::{
    CombatState, FighterState, ACTION_CROUCHING, ACTION_DEFENDING, ACTION_IDLE,
    ACTION_JUMPING, ACTION_LOW_DEFENDING, ACTION_STRONG_PUNCH, ACTION_WALKING,
};
use std::fmt;

pub const CHARACTER_CHAVES: u8 = 1;

pub const ACTION_SPECIAL_1: u16 = 25;
pub const ACTION_SPECIAL_2: u16 = 26;
pub const ACTION_SPECIAL_3: u16 = 27;
pub const ACTION_SPECIAL_4: u16 = 28;
pub const ACTION_SPECIAL_5: u16 = 29;

pub const STRENGTH_NONE: u8 = 0;
pub const STRENGTH_WEAK: u8 = 1;
pub const STRENGTH_STRONG: u8 = 2;
pub const STRENGTH_SPECIAL: u8 = 3;

pub const JUMP_DIRECTION_NONE: u8 = 0;
pub const SOUND_FINAL_SPECIAL: u8 = 19;

/// The nine special commands understood by Chaves's C01 `[Acao]` section.
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
    /// Pascal `cm*` identifier from ChavesTipos.pas.
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

    const fn is_weak(self) -> bool {
        matches!(self, Self::Esp1Weak | Self::Esp2Weak | Self::Esp3Weak | Self::Esp4Weak)
    }
}

/// Values read by the Pascal script from game globals or the existing effect table.
///
/// `effect_7_exists` must be the result of the real `ExisteEfeito(7, Jog)` query.
/// It is an input seam, not a replacement effect collection or a synthetic effect queue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChavesSpecialContext {
    pub charge_gain: f64,
    pub charge_max: f64,
    pub ground_y: u16,
    pub effect_7_exists: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChavesSpecialRejection {
    IneligibleAction,
    Effect7AlreadyExists,
    TooFarAboveGround,
    FinalUnavailable,
}

/// Ordered writes performed by C01 `[Acao]`.
///
/// This records only entry-time mutations. Frame changes and effects/sounds which
/// happen later in `[Quadro]` are intentionally absent, except for the final move's
/// immediate frame, flash and pending sound writes.
#[derive(Debug, Clone, PartialEq)]
pub enum ChavesSpecialEvent {
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
    SpriteFrameChanged { value: u8 },
    AttackStrengthChanged { value: u8 },
    ScreenFlashChanged { value: u8 },
    PendingSoundChanged { value: u8 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChavesSpecialOutcome {
    pub applied: bool,
    pub rejection: Option<ChavesSpecialRejection>,
    /// Events preserve the order of assignments in the Pascal script.
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
}

impl fmt::Display for ChavesSpecialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FighterIndexOutOfRange(index) => {
                write!(f, "fighter index {index} is outside the two-player state")
            }
            Self::NotChaves { actual_character_id } => write!(
                f,
                "C01 special script cannot run for character id {actual_character_id}"
            ),
        }
    }
}

impl std::error::Error for ChavesSpecialError {}

impl CombatState {
    /// Apply one Chaves special command's C01 `[Acao]` transition.
    ///
    /// `fighter_index` is zero-based. The Pascal sprite offset (`Desloc`) is 0
    /// for player 1 and 50 for player 2. This method does not run `[Quadro]`.
    pub fn start_chaves_special(
        &mut self,
        fighter_index: usize,
        command: ChavesSpecialCommand,
        context: ChavesSpecialContext,
    ) -> Result<ChavesSpecialOutcome, ChavesSpecialError> {
        let desloc = match fighter_index {
            0 => 0_u8,
            1 => 50_u8,
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

        let action = fighter.action_id;
        let allowed = match command {
            ChavesSpecialCommand::Esp1Weak | ChavesSpecialCommand::Esp1Strong => {
                matches!(
                    action,
                    ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING | ACTION_LOW_DEFENDING
                )
            }
            ChavesSpecialCommand::Esp2Weak | ChavesSpecialCommand::Esp2Strong => {
                matches!(
                    action,
                    ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING | ACTION_LOW_DEFENDING
                )
            }
            ChavesSpecialCommand::Esp3Weak | ChavesSpecialCommand::Esp3Strong => {
                matches!(
                    action,
                    ACTION_IDLE
                        | ACTION_WALKING
                        | ACTION_JUMPING
                        | ACTION_LOW_DEFENDING
                        | ACTION_SPECIAL_2
                )
            }
            ChavesSpecialCommand::Esp4Weak | ChavesSpecialCommand::Esp4Strong => {
                matches!(
                    action,
                    ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING | ACTION_LOW_DEFENDING
                )
            }
            ChavesSpecialCommand::Final => matches!(
                action,
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

        if matches!(
            command,
            ChavesSpecialCommand::Esp1Weak | ChavesSpecialCommand::Esp1Strong
        ) && context.effect_7_exists
        {
            return Ok(ChavesSpecialOutcome::rejected(
                ChavesSpecialRejection::Effect7AlreadyExists,
            ));
        }

        if matches!(
            command,
            ChavesSpecialCommand::Esp3Weak | ChavesSpecialCommand::Esp3Strong
        ) && (f64::from(context.ground_y) - 90.0 > fighter.y)
        {
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
                set_action(fighter, ACTION_SPECIAL_1, &mut events);
                set_action_frame(fighter, 0, &mut events);
                set_action_duration(fighter, 28, &mut events);
                gain_charge(fighter, 5.0, context, &mut events);
                set_attack_strength(
                    fighter,
                    if command.is_weak() {
                        STRENGTH_WEAK
                    } else {
                        STRENGTH_STRONG
                    },
                    &mut events,
                );
            }
            ChavesSpecialCommand::Esp2Weak | ChavesSpecialCommand::Esp2Strong => {
                set_action(fighter, ACTION_SPECIAL_2, &mut events);
                set_action_frame(fighter, 0, &mut events);
                gain_charge(fighter, 5.0, context, &mut events);
                set_action_duration(
                    fighter,
                    if command.is_weak() { 16 } else { 27 },
                    &mut events,
                );
                set_attack_strength(
                    fighter,
                    if command.is_weak() {
                        STRENGTH_WEAK
                    } else {
                        STRENGTH_STRONG
                    },
                    &mut events,
                );
            }
            ChavesSpecialCommand::Esp3Weak | ChavesSpecialCommand::Esp3Strong => {
                set_action(fighter, ACTION_SPECIAL_3, &mut events);
                gain_charge(fighter, 6.0, context, &mut events);
                fighter.y = f64::from(context.ground_y);
                events.push(ChavesSpecialEvent::PositionYChanged { value: fighter.y });
                fighter.in_air = false;
                events.push(ChavesSpecialEvent::InAirChanged { value: false });
                set_action_frame(fighter, 0, &mut events);
                fighter.vertical_velocity = 0;
                events.push(ChavesSpecialEvent::VerticalVelocityChanged { value: 0 });
                fighter.jump_direction = JUMP_DIRECTION_NONE;
                events.push(ChavesSpecialEvent::JumpDirectionChanged {
                    value: JUMP_DIRECTION_NONE,
                });
                fighter.low_time = 0;
                events.push(ChavesSpecialEvent::LowTimeChanged { value: 0 });
                fighter.sequence_state = 0;
                events.push(ChavesSpecialEvent::SequenceStateChanged { value: 0 });
                set_action_duration(
                    fighter,
                    if command.is_weak() { 10 } else { 16 },
                    &mut events,
                );
                set_attack_strength(
                    fighter,
                    if command.is_weak() {
                        STRENGTH_WEAK
                    } else {
                        STRENGTH_STRONG
                    },
                    &mut events,
                );
            }
            ChavesSpecialCommand::Esp4Weak | ChavesSpecialCommand::Esp4Strong => {
                set_action(fighter, ACTION_SPECIAL_4, &mut events);
                set_action_frame(fighter, 0, &mut events);
                gain_charge(fighter, 5.0, context, &mut events);
                set_action_duration(fighter, 22, &mut events);
                set_attack_strength(
                    fighter,
                    if command.is_weak() {
                        STRENGTH_WEAK
                    } else {
                        STRENGTH_STRONG
                    },
                    &mut events,
                );
            }
            ChavesSpecialCommand::Final => {
                set_action(fighter, ACTION_SPECIAL_5, &mut events);
                set_action_frame(fighter, 0, &mut events);
                set_action_duration(fighter, 38, &mut events);

                if fighter.power == 3 {
                    fighter.charge_power = 0.0;
                    events.push(ChavesSpecialEvent::ChargePowerChanged { value: 0.0 });
                }

                // Pascal Poder is Word. The script's assignment converts the
                // arithmetic result back to Word, so preserve 16-bit wraparound.
                fighter.power = fighter.power.wrapping_sub(1);
                events.push(ChavesSpecialEvent::PowerChanged {
                    value: fighter.power,
                });

                fighter.sprite_frame = 36 + desloc;
                events.push(ChavesSpecialEvent::SpriteFrameChanged {
                    value: fighter.sprite_frame,
                });
                set_attack_strength(fighter, STRENGTH_SPECIAL, &mut events);

                self.screen_flash = 22;
                events.push(ChavesSpecialEvent::ScreenFlashChanged { value: 22 });
                self.pending_sound = SOUND_FINAL_SPECIAL;
                events.push(ChavesSpecialEvent::PendingSoundChanged {
                    value: SOUND_FINAL_SPECIAL,
                });
            }
        }

        Ok(ChavesSpecialOutcome::applied(events))
    }
}

fn set_action(
    fighter: &mut FighterState,
    value: u16,
    events: &mut Vec<ChavesSpecialEvent>,
) {
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

fn set_attack_strength(
    fighter: &mut FighterState,
    value: u8,
    events: &mut Vec<ChavesSpecialEvent>,
) {
    fighter.attack_strength = value;
    events.push(ChavesSpecialEvent::AttackStrengthChanged { value });
}

fn gain_charge(
    fighter: &mut FighterState,
    amount: f64,
    context: ChavesSpecialContext,
    events: &mut Vec<ChavesSpecialEvent>,
) {
    // The Pascal guard checks the old value only. It does not clamp the new value.
    if fighter.charge_power < context.charge_max {
        fighter.charge_power += amount * context.charge_gain;
        events.push(ChavesSpecialEvent::ChargePowerChanged {
            value: fighter.charge_power,
        });
    }
}
