//! Renderer-free melee collision detection from the pinned Pascal L7_Acertos contract.
//!
//! Detection is separate from damage, guard, hit-stop duration, effects, and character
//! [Acerto] reactions. Sprite heights are explicit because FighterState does not yet
//! carry the source AlturaSprite value; guessing a universal height would be incorrect.

use crate::combat_runtime::{BodyBox, FighterState};
use crate::{Aabb, HitboxRecord, RawBounds};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FighterCollisionGeometry {
    pub x: f64,
    pub y: f64,
    pub sprite_height: i32,
    pub body: BodyBox,
}

impl FighterCollisionGeometry {
    pub fn from_fighter(fighter: &FighterState, sprite_height: i32) -> Self {
        Self {
            x: fighter.x,
            y: fighter.y,
            sprite_height,
            body: fighter.body,
        }
    }

    fn world_bounds(self, local: RawBounds) -> Result<Aabb, CollisionGeometryError> {
        let x = round_pascal(self.x)?;
        let y = round_pascal(self.y)?;
        Ok(Aabb {
            left: x + i32::from(local.x1),
            top: y - self.sprite_height + i32::from(local.y1),
            right: x + i32::from(local.x2),
            bottom: y - self.sprite_height + i32::from(local.y2),
        })
    }

    fn world_body(self) -> Result<Aabb, CollisionGeometryError> {
        let x = round_pascal(self.x)?;
        let y = round_pascal(self.y)?;
        Ok(Aabb {
            left: x + round_pascal(self.body.x1)?,
            top: y - self.sprite_height + round_pascal(self.body.y1)?,
            right: x + round_pascal(self.body.x2)?,
            bottom: y - self.sprite_height + round_pascal(self.body.y2)?,
        })
    }
}

/// One detected melee contact, before gameplay consequences are applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeleeCollision {
    pub attacker_index: usize,
    pub defender_index: usize,
    pub selector: u16,
    pub action_timer: u16,
    pub strength: u8,
    pub damage: u16,
    pub guard_damage: u16,
    pub attack_bounds: Aabb,
    pub hurt_bounds: Aabb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionGeometryError {
    NonFiniteCoordinate,
    CoordinateOutOfRange,
}

impl fmt::Display for CollisionGeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteCoordinate => write!(f, "collision coordinate is not finite"),
            Self::CoordinateOutOfRange => write!(f, "collision coordinate is outside i32 range"),
        }
    }
}

impl std::error::Error for CollisionGeometryError {}

/// Delphi/Pascal Round: nearest integer, with exact halves rounded to even.
fn round_pascal(value: f64) -> Result<i32, CollisionGeometryError> {
    if !value.is_finite() {
        return Err(CollisionGeometryError::NonFiniteCoordinate);
    }
    let floor = value.floor();
    let fraction = value - floor;
    let rounded = if fraction < 0.5 {
        floor
    } else if fraction > 0.5 {
        floor + 1.0
    } else if (floor as i64) % 2 == 0 {
        floor
    } else {
        floor + 1.0
    };
    if rounded < i32::MIN as f64 || rounded > i32::MAX as f64 {
        return Err(CollisionGeometryError::CoordinateOutOfRange);
    }
    Ok(rounded as i32)
}

/// Project one attack and one body box and return their strict-overlap contact.
/// attack_active is the fighter-level Pascal GolpeAtivo flag. The selected row's
/// active metadata remains separate. Hit-stop and screen flash suppress contact.
pub fn detect_melee_contact(
    attacker_index: usize,
    attacker: FighterCollisionGeometry,
    attack: HitboxRecord,
    attack_active: bool,
    defender_index: usize,
    defender: FighterCollisionGeometry,
    defender_invulnerable: bool,
    hit_stop: bool,
    screen_flash: bool,
) -> Result<Option<MeleeCollision>, CollisionGeometryError> {
    if !attack_active || defender_invulnerable || hit_stop || screen_flash {
        return Ok(None);
    }

    let attack_bounds = attacker.world_bounds(attack.bounds)?;
    let hurt_bounds = defender.world_body()?;
    if !attack_bounds.overlaps_strict(hurt_bounds) {
        return Ok(None);
    }

    Ok(Some(MeleeCollision {
        attacker_index,
        defender_index,
        selector: attack.selector,
        action_timer: attack.timer,
        strength: attack.strength,
        damage: attack.damage,
        guard_damage: attack.guard_damage,
        attack_bounds,
        hurt_bounds,
    }))
}

/// Detect both directional pairs in Pascal's P1-then-P2 order.
pub fn detect_fight_melee_contacts(
    fighters: [&FighterState; 2],
    sprite_heights: [i32; 2],
    hit_stop: bool,
    screen_flash: bool,
) -> Result<Vec<MeleeCollision>, CollisionGeometryError> {
    let mut contacts = Vec::with_capacity(2);
    for attacker_index in 0..2 {
        let defender_index = 1 - attacker_index;
        let attacker = fighters[attacker_index];
        let defender = fighters[defender_index];
        let Some(attack) = attacker.attack_box else {
            continue;
        };
        if let Some(contact) = detect_melee_contact(
            attacker_index,
            FighterCollisionGeometry::from_fighter(attacker, sprite_heights[attacker_index]),
            attack,
            attacker.attack_active,
            defender_index,
            FighterCollisionGeometry::from_fighter(defender, sprite_heights[defender_index]),
            defender.invulnerable,
            hit_stop,
            screen_flash,
        )? {
            contacts.push(contact);
        }
    }
    Ok(contacts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attack(bounds: RawBounds) -> HitboxRecord {
        HitboxRecord {
            selector: 25,
            timer: 3,
            bounds,
            active: true,
            strength: 2,
            damage: 18,
            guard_damage: 7,
        }
    }

    fn geometry(x: f64, y: f64) -> FighterCollisionGeometry {
        FighterCollisionGeometry {
            x,
            y,
            sprite_height: 180,
            body: BodyBox {
                x1: -10.0,
                y1: 20.0,
                x2: 10.0,
                y2: 160.0,
            },
        }
    }

    #[test]
    fn detects_world_space_overlap_and_preserves_attack_metadata() {
        let hit = attack(RawBounds {
            x1: 0,
            y1: 100,
            x2: 30,
            y2: 130,
        });
        let contact = detect_melee_contact(
            0,
            geometry(100.0, 240.0),
            hit,
            true,
            1,
            geometry(125.0, 240.0),
            false,
            false,
            false,
        )
        .unwrap()
        .unwrap();
        assert_eq!((contact.attacker_index, contact.defender_index), (0, 1));
        assert_eq!((contact.selector, contact.action_timer), (25, 3));
        assert_eq!(
            (contact.strength, contact.damage, contact.guard_damage),
            (2, 18, 7)
        );
        assert_eq!(
            contact.attack_bounds,
            Aabb {
                left: 100,
                top: 160,
                right: 130,
                bottom: 190
            }
        );
        assert_eq!(
            contact.hurt_bounds,
            Aabb {
                left: 115,
                top: 80,
                right: 135,
                bottom: 220
            }
        );
    }

    #[test]
    fn edge_touch_is_not_a_collision() {
        let hit = attack(RawBounds {
            x1: 0,
            y1: 100,
            x2: 10,
            y2: 130,
        });
        let contact = detect_melee_contact(
            0,
            geometry(100.0, 240.0),
            hit,
            true,
            1,
            geometry(120.0, 240.0),
            false,
            false,
            false,
        )
        .unwrap();
        assert!(contact.is_none());
    }

    #[test]
    fn attack_flag_invulnerability_hit_stop_and_flash_gate_contact() {
        let hit = attack(RawBounds {
            x1: 0,
            y1: 100,
            x2: 30,
            y2: 130,
        });
        for (active, invulnerable, hit_stop, flash) in [
            (false, false, false, false),
            (true, true, false, false),
            (true, false, true, false),
            (true, false, false, true),
        ] {
            let contact = detect_melee_contact(
                    0,
                geometry(100.0, 240.0),
                hit,
                active,
                1,
                geometry(125.0, 240.0),
                invulnerable,
                hit_stop,
                flash,
            )
            .unwrap();
            assert!(contact.is_none());
        }
    }

    #[test]
    fn pascal_round_uses_bankers_rounding_for_exact_halves() {
        assert_eq!(round_pascal(10.5), Ok(10));
        assert_eq!(round_pascal(11.5), Ok(12));
        assert_eq!(round_pascal(-10.5), Ok(-10));
        assert_eq!(round_pascal(-11.5), Ok(-12));
    }

    #[test]
    fn non_finite_positions_are_rejected() {
        let hit = attack(RawBounds {
            x1: 0,
            y1: 100,
            x2: 30,
            y2: 130,
        });
        let result = detect_melee_contact(
            0,
            geometry(f64::NAN, 240.0),
            hit,
            true,
            1,
            geometry(125.0, 240.0),
            false,
            false,
            false,
        );
        assert_eq!(result, Err(CollisionGeometryError::NonFiniteCoordinate));
    }
}
