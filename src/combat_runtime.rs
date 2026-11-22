//! First executable Pascal-derived combat runtime slice.
//!
//! This implements the source-ordered Common.rot [IA] path for target selection,
//! movement, and the close-range probability chain. Character-specific ROT execution
//! remains an injected child-script seam; unsupported behavior is not fabricated.

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
    pub character_id: u8,
    pub action_id: u16,
    pub action_frame: u16,
    pub action_duration: u16,
    pub facing: u8,
    pub defending: bool,
    pub vertical_velocity: i8,
    pub in_air: bool,
    pub x: f64,
    pub y: f64,
    pub speed: f64,
    pub distance_ai: i32,
    pub health: i32,
    pub body: BodyBox,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombatState {
    pub tick_index: u64,
    pub game_state: u8,
    pub camera_x: f64,
    pub clock_stopped: bool,
    pub fighters: [FighterState; 2],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiTickInputs {
    pub distance: i32,
    pub projectile_threat: bool,
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
                "invalid Pascal probability range at Common.rot line {source_line}: {range}"
            ),
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

impl CombatState {
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
        let ai_actions_enabled = !self.fighters[1].defending && !input.projectile_threat;
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
        if !self.fighters[1].defending && !input.projectile_threat && input.distance <= 80 {
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
        let fighter = &mut self.fighters[fighter_index];
        if fighter.action_id == ACTION_WALKING {
            fighter.action_frame = fighter.action_frame.saturating_add(1);
            if fighter.action_frame > 16 {
                fighter.action_frame = 0;
            }
        }
        Ok(())
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
        }
    }

    fn set_attack(fighter: &mut FighterState, action_id: u16, duration: u16) {
        fighter.action_id = action_id;
        fighter.action_frame = 0;
        fighter.action_duration = duration;
    }

    /// Pascal ChavesGolpes.Empurra, including body separation and screen clamps.
    fn push_fighter(&mut self, fighter_index: usize, dx: f64) {
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

        if fighter.health > 0 {
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
