//! Pascal-style input command recognition primitives.
use crate::combat_runtime::{FighterState, SIDE_LEFT, SIDE_RIGHT};
use crate::input::{Button, Direction, InputFrame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackButton {
    WeakPunch = 0,
    WeakKick = 1,
    StrongPunch = 2,
    StrongKick = 3,
}
impl AttackButton {
    pub const fn input_button(self) -> Button {
        Button(self as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionAtom {
    Down,
    Forward,
    Back,
    Up,
    DownHeld(u16),
    BackHeld(u16),
    Grounded,
    SequenceState(u8),
    Airborne,
    PunchCharge(u16),
    KickCharge(u16),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxCondition {
    pub condition: ConditionAtom,
    pub negated: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRule {
    pub attack_button: AttackButton,
    pub command_code: u8,
    pub conditions: Vec<BoxCondition>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceRule {
    pub from_state: u8,
    pub to_state: u8,
    pub conditions: Vec<BoxCondition>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleParseError {
    pub token: String,
}

/// Parse documented condition letters: B/F/T/C/H/A and numeric b/t/P/s/k; ! negates.
pub fn parse_condition(token: &str) -> Result<BoxCondition, RuleParseError> {
    let token = token.trim();
    let (negated, atom) = token
        .strip_prefix('!')
        .map_or((false, token), |s| (true, s));
    let mut chars = atom.chars();
    let kind = chars.next().ok_or_else(|| RuleParseError {
        token: token.into(),
    })?;
    let value: String = chars.collect();
    let n = value.parse::<u16>().ok();
    let condition = match kind {
        'B' if value.is_empty() => ConditionAtom::Down,
        'F' if value.is_empty() => ConditionAtom::Forward,
        'T' if value.is_empty() => ConditionAtom::Back,
        'C' if value.is_empty() => ConditionAtom::Up,
        'H' if value.is_empty() => ConditionAtom::Grounded,
        'A' if value.is_empty() => ConditionAtom::Airborne,
        'b' if n.is_some() => ConditionAtom::DownHeld(n.unwrap()),
        't' if n.is_some() => ConditionAtom::BackHeld(n.unwrap()),
        'P' if n.is_some() => ConditionAtom::SequenceState(n.unwrap() as u8),
        's' if n.is_some() => ConditionAtom::PunchCharge(n.unwrap()),
        'k' if n.is_some() => ConditionAtom::KickCharge(n.unwrap()),
        _ => {
            return Err(RuleParseError {
                token: token.into(),
            })
        }
    };
    Ok(BoxCondition { condition, negated })
}

fn relative(direction: Direction, facing: u8) -> (bool, bool, bool, bool) {
    let (l, r, u, d) = match direction {
        Direction::Neutral => (false, false, false, false),
        Direction::Up => (false, false, true, false),
        Direction::UpRight => (false, true, true, false),
        Direction::Right => (false, true, false, false),
        Direction::DownRight => (false, true, false, true),
        Direction::Down => (false, false, false, true),
        Direction::DownLeft => (true, false, false, true),
        Direction::Left => (true, false, false, false),
        Direction::UpLeft => (true, false, true, false),
    };
    let (f, b) = if facing == SIDE_RIGHT {
        (r, l)
    } else if facing == SIDE_LEFT {
        (l, r)
    } else {
        (false, false)
    };
    (d, f, b, u)
}

/// Evaluate ordered AND conditions against sampled input and current fighter state.
pub fn rule_matches(
    conditions: &[BoxCondition],
    fighter: &FighterState,
    input: InputFrame,
    down_ticks: u16,
    back_ticks: u16,
    ground_y: f64,
) -> bool {
    let (down, forward, back, up) = relative(input.direction, fighter.facing);
    conditions.iter().all(|c| {
        let v = match c.condition {
            ConditionAtom::Down => down,
            ConditionAtom::Forward => forward,
            ConditionAtom::Back => back,
            ConditionAtom::Up => up,
            ConditionAtom::DownHeld(n) => down_ticks >= n,
            ConditionAtom::BackHeld(n) => back_ticks >= n,
            // Pascal RegraVale('H') compares ChaoY = P.Y directly.
            ConditionAtom::Grounded => fighter.y == ground_y,
            ConditionAtom::SequenceState(n) => fighter.sequence_state == n,
            ConditionAtom::Airborne => fighter.vertical_velocity != 0,
            ConditionAtom::PunchCharge(n) => {
                fighter.punch_charge >= n.min(u16::from(u8::MAX)) as u8
            }
            ConditionAtom::KickCharge(n) => fighter.kick_charge >= n.min(u16::from(u8::MAX)) as u8,
        };
        v != c.negated
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommandRecognizer {
    down_ticks: u16,
    back_ticks: u16,
}
impl CommandRecognizer {
    /// Update Pascal TempoBaixo/TempoTras counters once per logical tick.
    pub fn update_counters(&mut self, fighter: &mut FighterState, input: InputFrame) {
        let (down, _, back, _) = relative(input.direction, fighter.facing);
        if down {
            self.down_ticks = self.down_ticks.saturating_add(1).min(18);
        } else if self.down_ticks > 1 {
            self.down_ticks -= 1;
        }
        if back {
            self.back_ticks = self.back_ticks.saturating_add(1).min(18);
        } else if self.back_ticks > 1 {
            self.back_ticks -= 1;
        }
        fighter.low_time = self.down_ticks as u8;
        fighter.back_time = self.back_ticks as u8;
    }

    pub fn counters(&self) -> (u16, u16) {
        (self.down_ticks, self.back_ticks)
    }

    /// Return all matching rules in source order, on a fresh button press only.
    pub fn commands_for_button(
        &self,
        fighter: &FighterState,
        input: InputFrame,
        button: AttackButton,
        rules: &[CommandRule],
        ground_y: f64,
    ) -> Vec<u8> {
        if !input.was_pressed(button.input_button()) {
            return vec![];
        }
        rules
            .iter()
            .filter(|r| {
                r.attack_button == button
                    && rule_matches(
                        &r.conditions,
                        fighter,
                        input,
                        self.down_ticks,
                        self.back_ticks,
                        ground_y,
                    )
            })
            .map(|r| r.command_code)
            .collect()
    }

    /// Apply configured sequence transitions in order, only from their source state.
    pub fn apply_sequences(
        &self,
        fighter: &mut FighterState,
        input: InputFrame,
        rules: &[SequenceRule],
        ground_y: f64,
    ) -> Vec<(u8, u8)> {
        let mut out = vec![];
        for r in rules {
            if fighter.sequence_state == r.from_state
                && rule_matches(
                    &r.conditions,
                    fighter,
                    input,
                    self.down_ticks,
                    self.back_ticks,
                    ground_y,
                )
            {
                fighter.sequence_state = r.to_state;
                fighter.sequence_timer = 9;
                out.push((r.from_state, r.to_state));
            }
        }
        out
    }

    /// Pascal decrements SeqTempo after evaluating every sequence rule. Hit-stop
    /// and screen flash pause only this timeout, not the direction counters/rules.
    pub fn tick_sequence_timeout(&self, fighter: &mut FighterState, paused: bool) {
        if fighter.sequence_timer > 0 && !paused {
            fighter.sequence_timer -= 1;
            if fighter.sequence_timer == 0 {
                fighter.sequence_state = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat_runtime::{BodyBox, ACTION_IDLE, FORCE_NONE};

    fn fighter(facing: u8) -> FighterState {
        FighterState {
            sprite_frame: 0,
            character_id: 1,
            action_id: ACTION_IDLE,
            action_frame: 0,
            action_duration: 99,
            facing,
            force_strength: FORCE_NONE,
            defending: false,
            invulnerable: false,
            hit_count: 0,
        attack_active: false,
            is_kick: false,
            recoil: 0,
            vertical_velocity: 0,
            in_air: false,
            x: 0.,
            y: 240.,
            speed: 0.,
            distance_ai: 0,
            health: 100.,
            power: 0,
            charge_power: 0.,
            jump_direction: 0,
            low_time: 0,
            back_time: 0,
            sequence_state: 0,
            sequence_timer: 0,
            punch_charge: 0,
            kick_charge: 0,
            body: BodyBox {
                x1: -1.,
                y1: 0.,
                x2: 1.,
                y2: 2.,
            },
        }
    }

    #[test]
    fn parser_accepts_atoms_and_rejects_unknown() {
        for t in [
            "B", "F", "T", "C", "H", "A", "b3", "t2", "P4", "s5", "k6", "!F",
        ] {
            assert!(parse_condition(t).is_ok(), "{t}");
        }
        assert!(parse_condition("Z").is_err());
        assert!(parse_condition("!").is_err());
    }

    #[test]
    fn direction_is_relative_to_facing() {
        let f = fighter(SIDE_LEFT);
        let c = [parse_condition("T").unwrap()];
        assert!(rule_matches(
            &c,
            &f,
            InputFrame {
                direction: Direction::Right,
                ..Default::default()
            },
            0,
            0,
            240.
        ));
        assert!(!rule_matches(
            &c,
            &f,
            InputFrame {
                direction: Direction::Left,
                ..Default::default()
            },
            0,
            0,
            240.
        ));
    }

    #[test]
    fn counters_cap_then_decay() {
        let mut r = CommandRecognizer::default();
        let mut f = fighter(SIDE_RIGHT);
        for _ in 0..25 {
            r.update_counters(
                &mut f,
                InputFrame {
                    direction: Direction::Down,
                    ..Default::default()
                },
            );
        }
        assert_eq!(r.counters().0, 18);
        r.update_counters(&mut f, InputFrame::default());
        assert_eq!(r.counters().0, 17);
    }

    #[test]
    fn matching_rules_preserve_order_and_allow_multiple() {
        let mut f = fighter(SIDE_RIGHT);
        let mut r = CommandRecognizer::default();
        r.update_counters(
            &mut f,
            InputFrame {
                direction: Direction::Down,
                ..Default::default()
            },
        );
        let rules = vec![
            CommandRule {
                attack_button: AttackButton::WeakPunch,
                command_code: 3,
                conditions: vec![parse_condition("B").unwrap()],
            },
            CommandRule {
                attack_button: AttackButton::WeakPunch,
                command_code: 16,
                conditions: vec![parse_condition("b1").unwrap()],
            },
            CommandRule {
                attack_button: AttackButton::WeakPunch,
                command_code: 5,
                conditions: vec![],
            },
        ];
        let input = InputFrame {
            pressed_buttons: 1,
            direction: Direction::Down,
            ..Default::default()
        };
        assert_eq!(
            r.commands_for_button(&f, input, AttackButton::WeakPunch, &rules, 240.),
            vec![3, 16, 5]
        );
    }

    #[test]
    fn sequence_rule_requires_matching_from_state() {
        let mut f = fighter(SIDE_RIGHT);
        let rules = vec![SequenceRule {
            from_state: 0,
            to_state: 1,
            conditions: vec![parse_condition("F").unwrap()],
        }];
        assert_eq!(
            CommandRecognizer::default().apply_sequences(
                &mut f,
                InputFrame {
                    direction: Direction::Right,
                    ..Default::default()
                },
                &rules,
                240.
            ),
            vec![(0, 1)]
        );
        assert_eq!(f.sequence_state, 1);
    }
}
