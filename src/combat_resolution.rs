//! Source-ordered melee consequence rules from Pascal Estado7_Luta.inc.
//!
//! This module is deliberately renderer-free. It converts an already-detected
//! contact into fighter-state consequences; the caller remains responsible for
//! running the attacker's character-specific [Acerto] ROT section afterwards.

use crate::combat_runtime::{
    FighterState, ACTION_AIR_STRONG_KICK, ACTION_AIR_STRONG_PUNCH, ACTION_AIR_WEAK_KICK,
    ACTION_AIR_WEAK_PUNCH, ACTION_CROUCH_STRONG_KICK, ACTION_CROUCH_WEAK_KICK, ACTION_DEFENDING,
    ACTION_LOW_DEFENDING, ACTION_SPECIAL2, ACTION_SPECIAL3, ACTION_SPECIAL4, ACTION_SPECIAL_FINAL,
    FORCE_STRONG, FORCE_WEAK,
};
use crate::melee_collision::MeleeCollision;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombatResolutionContext {
    /// Pascal NivelDano. Kept explicit instead of silently assuming a difficulty.
    pub damage_level: f64,
    /// Pascal AvisoEntrada suppresses health mutation while the intro warning is active.
    pub entry_warning: bool,
    /// Pascal RelogioParado suppresses health mutation during the stopped clock.
    pub clock_stopped: bool,
    /// Pascal ChaoY, used by the standard knockdown predicate.
    pub ground_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CombatResolution {
    Blocked {
        attacker_index: usize,
        defender_index: usize,
        chip_damage: f64,
        defender_knocked_out: bool,
    },
    Hit {
        attacker_index: usize,
        defender_index: usize,
        /// Chip damage already applied before a guard-break KO continued as a hit.
        chip_damage: f64,
        damage: f64,
        combo_count: u8,
        defender_knocked_out: bool,
        knockdown_after_reaction: bool,
        defender_reaction: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatResolutionError {
    InvalidFighterIndex,
    InvalidDamageLevel,
    GuardNotBroken,
}

fn is_low_attack(action_id: u16) -> bool {
    matches!(
        action_id,
        ACTION_CROUCH_WEAK_KICK | ACTION_CROUCH_STRONG_KICK
    )
}

fn is_air_attack(action_id: u16) -> bool {
    matches!(
        action_id,
        ACTION_AIR_WEAK_PUNCH
            | ACTION_AIR_WEAK_KICK
            | ACTION_AIR_STRONG_PUNCH
            | ACTION_AIR_STRONG_KICK
    )
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CharacterHitOverrides {
    combo_window: Option<u8>,
    hitstun: Option<u8>,
    spinning: bool,
    vertical_launch: Option<i8>,
}

/// C17's character-specific branch table from `melee_step`.
/// Pascal/ROT remains authoritative for the original game; C17 is differential evidence.
fn character_hit_overrides(
    character_id: u8,
    action_id: u16,
    action_frame: u16,
    force_strength: u8,
) -> CharacterHitOverrides {
    let strong = force_strength == FORCE_STRONG;
    let mut result = CharacterHitOverrides::default();
    match character_id {
        1 if action_id == ACTION_SPECIAL2 && strong => result.hitstun = Some(20),
        2 if action_id == ACTION_SPECIAL2 && strong => result.hitstun = Some(13),
        4 => {
            if action_id == ACTION_SPECIAL2 { result.combo_window = Some(46); }
            if action_id == ACTION_SPECIAL3 { result.combo_window = Some(47); }
            if action_id == ACTION_SPECIAL_FINAL {
                result.hitstun = Some(20);
                result.spinning = true;
                result.vertical_launch = Some(-30);
            }
        }
        5 if action_id == ACTION_SPECIAL_FINAL && action_frame >= 15 => {
            result.spinning = true;
            result.vertical_launch = Some(-30);
        }
        6 => {
            if action_id == ACTION_SPECIAL_FINAL {
                result.combo_window = Some(50);
                if action_frame >= 22 { result.spinning = true; }
            }
            if action_id == ACTION_SPECIAL3 && strong { result.hitstun = Some(25); }
        }
        7 if action_id == ACTION_SPECIAL3 && strong => result.hitstun = Some(18),
        9 => {
            if action_id == ACTION_SPECIAL_FINAL && action_frame >= 14 { result.spinning = true; }
            if action_id == ACTION_SPECIAL_FINAL && action_frame >= 15 { result.hitstun = Some(25); }
        }
        10 => {
            if action_id == ACTION_SPECIAL_FINAL && action_frame >= 15 { result.spinning = true; }
            if action_id == ACTION_SPECIAL4 && strong { result.hitstun = Some(13); }
        }
        11 if action_id == ACTION_SPECIAL_FINAL && action_frame >= 20 => result.spinning = true,
        12 => {
            if action_id == ACTION_SPECIAL4 && action_frame >= 22 { result.spinning = true; }
            if action_id == ACTION_SPECIAL2 && strong { result.hitstun = Some(25); }
        }
        15 if action_id == ACTION_SPECIAL3 => {
            result.combo_window = Some(50);
            result.vertical_launch = Some(-26);
        }
        _ => {}
    }
    result
}

/// Apply the common Pascal block/hit consequences to one detected contact.
///
/// Source order preserved:
/// 1. Wrong-height guard is cancelled.
/// 2. If guard remains, apply chip damage and consume the attack.
/// 3. Otherwise update facing/combo/damage and return the source-selected reaction
///    and knockdown predicates. FightSimulation executes the ordered ROT actions,
///    writes recoil, consumes both active flags, and finally runs [Acerto].
/// Resolve one detected contact, including the initial guard decision.
pub fn resolve_melee_contact(
    fighters: &mut [FighterState; 2],
    contact: MeleeCollision,
    context: CombatResolutionContext,
) -> Result<CombatResolution, CombatResolutionError> {
    resolve_melee_contact_inner(fighters, contact, context, false, 0.0)
}

/// Continue a chip-damage KO after the source-ordered knockdown action has
/// run and cleared the defender's guard. This skips the guard branch so chip
/// damage is not applied a second time.
pub(crate) fn resolve_melee_contact_after_guard_break(
    fighters: &mut [FighterState; 2],
    contact: MeleeCollision,
    context: CombatResolutionContext,
    chip_damage: f64,
) -> Result<CombatResolution, CombatResolutionError> {
    resolve_melee_contact_inner(fighters, contact, context, true, chip_damage)
}

fn resolve_melee_contact_inner(
    fighters: &mut [FighterState; 2],
    contact: MeleeCollision,
    context: CombatResolutionContext,
    guard_already_broken: bool,
    prior_chip_damage: f64,
) -> Result<CombatResolution, CombatResolutionError> {
    if contact.attacker_index > 1
        || contact.defender_index > 1
        || contact.attacker_index == contact.defender_index
    {
        return Err(CombatResolutionError::InvalidFighterIndex);
    }
    if !context.damage_level.is_finite() || context.damage_level < 0.0 {
        return Err(CombatResolutionError::InvalidDamageLevel);
    }

    let attacker_index = contact.attacker_index;
    let defender_index = contact.defender_index;
    let attacker_action = fighters[attacker_index].action_id;

    // Pascal L7_Acertos: crouching kicks beat standing guard; aerial attacks
    // beat crouching guard. The failed guard is cleared before the branch below.
    if is_low_attack(attacker_action) && fighters[defender_index].action_id == ACTION_DEFENDING {
        fighters[defender_index].defending = false;
    }
    if is_air_attack(attacker_action) && fighters[defender_index].action_id == ACTION_LOW_DEFENDING
    {
        fighters[defender_index].defending = false;
    }

    if !guard_already_broken && fighters[defender_index].defending {
        let chip_damage = f64::from(contact.guard_damage) * context.damage_level;
        fighters[defender_index].action_frame = 1;
        if !context.entry_warning && !context.clock_stopped {
            fighters[defender_index].health -= chip_damage;
        }
        let knocked_out = fighters[defender_index].health <= 0.0;
        return Ok(CombatResolution::Blocked {
            attacker_index,
            defender_index,
            chip_damage: if context.entry_warning || context.clock_stopped {
                0.0
            } else {
                chip_damage
            },
            defender_knocked_out: knocked_out,
        });
    }

    if guard_already_broken && fighters[defender_index].defending {
        return Err(CombatResolutionError::GuardNotBroken);
    }

    // Successful hit: the source resets the combo only when its timer is <= 35,
    // increments the Byte counter, and then writes the new combo window.
    // Pascal updates facing before combo counters and health.
    let attacker_x = fighters[attacker_index].x;
    fighters[defender_index].facing = if fighters[defender_index].x < attacker_x {
        1
    } else {
        2
    };

    let (attacker_is_kick, combo_count) = {
        let attacker = &mut fighters[attacker_index];
        if attacker.hit_timer <= 35 {
            attacker.hit_count = 0;
        }
        attacker.hit_count = attacker.hit_count.wrapping_add(1);
        attacker.hit_timer = if matches!(
            attacker_action,
            ACTION_CROUCH_STRONG_KICK | ACTION_AIR_STRONG_KICK
        ) {
            50
        } else {
            44
        };
        (attacker.is_kick, attacker.hit_count)
    };

    let attacker_character = fighters[attacker_index].character_id;
    let attacker_frame = fighters[attacker_index].action_frame;
    let attacker_force = fighters[attacker_index].force_strength;
    let overrides = character_hit_overrides(attacker_character, attacker_action, attacker_frame, attacker_force);
    if let Some(window) = overrides.combo_window {
        fighters[attacker_index].hit_timer = window;
    }

    // C17 initializes slide distance on each successful hit. Rotation and launch
    // are only overwritten when the source branch actually assigns them.
    let defender_facing = fighters[defender_index].facing;
    {
        let defender = &mut fighters[defender_index];
        defender.hitstun = overrides.hitstun.unwrap_or(4);
        if overrides.spinning {
            defender.rotation_step = if defender_facing == 1 { -20 } else { 20 };
        }
        if let Some(vertical_launch) = overrides.vertical_launch {
            defender.vertical_velocity = vertical_launch;
        }
    }

    let damage = f64::from(contact.damage) * context.damage_level;
    let defender_action;
    {
        let defender = &mut fighters[defender_index];
        if !context.entry_warning && !context.clock_stopped {
            defender.health -= damage;
        }
        defender_action = if matches!(
            attacker_action,
            crate::combat_runtime::ACTION_WEAK_KICK
                | crate::combat_runtime::ACTION_STRONG_KICK
                | crate::combat_runtime::ACTION_CROUCH_WEAK_PUNCH
                | ACTION_CROUCH_WEAK_KICK
                | crate::combat_runtime::ACTION_CROUCH_STRONG_PUNCH
        ) {
            18
        } else {
            17
        };
    }

    let defender = &fighters[defender_index];
    Ok(CombatResolution::Hit {
        attacker_index,
        defender_index,
        chip_damage: prior_chip_damage,
        damage: if context.entry_warning || context.clock_stopped {
            0.0
        } else {
            damage
        },
        combo_count,
        defender_knocked_out: defender.health <= 0.0,
        knockdown_after_reaction: attacker_is_kick
            || defender.vertical_velocity != 0
            || context.ground_y > defender.y,
        defender_reaction: defender_action,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat_runtime::{BodyBox, FighterState};
    use crate::melee_collision::MeleeCollision;
    use crate::Aabb;

    fn fighter() -> FighterState {
        FighterState {
            sprite_frame: 0,
            character_id: 1,
            action_id: 1,
            action_frame: 0,
            action_duration: 99,
            facing: 1,
            force_strength: 1,
            defending: false,
            invulnerable: false,
            hit_count: 0,
            hit_timer: 0,
            attack_damage: 12,
            angular_velocity: 0,
            attack_active: true,
            walking_forward: false,
            attack_box: None,
            is_kick: false,
            recoil: 0,
            vertical_velocity: 0,
            in_air: false,
            x: 100.0,
            y: 300.0,
            speed: 2.0,
            distance_ai: 0,
            health: 100.0,
            power: 0,
            charge_power: 0.0,
            jump_direction: 0,
            low_time: 0,
            back_time: 0,
            sequence_state: 0,
            sequence_timer: 0,
            punch_charge: 0,
            kick_charge: 0,
            body: BodyBox {
                x1: -10.0,
                y1: 0.0,
                x2: 10.0,
                y2: 100.0,
            },
        }
    }

    fn contact(attacker_index: usize, defender_index: usize) -> MeleeCollision {
        MeleeCollision {
            attacker_index,
            defender_index,
            selector: 1,
            action_timer: 0,
            strength: 1,
            damage: 20,
            guard_damage: 5,
            attack_bounds: Aabb {
                left: 0,
                top: 0,
                right: 10,
                bottom: 10,
            },
            hurt_bounds: Aabb {
                left: 5,
                top: 5,
                right: 15,
                bottom: 15,
            },
        }
    }

    fn context() -> CombatResolutionContext {
        CombatResolutionContext {
            damage_level: 1.0,
            entry_warning: false,
            clock_stopped: false,
            ground_y: 300.0,
        }
    }

    #[test]
    fn successful_hit_applies_damage_combo_reaction_and_consumes_both_attacks() {
        let mut fighters = [fighter(), fighter()];
        fighters[0].x = 100.0;
        fighters[1].x = 120.0;
        let result = resolve_melee_contact(&mut fighters, contact(0, 1), context()).unwrap();
        assert_eq!(fighters[1].health, 80.0);
        assert_eq!(fighters[0].hit_count, 1);
        assert_eq!(fighters[0].hit_timer, 44);
        assert_eq!(fighters[1].action_id, 1);
        assert_eq!(fighters[1].recoil, 0);
        assert!(fighters[0].attack_active);
        assert!(fighters[1].attack_active);
        assert!(matches!(
            result,
            CombatResolution::Hit {
                damage: 20.0,
                combo_count: 1,
                ..
            }
        ));
    }

    #[test]
    fn standing_guard_blocks_and_applies_chip_damage() {
        let mut fighters = [fighter(), fighter()];
        fighters[1].defending = true;
        fighters[1].action_id = ACTION_DEFENDING;
        let result = resolve_melee_contact(&mut fighters, contact(0, 1), context()).unwrap();
        assert_eq!(fighters[1].health, 95.0);
        assert!(fighters[0].attack_active);
        assert!(fighters[1].attack_active);
        assert!(matches!(
            result,
            CombatResolution::Blocked {
                chip_damage: 5.0,
                ..
            }
        ));
    }

    #[test]
    fn crouching_kick_breaks_standing_guard_and_becomes_a_hit() {
        let mut fighters = [fighter(), fighter()];
        fighters[0].action_id = ACTION_CROUCH_WEAK_KICK;
        fighters[1].defending = true;
        fighters[1].action_id = ACTION_DEFENDING;
        let result = resolve_melee_contact(&mut fighters, contact(0, 1), context()).unwrap();
        assert!(matches!(result, CombatResolution::Hit { .. }));
        assert_eq!(fighters[1].health, 80.0);
    }

    #[test]
    fn aerial_attack_breaks_low_guard() {
        let mut fighters = [fighter(), fighter()];
        fighters[0].action_id = ACTION_AIR_WEAK_PUNCH;
        fighters[1].defending = true;
        fighters[1].action_id = ACTION_LOW_DEFENDING;
        let result = resolve_melee_contact(&mut fighters, contact(0, 1), context()).unwrap();
        assert!(matches!(result, CombatResolution::Hit { .. }));
    }

    #[test]
    fn chip_damage_ko_continues_as_a_hit_without_applying_chip_twice() {
        let mut fighters = [fighter(), fighter()];
        fighters[1].defending = true;
        fighters[1].action_id = ACTION_DEFENDING;
        fighters[1].health = 5.0;

        let blocked = resolve_melee_contact(&mut fighters, contact(0, 1), context()).unwrap();
        assert!(matches!(
            blocked,
            CombatResolution::Blocked {
                chip_damage: 5.0,
                defender_knocked_out: true,
                ..
            }
        ));
        assert_eq!(fighters[1].health, 0.0);
        assert_eq!(fighters[0].hit_count, 0);

        // The real FightSimulation path runs common [Acao](cmDerrubar) here.
        // Its first statement clears defending before L7_Acertos tests the guard again.
        fighters[1].defending = false;
        let hit =
            resolve_melee_contact_after_guard_break(&mut fighters, contact(0, 1), context(), 5.0)
                .unwrap();

        assert!(matches!(
            hit,
            CombatResolution::Hit {
                chip_damage: 5.0,
                damage: 20.0,
                combo_count: 1,
                ..
            }
        ));
        assert_eq!(fighters[1].health, -20.0);
        assert_eq!(fighters[0].hit_count, 1);
        assert_eq!(fighters[0].hit_timer, 44);
    }

    #[test]
    fn chip_ko_continuation_rejects_a_guard_that_was_not_cleared() {
        let mut fighters = [fighter(), fighter()];
        fighters[1].defending = true;
        fighters[1].action_id = ACTION_DEFENDING;

        let result =
            resolve_melee_contact_after_guard_break(&mut fighters, contact(0, 1), context(), 5.0);

        assert_eq!(result, Err(CombatResolutionError::GuardNotBroken));
        assert_eq!(fighters[1].health, 100.0);
    }

    #[test]
    fn combo_windows_match_common_pascal_rules_and_reset_at_thirty_five() {
        for (action_id, expected_timer) in [
            (crate::combat_runtime::ACTION_STRONG_PUNCH, 44),
            (ACTION_CROUCH_STRONG_KICK, 50),
            (ACTION_AIR_STRONG_KICK, 50),
        ] {
            let mut fighters = [fighter(), fighter()];
            fighters[0].action_id = action_id;
            fighters[0].hit_count = 3;
            fighters[0].hit_timer = 36;

            resolve_melee_contact(&mut fighters, contact(0, 1), context()).unwrap();

            assert_eq!(fighters[0].hit_count, 4, "action {action_id}");
            assert_eq!(fighters[0].hit_timer, expected_timer, "action {action_id}");
        }

        let mut fighters = [fighter(), fighter()];
        fighters[0].hit_count = 3;
        fighters[0].hit_timer = 35;
        resolve_melee_contact(&mut fighters, contact(0, 1), context()).unwrap();
        assert_eq!(fighters[0].hit_count, 1);
        assert_eq!(fighters[0].hit_timer, 44);
    }

    #[test]
    fn airborne_defender_is_marked_for_post_reaction_knockdown() {
        let mut fighters = [fighter(), fighter()];
        fighters[1].vertical_velocity = -12;

        let result = resolve_melee_contact(&mut fighters, contact(0, 1), context()).unwrap();

        assert!(matches!(
            result,
            CombatResolution::Hit {
                knockdown_after_reaction: true,
                ..
            }
        ));
    }

    #[test]
    fn damage_is_suppressed_during_entry_warning_or_stopped_clock() {
        for mut ctx in [context(), context()] {
            let mut fighters = [fighter(), fighter()];
            ctx.entry_warning = true;
            let result = resolve_melee_contact(&mut fighters, contact(0, 1), ctx).unwrap();
            assert!(matches!(result, CombatResolution::Hit { damage: 0.0, .. }));
            assert_eq!(fighters[1].health, 100.0);
        }
        let mut fighters = [fighter(), fighter()];
        let mut ctx = context();
        ctx.clock_stopped = true;
        resolve_melee_contact(&mut fighters, contact(0, 1), ctx).unwrap();
        assert_eq!(fighters[1].health, 100.0);
    }

    fn resolve_character_hit(
        character_id: u8, action_id: u16, action_frame: u16, force_strength: u8, defender_x: f64,
    ) -> [FighterState; 2] {
        let mut fighters = [fighter(), fighter()];
        fighters[0].character_id = character_id;
        fighters[0].action_id = action_id;
        fighters[0].action_frame = action_frame;
        fighters[0].force_strength = force_strength;
        fighters[0].x = 100.0;
        fighters[1].x = defender_x;
        resolve_melee_contact(&mut fighters, contact(0, 1), context()).unwrap();
        fighters
    }

    #[test]
    fn c17_combo_windows_apply_per_character_special_overrides() {
        for (character, action, expected) in [
            (4, ACTION_SPECIAL2, 46), (4, ACTION_SPECIAL3, 47),
            (6, ACTION_SPECIAL_FINAL, 50), (15, ACTION_SPECIAL3, 50),
        ] {
            let fighters = resolve_character_hit(character, action, 0, FORCE_STRONG, 120.0);
            assert_eq!(fighters[0].hit_timer, expected, "C{character:02} action {action}");
        }
    }

    #[test]
    fn c17_hitstun_values_match_the_character_action_force_table() {
        for (character, action, frame, force, expected) in [
            (1, ACTION_SPECIAL2, 0, FORCE_STRONG, 20),
            (2, ACTION_SPECIAL2, 0, FORCE_STRONG, 13),
            (4, ACTION_SPECIAL_FINAL, 0, FORCE_WEAK, 20),
            (6, ACTION_SPECIAL3, 0, FORCE_STRONG, 25),
            (7, ACTION_SPECIAL3, 0, FORCE_STRONG, 18),
            (9, ACTION_SPECIAL_FINAL, 15, FORCE_WEAK, 25),
            (10, ACTION_SPECIAL4, 0, FORCE_STRONG, 13),
            (12, ACTION_SPECIAL2, 0, FORCE_STRONG, 25),
        ] {
            let fighters = resolve_character_hit(character, action, frame, force, 120.0);
            assert_eq!(fighters[1].hitstun, expected, "C{character:02} action {action}");
        }
        let ordinary = resolve_character_hit(3, ACTION_SPECIAL2, 0, FORCE_STRONG, 120.0);
        assert_eq!(ordinary[1].hitstun, 4);
    }

    #[test]
    fn c17_spinning_specials_set_signed_rotation_at_exact_frame_gates() {
        for (character, action, frame) in [
            (4, ACTION_SPECIAL_FINAL, 0), (5, ACTION_SPECIAL_FINAL, 15),
            (6, ACTION_SPECIAL_FINAL, 22), (9, ACTION_SPECIAL_FINAL, 14),
            (10, ACTION_SPECIAL_FINAL, 15), (11, ACTION_SPECIAL_FINAL, 20),
            (12, ACTION_SPECIAL4, 22),
        ] {
            let right = resolve_character_hit(character, action, frame, FORCE_WEAK, 120.0);
            assert_eq!(right[1].rotation_step, 20, "C{character:02} right-facing");
            let left = resolve_character_hit(character, action, frame, FORCE_WEAK, 80.0);
            assert_eq!(left[1].rotation_step, -20, "C{character:02} left-facing");
        }
        for (character, action, frame) in [
            (5, ACTION_SPECIAL_FINAL, 14), (6, ACTION_SPECIAL_FINAL, 21),
            (9, ACTION_SPECIAL_FINAL, 13), (10, ACTION_SPECIAL_FINAL, 14),
            (11, ACTION_SPECIAL_FINAL, 19), (12, ACTION_SPECIAL4, 21),
        ] {
            let fighters = resolve_character_hit(character, action, frame, FORCE_WEAK, 120.0);
            assert_eq!(fighters[1].rotation_step, 0, "C{character:02} before spin gate");
        }
    }

    #[test]
    fn c17_vertical_launches_are_character_and_move_specific() {
        for (character, action, frame, expected) in [
            (4, ACTION_SPECIAL_FINAL, 0, -30),
            (5, ACTION_SPECIAL_FINAL, 15, -30),
            (15, ACTION_SPECIAL3, 0, -26),
        ] {
            let fighters = resolve_character_hit(character, action, frame, FORCE_WEAK, 120.0);
            assert_eq!(fighters[1].vertical_velocity, expected, "C{character:02}");
        }
        let before_gate = resolve_character_hit(5, ACTION_SPECIAL_FINAL, 14, FORCE_WEAK, 120.0);
        assert_eq!(before_gate[1].vertical_velocity, 0);
    }


}
