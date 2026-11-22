//! First executable Pascal-derived combat vertical slice.
//!
//! Scope is intentionally narrow: one Common.rot [IA] tick where Random(13) != 0,
//! followed by the common left/right walking command. Other IA branches fail closed
//! until their source semantics are implemented.

use crate::pascal_replay::PascalRandomSource;
use serde_json::json;
use std::fmt;

pub const ACTION_IDLE: u16 = 1;
pub const ACTION_WALKING: u16 = 2;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiTickInputs {
    pub distance: i32,
    pub projectile_threat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeDraw {
    pub range: i32,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTick {
    pub checkpoint_jsonl: String,
    pub draws: Vec<RuntimeDraw>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError<E> {
    Rng(E),
    UnsupportedBranch(&'static str),
}

impl<E: fmt::Display> fmt::Display for RuntimeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rng(error) => write!(f, "Pascal RNG replay failed: {error}"),
            Self::UnsupportedBranch(branch) => write!(f, "runtime branch not implemented: {branch}"),
        }
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for RuntimeError<E> {}

impl CombatState {
    /// Execute one source-ordered Common.rot [IA] tick for fighter 2.
    ///
    /// This is not a general .rot interpreter. It implements the observed
    /// Random(13) gate and the subsequent left/right walk selection only.
    pub fn tick_common_ai_walk<R: PascalRandomSource>(
        &mut self,
        input: AiTickInputs,
        rng: &mut R,
    ) -> Result<RuntimeTick, RuntimeError<R::Error>>
    where
        R::Error: fmt::Display,
    {
        let start = self.checkpoint(true, true);
        let mut draws = Vec::new();

        // Common.rot [IA], source line 293: if Random(13) = 0 then
        let random_value = rng.draw_longint(13).map_err(RuntimeError::Rng)?;
        draws.push(RuntimeDraw { range: 13, value: random_value });
        if random_value == 0 {
            return Err(RuntimeError::UnsupportedBranch(
                "Common.rot [IA] line 294: Random(6) distance-target selection",
            ));
        }

        let fighter = &mut self.fighters[1];

        // Common.rot [IA]: if Abs(Dist - P.DistanciaIA) <= 50 then P.DistanciaIA := Dist
        if (input.distance - fighter.distance_ai).abs() <= 50 {
            fighter.distance_ai = input.distance;
        }

        // The following attack-selection block is intentionally excluded.
        if !fighter.defending && !input.projectile_threat && input.distance <= 80 {
            return Err(RuntimeError::UnsupportedBranch(
                "Common.rot [IA]: close-range attack probability chain",
            ));
        }

        // Common.rot [IA], four mutually exclusive direction/facing predicates.
        if !fighter.defending && !input.projectile_threat {
            let command_left = (input.distance < fighter.distance_ai && fighter.facing == SIDE_RIGHT)
                || (input.distance >= fighter.distance_ai && fighter.facing == SIDE_LEFT);
            let command_right = (input.distance >= fighter.distance_ai && fighter.facing == SIDE_RIGHT)
                || (input.distance < fighter.distance_ai && fighter.facing == SIDE_LEFT);

            if command_left {
                self.execute_common_walk(1, -1.0)?;
            } else if command_right {
                self.execute_common_walk(1, 1.0)?;
            }
        }

        let end = self.checkpoint(false, false);
        Ok(RuntimeTick {
            checkpoint_jsonl: format!("{start}\n{end}\n"),
            draws,
        })
    }

    /// Common.rot [Acao] movement branch, followed by [AcaoFim].
    fn execute_common_walk<E>(
        &mut self,
        fighter_index: usize,
        direction: f64,
    ) -> Result<(), RuntimeError<E>> {
        let fighter = &mut self.fighters[fighter_index];

        // Common.rot [Acao] begins with P.Defendendo := False.
        fighter.defending = false;
        if !matches!(fighter.action_id, ACTION_IDLE | ACTION_WALKING) {
            return Ok(());
        }

        // Empurra(Jog, direction * P.Velocidade), preserving the source ordering.
        self.push_fighter(fighter_index, direction * fighter.speed);
        let fighter = &mut self.fighters[fighter_index];
        fighter.action_id = ACTION_WALKING;
        fighter.action_duration = 99;
        // Facing is unchanged by the common walking command.
        fighter.defending = false;

        // Common.rot [AcaoFim]: increment walk animation phase and wrap after 16.
        fighter.action_frame = fighter.action_frame.saturating_add(1);
        if fighter.action_frame > 16 {
            fighter.action_frame = 0;
        }

        // L7_Fisica does not increment a duration-99 action frame.
        Ok(())
    }

    /// Pascal ChavesGolpes.Empurra, including body separation and screen clamps.
    fn push_fighter(&mut self, fighter_index: usize, dx: f64) {
        let opponent_index = 1 - fighter_index;
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
