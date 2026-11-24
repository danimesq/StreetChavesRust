//! First executable Pascal-derived combat runtime slice.
//!
//! This implements the source-ordered Common.rot [IA] path for target selection,
//! movement, and the close-range probability chain. Character-specific ROT execution
//! remains an injected child-script seam; unsupported behavior is not fabricated.

use crate::chaves_specials::{
    ChavesCharacterActionScript, ChavesSpecialCommand, ChavesSpecialContext,
};
use crate::effect_runtime::{
    EffectArena, EffectSpawnError, EffectTickContext, EffectTickResult, EFFECT_SIDE_RIGHT,
};
use crate::pascal_replay::PascalRandomSource;
use serde_json::json;
use std::fmt;

pub const ACTION_IDLE: u16 = 1;
pub const ACTION_WALKING: u16 = 2;
pub const ACTION_WEAK_PUNCH: u16 = 3;
pub const ACTION_WEAK_KICK: u16 = 4;
pub const ACTION_STRONG_PUNCH: u16 = 5;
pub const ACTION_STRONG_KICK: u16 = 6;
pub const ACTION_CROUCHING: u16 = 7;
pub const ACTION_CROUCH_WEAK_PUNCH: u16 = 8;
pub const ACTION_CROUCH_WEAK_KICK: u16 = 9;
pub const ACTION_CROUCH_STRONG_PUNCH: u16 = 10;
pub const ACTION_CROUCH_STRONG_KICK: u16 = 11;
pub const ACTION_JUMPING: u16 = 12;
pub const ACTION_AIR_WEAK_PUNCH: u16 = 13;
pub const ACTION_AIR_WEAK_KICK: u16 = 14;
pub const ACTION_AIR_STRONG_PUNCH: u16 = 15;
pub const ACTION_AIR_STRONG_KICK: u16 = 16;
pub const ACTION_DEFENDING: u16 = 21;
pub const ACTION_LOW_DEFENDING: u16 = 22;
pub const ACTION_VICTORY: u16 = 23;
/// Pascal `acPose`, verified from ChavesTipos.pas at the pinned source revision.
pub const ACTION_POSE: u16 = 24;
pub const ACTION_SPECIAL1: u16 = 25;
pub const ACTION_SPECIAL2: u16 = 26;
pub const ACTION_SPECIAL3: u16 = 27;
pub const ACTION_SPECIAL4: u16 = 28;
pub const ACTION_SPECIAL_FINAL: u16 = 29;
/// Pascal `acEspecial6` (30), a continuation state used by selected fighter scripts.
pub const ACTION_SPECIAL6: u16 = 30;
pub const FORCE_NONE: u8 = 0;
pub const FORCE_WEAK: u8 = 1;
pub const FORCE_STRONG: u8 = 2;
pub const FORCE_SPECIAL: u8 = 3;
pub const ACTION_BOUNCING: u16 = 20;
pub const SIDE_RIGHT: u8 = 1;
pub const SIDE_LEFT: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyBox {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FighterState {
    /// Pascal P.Quadro, independent from the action timer.
    pub sprite_frame: i32,
    pub character_id: u8,
    pub action_id: u16,
    pub action_frame: u16,
    pub action_duration: u16,
    pub facing: u8,
    pub force_strength: u8,
    pub defending: bool,
    pub invulnerable: bool,
    pub hit_count: u8,
    /// Pascal TLutador.TempoAcertos (combo timer, Byte).
    pub hit_timer: u8,
    /// Pascal TLutador.DanoGolpe (Byte) consumed by collision resolution.
    pub attack_damage: u8,
    /// Pascal TLutador.VelAngular (SmallInt) used for spinning knockback.
    pub angular_velocity: i16,
    /// Pascal `TLutador.GolpeAtivo`: whether this fighter's current attack may still hit.
    /// This is fighter state, not the `active` metadata stored in a hitbox row.
    pub attack_active: bool,
    /// Pascal `TLutador.AndandoFrente`, used by the common walking [Quadro] branch.
    pub walking_forward: bool,
    /// Last selected Pascal [Ataque] row. A selector miss retains this record.
    pub attack_box: Option<crate::HitboxRecord>,
    pub is_kick: bool,
    pub recoil: u8,
    pub vertical_velocity: i8,
    pub in_air: bool,
    pub x: f64,
    pub y: f64,
    pub speed: f64,
    pub distance_ai: i32,
    /// Pascal TLutador.Vida is Double; preserve the C01 low-health gate in floating point.
    pub health: f64,
    /// Pascal TLutador.Poder (Word), used by C01's final special.
    pub power: u16,
    /// Pascal TLutador.CargaPoder (Double).
    pub charge_power: f64,
    /// Pascal TLutador.DirPulo, TempoBaixo and SeqEstado (Byte).
    pub jump_direction: u8,
    pub low_time: u8,
    /// Pascal TLutador.TempoTras.
    pub back_time: u8,
    pub sequence_state: u8,
    /// Pascal TLutador.SeqTempo (9 ticks after a matching sequence transition).
    pub sequence_timer: u8,
    /// Pascal TLutador.CargaSoco and CargaChute; no producer is inferred by this slice.
    pub punch_charge: u8,
    pub kick_charge: u8,
    pub body: BodyBox,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VictoryAnimationContext {
    /// Character sprite-sheet displacement (Pascal Desloc).
    pub sprite_offset: i32,
    /// Ground Y coordinate (Pascal ChaoY).
    pub ground_y: f64,
    /// Camera origin (Pascal CameraX).
    pub camera_x: f64,
    /// Camera-relative left turn threshold used by C12.
    pub camera_left_limit: f64,
    /// Camera-relative right turn threshold used by C12.
    pub camera_right_limit: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct EffectSpawnRequest {
    effect_type: u8,
    variant: Option<char>,
    owner: u8,
    owner_facing: u8,
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VictoryAnimationResult {
    pub draws: Vec<RuntimeDraw>,
    /// Zero-based live arena slots allocated by this frame update.
    pub spawned_effect_slots: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct C01FrameResult {
    pub draws: Vec<RuntimeDraw>,
    pub spawned_effect_slots: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombatState {
    pub tick_index: u64,
    pub game_state: u8,
    pub camera_x: f64,
    pub clock_stopped: bool,
    pub fighters: [FighterState; 2],
    /// Live fixed-capacity effect instances; never an event-only spawn queue.
    pub effects: EffectArena,
    /// Pascal SomPendente, consumed by the outer fight/audio phase.
    pub pending_sound: Option<u16>,
    /// Pascal FlashTela request from the C01 final-special entry.
    pub screen_flash: u8,
    /// Pascal TremorTela request from the C01 final special.
    pub screen_shake: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiTickInputs {
    /// Pascal IA input `Dist`.
    pub distance: i32,
    /// Pascal IA input `DistProj`, retained for later character-specific scripts.
    pub projectile_distance: i32,
    /// Pascal IA input `OppY`, retained for later character-specific scripts.
    pub opponent_y: i32,
    /// Pascal IA input `Ameaca`; the source guard is exactly `Ameaca = 0`.
    pub threat: i32,
    /// Pascal IA input `Fator`.
    pub difficulty_factor: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDrawWidth {
    LongInt,
    Int64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeDraw {
    pub width: RuntimeDrawWidth,
    pub range: i64,
    pub value: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiCommand {
    WalkLeft,
    WalkRight,
    WeakPunch,
    WeakKick,
    StrongPunch,
    StrongKick,
    Crouch,
    /// A character-specific special command requested by a ROT [IA] section.
    ChavesSpecial(ChavesSpecialCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTick {
    pub checkpoint_jsonl: String,
    pub draws: Vec<RuntimeDraw>,
    pub commands: Vec<AiCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError<E> {
    Rng(E),
    UnsupportedBranch(&'static str),
    InvalidProbabilityRange { source_line: u32, range: i64 },
    UnsupportedEffectType(u8),
    InvalidEffectOwner(u8),
    InvalidEffectFacing(u8),
    InvalidEffectVariant(u8),
}

impl<E: fmt::Display> fmt::Display for RuntimeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rng(error) => write!(f, "Pascal RNG replay failed: {error}"),
            Self::UnsupportedBranch(branch) => {
                write!(f, "runtime branch not implemented: {branch}")
            }
            Self::InvalidProbabilityRange { source_line, range } => write!(
                f,
                "invalid Pascal AI probability range at source line {source_line}: {range}"
            ),
            Self::UnsupportedEffectType(effect_type) => {
                write!(f, "effect type {effect_type} is not implemented")
            }
            Self::InvalidEffectOwner(owner) => write!(f, "invalid effect owner {owner}"),
            Self::InvalidEffectFacing(facing) => write!(f, "invalid effect facing {facing}"),
            Self::InvalidEffectVariant(effect_type) => {
                write!(
                    f,
                    "effect type {effect_type} requires a valid source variant"
                )
            }
        }
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for RuntimeError<E> {}

/// Hook for a character-specific child ROT invoked by AcaoJogador.
///
/// The same mutable RNG stream is passed to the child, so any nested Random call
/// advances the parent tick's single cursor rather than starting a new replay.
pub trait CharacterActionScript {
    fn execute<R: PascalRandomSource>(
        &mut self,
        state: &mut CombatState,
        fighter_index: usize,
        command: AiCommand,
        rng: &mut R,
    ) -> Result<(), RuntimeError<R::Error>>;

    /// Execute the character-specific [IA] section after Common.rot [IA].
    /// The child receives the same mutable RNG cursor as the common script.
    fn execute_character_ai<R: PascalRandomSource>(
        &mut self,
        _state: &mut CombatState,
        _fighter_index: usize,
        _input: AiTickInputs,
        _rng: &mut R,
        _commands: &mut Vec<AiCommand>,
    ) -> Result<(), RuntimeError<R::Error>> {
        Ok(())
    }
}

/// The common-action-only runtime uses this until a character ROT is implemented.
#[derive(Debug, Default)]
pub struct NoopCharacterActionScript;

impl CharacterActionScript for NoopCharacterActionScript {
    fn execute<R: PascalRandomSource>(
        &mut self,
        _state: &mut CombatState,
        _fighter_index: usize,
        _command: AiCommand,
        _rng: &mut R,
    ) -> Result<(), RuntimeError<R::Error>> {
        Ok(())
    }
}

/// Records every RNG request made by the parent AI and any nested child script.
struct RecordingRandomSource<'a, R> {
    inner: &'a mut R,
    draws: Vec<RuntimeDraw>,
}

impl<'a, R> RecordingRandomSource<'a, R> {
    fn new(inner: &'a mut R) -> Self {
        Self {
            inner,
            draws: Vec::new(),
        }
    }

    fn into_draws(self) -> Vec<RuntimeDraw> {
        self.draws
    }
}

impl<R: PascalRandomSource> PascalRandomSource for RecordingRandomSource<'_, R> {
    type Error = R::Error;

    fn draw_longint(&mut self, range: i32) -> Result<i32, Self::Error> {
        let value = self.inner.draw_longint(range)?;
        self.draws.push(RuntimeDraw {
            width: RuntimeDrawWidth::LongInt,
            range: range as i64,
            value: value as i64,
        });
        Ok(value)
    }

    fn draw_int64(&mut self, range: i64) -> Result<i64, Self::Error> {
        let value = self.inner.draw_int64(range)?;
        self.draws.push(RuntimeDraw {
            width: RuntimeDrawWidth::Int64,
            range,
            value,
        });
        Ok(value)
    }
}

fn pascal_round(value: f64) -> i32 {
    // Delphi/Pascal Round uses ties-to-even; preserve that at exact half values.
    let floor = value.floor();
    let fraction = value - floor;
    if fraction < 0.5 {
        floor as i32
    } else if fraction > 0.5 {
        (floor + 1.0) as i32
    } else if (floor as i64).rem_euclid(2) == 0 {
        floor as i32
    } else {
        (floor + 1.0) as i32
    }
}

impl CombatState {
    /// Apply the shared C01-C15 Pascal [Acao] victory-command branch.
    ///
    /// Every original fighter's pinned ROT uses the same eligibility set and writes:
    /// acVitoria, QuadroAcao = 0, DuracaoAcao = 250. The character-specific victory
    /// animation remains in each fighter's [Quadro] script and is deliberately separate.
    pub fn apply_character_victory(&mut self, fighter_index: usize) -> bool {
        let Some(fighter) = self.fighters.get_mut(fighter_index) else {
            return false;
        };

        if !matches!(
            fighter.action_id,
            ACTION_IDLE | ACTION_WALKING | ACTION_CROUCHING
        ) {
            return false;
        }

        fighter.action_id = ACTION_VICTORY;
        fighter.action_frame = 0;
        fighter.action_duration = 250;
        true
    }

    /// Execute one character's Pascal [Quadro] victory branch in source order.
    ///
    /// The caller supplies the same RNG cursor used by the enclosing game tick.
    /// C12's ordered type-2 spawns are allocated into the live effect arena after
    /// the character state borrow ends, without changing their RNG call order.
    pub fn update_victory_animation<R: PascalRandomSource>(
        &mut self,
        fighter_index: usize,
        context: VictoryAnimationContext,
        rng: &mut R,
    ) -> Result<VictoryAnimationResult, RuntimeError<R::Error>>
    where
        R::Error: fmt::Display,
    {
        let mut rng = RecordingRandomSource::new(rng);
        let mut effect_requests = Vec::new();
        {
            let fighter =
                self.fighters
                    .get_mut(fighter_index)
                    .ok_or(RuntimeError::UnsupportedBranch(
                        "victory animation: invalid fighter index",
                    ))?;
            if fighter.action_id != ACTION_VICTORY {
                return Err(RuntimeError::UnsupportedBranch(
                    "victory animation requires ACTION_VICTORY",
                ));
            }

            let frame = fighter.action_frame;
            let offset = context.sprite_offset;
            match fighter.character_id {
                // C01: independent frame-boundary writes, including the frame-28 reset.
                1 => {
                    if frame == 0 {
                        fighter.sprite_frame = offset + 33;
                    }
                    if frame == 7 {
                        fighter.sprite_frame = offset + 34;
                    }
                    if frame == 14 {
                        fighter.sprite_frame = offset + 35;
                    }
                    if frame == 21 {
                        fighter.sprite_frame = offset + 34;
                    }
                    if frame == 28 {
                        fighter.sprite_frame = offset + 33;
                        fighter.action_frame = 0;
                    }
                }
                // C02: the unconditional base write precedes Random(2).
                2 => {
                    fighter.sprite_frame = offset + 33;
                    if rng.draw_longint(2).map_err(RuntimeError::Rng)? == 0 {
                        fighter.sprite_frame = offset + 34;
                    }
                }
                3 => {
                    if frame == 0 {
                        fighter.sprite_frame = offset + 33;
                    }
                    if frame == 5 {
                        fighter.sprite_frame = offset + 34;
                    }
                    if frame == 10 {
                        fighter.sprite_frame = offset + 35;
                    }
                    if frame == 15 {
                        fighter.sprite_frame = offset + 34;
                    }
                    if frame == 20 {
                        fighter.sprite_frame = offset + 33;
                    }
                    if frame == 25 {
                        fighter.sprite_frame = offset + 34;
                    }
                    if frame == 30 {
                        fighter.sprite_frame = offset + 35;
                    }
                    if frame == 35 {
                        fighter.sprite_frame = offset + 34;
                    }
                }
                4 => {
                    if frame < 13 {
                        fighter.sprite_frame = offset + 33;
                    }
                    if frame == 13 {
                        fighter.sprite_frame = offset + 34;
                    }
                }
                5 => {
                    if frame < 5 {
                        fighter.sprite_frame = offset + 33;
                    }
                    if frame == 9 || frame == 17 || frame == 25 || frame == 33 {
                        fighter.sprite_frame = offset + 34;
                    }
                    if frame == 13 || frame == 21 || frame == 29 || frame == 37 {
                        fighter.sprite_frame = offset + 33;
                    }
                    if frame == 41 {
                        fighter.sprite_frame = offset + 35;
                    }
                }
                6 => {
                    if frame % 4 == 0 {
                        fighter.sprite_frame = if fighter.sprite_frame == offset + 33 {
                            offset + 34
                        } else {
                            offset + 33
                        };
                    }
                }
                7 => {
                    fighter.sprite_frame = offset + if frame % 2 == 0 { 33 } else { 34 };
                }
                8 => {
                    fighter.sprite_frame = offset + 33;
                    if frame % 2 == 0 {
                        fighter.y -= 4.0;
                    }
                    if frame % 2 != 0 {
                        fighter.y += 4.0;
                    }
                    if context.ground_y < fighter.y {
                        fighter.y = context.ground_y;
                    }
                }
                9 => {
                    if frame % 3 == 0 {
                        fighter.sprite_frame = if fighter.sprite_frame == offset + 33 {
                            offset + 34
                        } else {
                            offset + 33
                        };
                    }
                }
                10 => {
                    if frame < 12 {
                        fighter.sprite_frame = offset + 33;
                    }
                    if frame == 12 {
                        fighter.sprite_frame = offset + 34;
                    }
                }
                11 => {
                    fighter.sprite_frame = offset + 33;
                }
                12 => {
                    if frame % 2 == 0 {
                        fighter.sprite_frame = offset + 33;
                    } else {
                        fighter.sprite_frame = offset + 34;
                    }

                    if fighter.facing == SIDE_RIGHT {
                        fighter.x += 12.0;
                    }
                    if fighter.facing == SIDE_LEFT {
                        fighter.x -= 12.0;
                    }

                    let camera_relative_x = fighter.x - context.camera_x;
                    if camera_relative_x <= context.camera_left_limit {
                        fighter.facing = SIDE_RIGHT;
                    }
                    if camera_relative_x >= context.camera_right_limit {
                        fighter.facing = SIDE_LEFT;
                    }

                    if frame % 8 == 0 && rng.draw_longint(2).map_err(RuntimeError::Rng)? == 0 {
                        if fighter.facing == SIDE_RIGHT {
                            fighter.facing = SIDE_LEFT;
                        } else if fighter.facing == SIDE_LEFT {
                            fighter.facing = SIDE_RIGHT;
                        }
                    }

                    // Pascal evaluates the two Random expressions for each effect in order.
                    for _ in 0..5 {
                        let dx = rng.draw_longint(40).map_err(RuntimeError::Rng)?;
                        let dy = rng.draw_longint(10).map_err(RuntimeError::Rng)?;
                        effect_requests.push(EffectSpawnRequest {
                            effect_type: 2,
                            variant: None,
                            owner: 0,
                            owner_facing: EFFECT_SIDE_RIGHT,
                            x: fighter.x - 30.0 + dx as f64,
                            y: fighter.y + dy as f64,
                        });
                    }
                }
                13 => {
                    fighter.sprite_frame = offset + 33;
                    let displacement = pascal_round((f64::from(frame) / 3.0).cos() * 6.0);
                    fighter.y -= f64::from(displacement);
                    if context.ground_y - 10.0 < fighter.y {
                        fighter.y = context.ground_y - 10.0;
                    }
                }
                14 => {
                    if frame % 3 == 0 {
                        fighter.sprite_frame = if fighter.sprite_frame == offset + 33 {
                            offset + 34
                        } else {
                            offset + 33
                        };
                    }
                }
                15 => {
                    if frame <= 10 {
                        fighter.sprite_frame = offset + 33;
                    }
                    if frame == 10 {
                        fighter.sprite_frame = offset + 34;
                    }
                    if frame == 14 {
                        fighter.sprite_frame = offset + 35;
                    }
                }
                _ => {
                    return Err(RuntimeError::UnsupportedBranch(
                        "victory animation: character ID outside audited C01-C15 roster",
                    ))
                }
            }
        }

        let mut spawned_effect_slots = Vec::new();
        for request in effect_requests {
            if let Some(slot) = self.spawn_effect(
                request.effect_type,
                request.variant,
                request.owner,
                request.owner_facing,
                request.x,
                request.y,
                &mut rng,
            )? {
                spawned_effect_slots.push(slot);
            }
        }

        Ok(VictoryAnimationResult {
            draws: rng.into_draws(),
            spawned_effect_slots,
        })
    }

    /// Allocate one real effect instance through the fixed Pascal-compatible arena.
    fn spawn_effect<R: PascalRandomSource>(
        &mut self,
        effect_type: u8,
        variant: Option<char>,
        owner: u8,
        owner_facing: u8,
        x: f64,
        y: f64,
        rng: &mut R,
    ) -> Result<Option<usize>, RuntimeError<R::Error>>
    where
        R::Error: fmt::Display,
    {
        self.effects
            .spawn(effect_type, variant, owner, owner_facing, x, y, rng)
            .map_err(|error| match error {
                EffectSpawnError::Rng(error) => RuntimeError::Rng(error),
                EffectSpawnError::UnsupportedType(effect_type) => {
                    RuntimeError::UnsupportedEffectType(effect_type)
                }
                EffectSpawnError::InvalidOwner(owner) => RuntimeError::InvalidEffectOwner(owner),
                EffectSpawnError::InvalidFacing(facing) => {
                    RuntimeError::InvalidEffectFacing(facing)
                }
                EffectSpawnError::InvalidVariant { effect_type } => {
                    RuntimeError::InvalidEffectVariant(effect_type)
                }
            })
    }

    /// Run the C01 [Quadro] branch for the current special/victory/pose action.
    ///
    /// This is deliberately separate from command entry. The frame path owns
    /// sprite writes, live effects, sounds, movement, invulnerability and the
    /// on-hit [Acerto] recoil hook.
    pub fn update_c01_character_frame<R: PascalRandomSource>(
        &mut self,
        fighter_index: usize,
        context: VictoryAnimationContext,
        rng: &mut R,
    ) -> Result<C01FrameResult, RuntimeError<R::Error>>
    where
        R::Error: fmt::Display,
    {
        let (character_id, action_id, frame, force, facing) = {
            let fighter =
                self.fighters
                    .get(fighter_index)
                    .ok_or(RuntimeError::UnsupportedBranch(
                        "C01 [Quadro]: invalid fighter index",
                    ))?;
            (
                fighter.character_id,
                fighter.action_id,
                fighter.action_frame,
                fighter.force_strength,
                fighter.facing,
            )
        };
        if character_id != 1 {
            return Err(RuntimeError::UnsupportedBranch(
                "C01 [Quadro]: fighter is not C01",
            ));
        }

        match action_id {
            ACTION_VICTORY => {
                let result = self.update_victory_animation(fighter_index, context, rng)?;
                return Ok(C01FrameResult {
                    draws: result.draws,
                    spawned_effect_slots: result.spawned_effect_slots,
                });
            }
            ACTION_POSE => {
                self.update_pose_animation(fighter_index, context.sprite_offset);
                return Ok(C01FrameResult {
                    draws: Vec::new(),
                    spawned_effect_slots: Vec::new(),
                });
            }
            _ => {}
        }

        let player = u8::try_from(fighter_index + 1)
            .map_err(|_| RuntimeError::UnsupportedBranch("C01 [Quadro]: invalid player index"))?;
        let offset = context.sprite_offset;
        let mut rng = RecordingRandomSource::new(rng);
        let mut spawned_effect_slots = Vec::new();

        match action_id {
            ACTION_SPECIAL1 => {
                if frame == 0 {
                    self.fighters[fighter_index].sprite_frame = offset + 36;
                }
                if frame == 4 {
                    self.fighters[fighter_index].sprite_frame = offset + 37;
                }
                if frame == 7 {
                    self.fighters[fighter_index].sprite_frame = offset + 38;
                }
                if frame == 10 {
                    self.fighters[fighter_index].sprite_frame = offset + 39;
                    let variant = match force {
                        FORCE_WEAK => Some('A'),
                        FORCE_STRONG => Some('B'),
                        _ => None,
                    };
                    let x = match facing {
                        SIDE_RIGHT => Some(self.fighters[fighter_index].x + 45.0),
                        SIDE_LEFT => Some(self.fighters[fighter_index].x - 45.0),
                        _ => None,
                    };
                    if let (Some(variant), Some(x)) = (variant, x) {
                        if let Some(slot) = self.spawn_effect(
                            7,
                            Some(variant),
                            player,
                            facing,
                            x,
                            self.fighters[fighter_index].y,
                            &mut rng,
                        )? {
                            spawned_effect_slots.push(slot);
                        }
                    }
                    self.pending_sound = Some(14);
                }
            }
            ACTION_SPECIAL2 => {
                if frame == 0 {
                    self.pending_sound = Some(15);
                }
                if frame == 0 {
                    self.fighters[fighter_index].sprite_frame = offset + 40;
                }
                if frame == 6 && force == FORCE_WEAK {
                    self.fighters[fighter_index].sprite_frame = offset + 41;
                }
                if frame == 10 && force == FORCE_STRONG {
                    self.fighters[fighter_index].sprite_frame = offset + 41;
                }
                if frame == 12 && force == FORCE_WEAK {
                    self.fighters[fighter_index].sprite_frame = offset + 42;
                }
                if frame == 19 && force == FORCE_STRONG {
                    self.fighters[fighter_index].sprite_frame = offset + 42;
                }

                // Pascal checks the selected sprite frame, not just the action timer.
                if self.fighters[fighter_index].sprite_frame == offset + 41 {
                    for _ in 0..3 {
                        let dx = rng.draw_longint(40).map_err(RuntimeError::Rng)?;
                        let dy = rng.draw_longint(10).map_err(RuntimeError::Rng)?;
                        let x = self.fighters[fighter_index].x - 20.0 + f64::from(dx);
                        let y = self.fighters[fighter_index].y + f64::from(dy);
                        if let Some(slot) =
                            self.spawn_effect(2, None, 0, EFFECT_SIDE_RIGHT, x, y, &mut rng)?
                        {
                            spawned_effect_slots.push(slot);
                        }
                    }

                    let push = match (facing, force) {
                        (SIDE_RIGHT, FORCE_WEAK) => Some(19.0),
                        (SIDE_LEFT, FORCE_WEAK) => Some(-19.0),
                        (SIDE_RIGHT, FORCE_STRONG) => Some(21.0),
                        (SIDE_LEFT, FORCE_STRONG) => Some(-21.0),
                        _ => None,
                    };
                    if let Some(push) = push {
                        self.push_fighter(fighter_index, push);
                    }
                }

                if frame >= 9 {
                    let side = self.fighters[fighter_index].facing;
                    let origin_x = self.fighters[fighter_index].x;
                    let origin_y = self.fighters[fighter_index].y;
                    let x = match side {
                        SIDE_RIGHT => Some(origin_x + 60.0),
                        SIDE_LEFT => Some(origin_x - 60.0),
                        _ => None,
                    };
                    if let Some(x) = x {
                        if let Some(slot) =
                            self.spawn_effect(16, None, player, side, x, origin_y - 85.0, &mut rng)?
                        {
                            spawned_effect_slots.push(slot);
                        }
                    }
                }
                self.fighters[fighter_index].is_kick = true;
            }
            ACTION_SPECIAL3 => {
                let opponent_index = 1 - fighter_index;
                if self.fighters[fighter_index].hit_count >= 1 {
                    self.fighters[opponent_index].invulnerable = false;
                }
                if self.fighters[fighter_index].hit_count >= 4 {
                    self.fighters[opponent_index].invulnerable = true;
                }
                if frame > 7 {
                    self.fighters[opponent_index].invulnerable = true;
                }
                if frame == 0 {
                    self.pending_sound = Some(16);
                }
                if frame % 3 == 0 {
                    let previous = self.fighters[fighter_index].sprite_frame;
                    self.fighters[fighter_index].sprite_frame = if previous == offset + 43 {
                        offset + 44
                    } else {
                        offset + 43
                    };
                }
                self.fighters[fighter_index].y -= 10.0;
                match facing {
                    SIDE_RIGHT => self.push_fighter(fighter_index, 5.0),
                    SIDE_LEFT => self.push_fighter(fighter_index, -5.0),
                    _ => {}
                }
                if frame == 20 {
                    self.fighters[fighter_index].in_air = true;
                }
                self.fighters[fighter_index].is_kick = true;
            }
            ACTION_SPECIAL4 => {
                if frame == 0 {
                    self.pending_sound = Some(17);
                    self.fighters[fighter_index].sprite_frame = offset + 45;
                }
                if frame == 4 {
                    self.fighters[fighter_index].sprite_frame = offset + 46;
                }
                if frame == 8 {
                    self.fighters[fighter_index].sprite_frame = offset + 47;
                }
                if frame == 12 {
                    self.fighters[fighter_index].sprite_frame = offset + 48;
                }
                if frame == 15 {
                    self.fighters[fighter_index].sprite_frame = offset + 49;
                }
                if frame == 18 {
                    self.fighters[fighter_index].sprite_frame = offset + 12;
                }
                if (4..=13).contains(&frame) {
                    let push = match (facing, force) {
                        (SIDE_RIGHT, FORCE_WEAK) => Some(17.0),
                        (SIDE_LEFT, FORCE_WEAK) => Some(-17.0),
                        (SIDE_RIGHT, FORCE_STRONG) => Some(21.0),
                        (SIDE_LEFT, FORCE_STRONG) => Some(-21.0),
                        _ => None,
                    };
                    if let Some(push) = push {
                        self.push_fighter(fighter_index, push);
                    }
                }
                if frame >= 12 {
                    self.fighters[fighter_index].is_kick = true;
                }
            }
            ACTION_SPECIAL_FINAL => {
                if frame == 0 {
                    self.fighters[fighter_index].sprite_frame = offset + 36;
                }
                if frame == 3 {
                    self.fighters[fighter_index].sprite_frame = offset + 37;
                }
                if frame == 6 {
                    self.fighters[fighter_index].sprite_frame = offset + 38;
                }
                if frame == 9 {
                    self.pending_sound = Some(18);
                    self.fighters[fighter_index].sprite_frame = offset + 39;
                    let origin_x = self.fighters[fighter_index].x;
                    let origin_y = self.fighters[fighter_index].y;
                    let x = match facing {
                        SIDE_RIGHT => Some(origin_x + 110.0),
                        SIDE_LEFT => Some(origin_x - 110.0),
                        _ => None,
                    };
                    if let Some(x) = x {
                        if let Some(slot) =
                            self.spawn_effect(8, None, player, facing, x, origin_y, &mut rng)?
                        {
                            spawned_effect_slots.push(slot);
                        }
                    }
                    self.screen_shake = 20;
                }
            }
            _ => {}
        }

        Ok(C01FrameResult {
            draws: rng.into_draws(),
            spawned_effect_slots,
        })
    }

    /// C01 [Acerto]: strong Esp2 gives the opponent the character-specific recoil.
    pub fn apply_c01_hit_reaction(&mut self, attacker_index: usize) -> bool {
        let Some(attacker) = self.fighters.get(attacker_index) else {
            return false;
        };
        if attacker.character_id != 1
            || attacker.action_id != ACTION_SPECIAL2
            || attacker.force_strength != FORCE_STRONG
            || attacker_index > 1
        {
            return false;
        }
        self.fighters[1 - attacker_index].recoil = 20;
        true
    }

    /// Advance effects in slot order and return immutable render snapshots.
    pub fn tick_effects(&mut self, context: EffectTickContext) -> EffectTickResult {
        self.effects.advance(context)
    }

    /// Execute the source-backed C01-C12 pose [Quadro] frame writes.
    ///
    /// All twelve pinned ROT files write Desloc + 12 at action frame 0 and
    /// Desloc + 30 at action frame 2. Other frames retain the previous sprite.
    /// C13-C15 do not define the pose command and are rejected.
    pub fn update_pose_animation(&mut self, fighter_index: usize, sprite_offset: i32) -> bool {
        let Some(fighter) = self.fighters.get_mut(fighter_index) else {
            return false;
        };

        if !(1..=12).contains(&fighter.character_id) || fighter.action_id != ACTION_POSE {
            return false;
        }

        if fighter.action_frame == 0 {
            fighter.sprite_frame = sprite_offset + 12;
        }
        if fighter.action_frame == 2 {
            fighter.sprite_frame = sprite_offset + 30;
        }
        true
    }

    /// Apply Pascal `cmPose` for C01-C12.
    ///
    /// The pinned C01-C12 [Acao] scripts all accept only idle/walking and write
    /// acPose, frame 0, duration 20. C13-C15 do not define cmPose and are rejected.
    pub fn apply_character_pose(&mut self, fighter_index: usize) -> bool {
        let Some(fighter) = self.fighters.get_mut(fighter_index) else {
            return false;
        };

        if !(1..=12).contains(&fighter.character_id)
            || !matches!(fighter.action_id, ACTION_IDLE | ACTION_WALKING)
        {
            return false;
        }

        fighter.action_id = ACTION_POSE;
        fighter.action_frame = 0;
        fighter.action_duration = 20;
        true
    }

    /// Execute Common.rot and C01 [IA] with a shared Pascal RNG cursor.
    ///
    /// `context` carries the globals read by C01 [Acao]/[IA]. The Special 1 guard
    /// queries the actual live `EffectArena`; no parallel effect list is maintained.
    pub fn tick_common_ai_with_chaves<R: PascalRandomSource>(
        &mut self,
        input: AiTickInputs,
        context: ChavesSpecialContext,
        rng: &mut R,
    ) -> Result<RuntimeTick, RuntimeError<R::Error>>
    where
        R::Error: fmt::Display,
    {
        let mut script = ChavesCharacterActionScript::new(context);
        self.tick_common_ai_with_child(input, rng, &mut script)
    }

    /// Execute one source-ordered Common.rot [IA] tick using no character-specific child ROT.
    pub fn tick_common_ai<R: PascalRandomSource>(
        &mut self,
        input: AiTickInputs,
        rng: &mut R,
    ) -> Result<RuntimeTick, RuntimeError<R::Error>>
    where
        R::Error: fmt::Display,
    {
        let mut child_script = NoopCharacterActionScript;
        self.tick_common_ai_with_child(input, rng, &mut child_script)
    }

    /// Execute the same AI tick with an injected character-specific child-script executor.
    ///
    /// This is the integration point for Cnn.rot [Acao]. The child receives the same
    /// RNG cursor as Common.rot [IA], and its draws are included in RuntimeTick.
    pub fn tick_common_ai_with_child<R, S>(
        &mut self,
        input: AiTickInputs,
        rng: &mut R,
        child_script: &mut S,
    ) -> Result<RuntimeTick, RuntimeError<R::Error>>
    where
        R: PascalRandomSource,
        R::Error: fmt::Display,
        S: CharacterActionScript,
    {
        let start = self.checkpoint(true, true);
        let mut rng = RecordingRandomSource::new(rng);
        let mut commands = Vec::new();

        // Common.rot [IA], line 293: if Random(13) = 0 then case Random(6) of ...
        let random_gate = rng.draw_longint(13).map_err(RuntimeError::Rng)?;
        if random_gate == 0 {
            let target_index = rng.draw_longint(6).map_err(RuntimeError::Rng)?;
            self.fighters[1].distance_ai = match target_index {
                0 => 100,
                1 => 150,
                2 => 200,
                3 => 250,
                4 => 300,
                5 => 350,
                _ => {
                    return Err(RuntimeError::UnsupportedBranch(
                        "Common.rot [IA] line 294: Random(6) returned outside 0..6",
                    ))
                }
            };
        }

        // This assignment happens after the optional random target selection.
        let fighter = &mut self.fighters[1];
        if (input.distance as i64 - fighter.distance_ai as i64).abs() <= 50 {
            fighter.distance_ai = input.distance;
        }

        // Source lines 302-308: four mutually exclusive direction/facing predicates.
        // Cache the entry guard: each AcaoJogador call clears P.Defendendo internally.
        let ai_actions_enabled = !self.fighters[1].defending && input.threat == 0;
        if ai_actions_enabled {
            let fighter = &self.fighters[1];
            let command = match (input.distance >= fighter.distance_ai, fighter.facing) {
                (true, SIDE_RIGHT) => Some(AiCommand::WalkRight),
                (false, SIDE_RIGHT) => Some(AiCommand::WalkLeft),
                (true, SIDE_LEFT) => Some(AiCommand::WalkLeft),
                (false, SIDE_LEFT) => Some(AiCommand::WalkRight),
                (_, _) => {
                    return Err(RuntimeError::UnsupportedBranch(
                        "Common.rot [IA]: unknown P.Lado value",
                    ))
                }
            };
            if let Some(command) = command {
                self.execute_action_command(1, command, &mut rng, child_script, &mut commands)?;
            }
        }

        // Common.rot [IA], lines 310-323. Each independent if consumes its own draw;
        // only the four crouching probabilities are conditional on the fifth draw.
        if !self.fighters[1].defending && input.threat == 0 && input.distance <= 80 {
            let factor = input.difficulty_factor;
            if Self::random_zero_int64(&mut rng, 8.0, factor, 312)? {
                self.execute_action_command(
                    1,
                    AiCommand::WeakPunch,
                    &mut rng,
                    child_script,
                    &mut commands,
                )?;
            }
            if Self::random_zero_int64(&mut rng, 8.0, factor, 313)? {
                self.execute_action_command(
                    1,
                    AiCommand::WeakKick,
                    &mut rng,
                    child_script,
                    &mut commands,
                )?;
            }
            if Self::random_zero_int64(&mut rng, 9.0, factor, 314)? {
                self.execute_action_command(
                    1,
                    AiCommand::StrongPunch,
                    &mut rng,
                    child_script,
                    &mut commands,
                )?;
            }
            if Self::random_zero_int64(&mut rng, 9.0, factor, 315)? {
                self.execute_action_command(
                    1,
                    AiCommand::StrongKick,
                    &mut rng,
                    child_script,
                    &mut commands,
                )?;
            }
            if Self::random_zero_int64(&mut rng, 4.0, factor, 316)? {
                self.execute_action_command(
                    1,
                    AiCommand::Crouch,
                    &mut rng,
                    child_script,
                    &mut commands,
                )?;
                if Self::random_zero_int64(&mut rng, 2.0, factor, 319)? {
                    self.execute_action_command(
                        1,
                        AiCommand::WeakPunch,
                        &mut rng,
                        child_script,
                        &mut commands,
                    )?;
                }
                if Self::random_zero_int64(&mut rng, 2.0, factor, 320)? {
                    self.execute_action_command(
                        1,
                        AiCommand::WeakKick,
                        &mut rng,
                        child_script,
                        &mut commands,
                    )?;
                }
                if Self::random_zero_int64(&mut rng, 3.0, factor, 321)? {
                    self.execute_action_command(
                        1,
                        AiCommand::StrongPunch,
                        &mut rng,
                        child_script,
                        &mut commands,
                    )?;
                }
                if Self::random_zero_int64(&mut rng, 3.0, factor, 322)? {
                    self.execute_action_command(
                        1,
                        AiCommand::StrongKick,
                        &mut rng,
                        child_script,
                        &mut commands,
                    )?;
                }
            }
        }

        // Character [IA] runs after Common.rot [IA], using the same recorded RNG stream.
        child_script.execute_character_ai(self, 1, input, &mut rng, &mut commands)?;

        let end = self.checkpoint(false, false);
        Ok(RuntimeTick {
            checkpoint_jsonl: format!("{start}\n{end}\n"),
            draws: rng.into_draws(),
            commands,
        })
    }

    fn random_zero_int64<R: PascalRandomSource>(
        rng: &mut R,
        multiplier: f64,
        factor: f64,
        source_line: u32,
    ) -> Result<bool, RuntimeError<R::Error>> {
        let unrounded = multiplier * factor;
        // Keep the declared Rust 1.70 MSRV: implement Pascal's ties-to-even
        // rounding explicitly instead of relying on newer f64 methods.
        if !unrounded.is_finite()
            || unrounded >= 9_223_372_036_854_775_808.0
            || unrounded < -9_223_372_036_854_775_808.0
        {
            return Err(RuntimeError::InvalidProbabilityRange {
                source_line,
                range: 0,
            });
        }
        let floor = unrounded.floor();
        let fraction = unrounded - floor;
        let rounded = if fraction < 0.5 {
            floor
        } else if fraction > 0.5 {
            floor + 1.0
        } else if (floor as i64).rem_euclid(2) == 0 {
            floor
        } else {
            floor + 1.0
        };
        let range = rounded as i64;
        if range <= 0 {
            return Err(RuntimeError::InvalidProbabilityRange { source_line, range });
        }
        let value = rng.draw_int64(range).map_err(RuntimeError::Rng)?;
        Ok(value == 0)
    }

    /// Execute the common [Acao] body, the character-specific child ROT, then [AcaoFim].
    fn execute_action_command<R, S>(
        &mut self,
        fighter_index: usize,
        command: AiCommand,
        rng: &mut R,
        child_script: &mut S,
        commands: &mut Vec<AiCommand>,
    ) -> Result<(), RuntimeError<R::Error>>
    where
        R: PascalRandomSource,
        S: CharacterActionScript,
    {
        commands.push(command);
        self.execute_common_command(fighter_index, command);

        // AcaoJogador calls the character-specific [Acao] between common [Acao]
        // and common [AcaoFim]. This child receives the parent's mutable RNG stream.
        child_script.execute(self, fighter_index, command, rng)?;

        // Common.rot [AcaoFim]: only an action still walking advances here.
        self.finish_common_action(fighter_index);
        Ok(())
    }

    /// Apply the Common.rot [AcaoFim] walk-frame update after any AcaoJogador call.
    /// Returns the resulting frame only when the fighter remains in the walking action.
    pub(crate) fn finish_common_action(&mut self, fighter_index: usize) -> Option<u16> {
        let fighter = self.fighters.get_mut(fighter_index)?;
        if fighter.action_id != ACTION_WALKING {
            return None;
        }
        fighter.action_frame = fighter.action_frame.saturating_add(1);
        if fighter.action_frame > 16 {
            fighter.action_frame = 0;
        }
        Some(fighter.action_frame)
    }

    fn execute_common_command(&mut self, fighter_index: usize, command: AiCommand) {
        let fighter = &mut self.fighters[fighter_index];
        fighter.defending = false;

        match command {
            AiCommand::WalkLeft | AiCommand::WalkRight => {
                if matches!(fighter.action_id, ACTION_IDLE | ACTION_WALKING) {
                    let direction = if command == AiCommand::WalkLeft {
                        -1.0
                    } else {
                        1.0
                    };
                    let speed = fighter.speed;
                    self.push_fighter(fighter_index, direction * speed);
                    let fighter = &mut self.fighters[fighter_index];
                    fighter.action_id = ACTION_WALKING;
                    fighter.action_duration = 99;
                }
            }
            AiCommand::Crouch => {
                if matches!(fighter.action_id, ACTION_IDLE | ACTION_WALKING) {
                    fighter.action_id = ACTION_CROUCHING;
                    fighter.action_duration = 99;
                }
            }
            AiCommand::WeakPunch => {
                if matches!(
                    fighter.action_id,
                    ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING
                ) && fighter.vertical_velocity == 0
                {
                    Self::set_attack(fighter, ACTION_WEAK_PUNCH, 6);
                }
                if matches!(fighter.action_id, ACTION_CROUCHING | ACTION_LOW_DEFENDING) {
                    Self::set_attack(fighter, ACTION_CROUCH_WEAK_PUNCH, 6);
                }
                if fighter.action_id == ACTION_JUMPING {
                    Self::set_attack(fighter, ACTION_AIR_WEAK_PUNCH, 15);
                }
            }
            AiCommand::WeakKick => {
                if (matches!(
                    fighter.action_id,
                    ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING
                ) || (fighter.action_id == ACTION_WEAK_PUNCH && fighter.action_frame >= 2))
                    && fighter.vertical_velocity == 0
                {
                    Self::set_attack(fighter, ACTION_WEAK_KICK, 6);
                }
                if matches!(fighter.action_id, ACTION_CROUCHING | ACTION_LOW_DEFENDING)
                    || (fighter.action_id == ACTION_CROUCH_WEAK_PUNCH && fighter.action_frame >= 2)
                {
                    Self::set_attack(fighter, ACTION_CROUCH_WEAK_KICK, 6);
                }
                if fighter.action_id == ACTION_JUMPING {
                    Self::set_attack(fighter, ACTION_AIR_WEAK_KICK, 15);
                }
            }
            AiCommand::StrongPunch => {
                if (matches!(
                    fighter.action_id,
                    ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING
                ) || (matches!(fighter.action_id, ACTION_WEAK_PUNCH | ACTION_WEAK_KICK)
                    && fighter.action_frame >= 2))
                    && fighter.vertical_velocity == 0
                {
                    Self::set_attack(fighter, ACTION_STRONG_PUNCH, 11);
                }
                if matches!(fighter.action_id, ACTION_CROUCHING | ACTION_LOW_DEFENDING)
                    || (matches!(
                        fighter.action_id,
                        ACTION_CROUCH_WEAK_PUNCH | ACTION_CROUCH_WEAK_KICK
                    ) && fighter.action_frame >= 2)
                {
                    Self::set_attack(fighter, ACTION_CROUCH_STRONG_PUNCH, 11);
                }
                if fighter.action_id == ACTION_JUMPING {
                    Self::set_attack(fighter, ACTION_AIR_STRONG_PUNCH, 11);
                }
            }
            AiCommand::StrongKick => {
                if matches!(
                    fighter.action_id,
                    ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING
                ) && fighter.vertical_velocity == 0
                    && !fighter.in_air
                {
                    Self::set_attack(fighter, ACTION_STRONG_KICK, 11);
                }
                if matches!(fighter.action_id, ACTION_CROUCHING | ACTION_LOW_DEFENDING)
                    || (fighter.action_id == ACTION_CROUCH_WEAK_KICK && fighter.action_frame >= 2)
                {
                    Self::set_attack(fighter, ACTION_CROUCH_STRONG_KICK, 11);
                    if fighter.character_id == 13 {
                        fighter.action_duration = 18;
                    }
                }
                if fighter.action_id == ACTION_JUMPING {
                    Self::set_attack(fighter, ACTION_AIR_STRONG_KICK, 11);
                }
            }
            // Special requests are dispatched by the character script, not by
            // the common-action state machine.
            AiCommand::ChavesSpecial(_) => {}
        }
    }

    fn set_attack(fighter: &mut FighterState, action_id: u16, duration: u16) {
        fighter.action_id = action_id;
        fighter.action_frame = 0;
        fighter.action_duration = duration;
    }

    /// Pascal ChavesGolpes.Empurra, including body separation and screen clamps.
    pub(crate) fn push_fighter(&mut self, fighter_index: usize, dx: f64) {
        let (fighters_left, fighters_right) = self.fighters.split_at_mut(1);
        let (fighter, opponent) = if fighter_index == 0 {
            (&mut fighters_left[0], &mut fighters_right[0])
        } else {
            (&mut fighters_right[0], &mut fighters_left[0])
        };

        fighter.x += dx;
        let overlap = opponent.body.x2 + opponent.x > fighter.body.x1 + fighter.x
            && opponent.body.x1 + opponent.x < fighter.body.x2 + fighter.x
            && opponent.body.y2 + opponent.y > fighter.body.y1 + fighter.y
            && opponent.body.y1 + opponent.y < fighter.body.y2 + fighter.y
            && fighter.action_id != ACTION_VICTORY
            && fighter.action_id != ACTION_BOUNCING;

        if overlap && !self.clock_stopped {
            if fighter.x <= opponent.x {
                opponent.x = (opponent.x + 5.0).min(self.camera_x + 370.0);
                fighter.x = (opponent.body.x1 + opponent.x - fighter.body.x2) - 5.0;
            }
            if fighter.x > opponent.x {
                opponent.x = (opponent.x - 5.0).max(self.camera_x + 30.0);
                fighter.x = (opponent.body.x2 + opponent.x - fighter.body.x1) + 5.0;
            }
        }

        if fighter.health > 0.0 {
            fighter.x = fighter.x.clamp(self.camera_x + 30.0, self.camera_x + 370.0);
        }
    }

    fn checkpoint(&self, start: bool, include_durations: bool) -> String {
        let p1 = &self.fighters[0];
        let p2 = &self.fighters[1];
        let mut record = json!({
            "record_type": if start { "tick_start" } else { "tick_end" },
            "tick_index": self.tick_index,
            "game_state": self.game_state,
            "p1_character_id": p1.character_id,
            "p1_action_id": p1.action_id,
            "p1_action_frame": p1.action_frame,
            "p2_character_id": p2.character_id,
            "p2_action_id": p2.action_id,
            "p2_action_frame": p2.action_frame,
        });
        if start {
            record["executed"] = json!(true);
        }
        if include_durations {
            record["p1_action_duration"] = json!(p1.action_duration);
            record["p2_action_duration"] = json!(p2.action_duration);
        }
        record.to_string()
    }
}
