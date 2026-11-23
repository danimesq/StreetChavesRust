//! Fixed-capacity, source-ordered live effect instances for the combat core.
//!
//! The arena follows the Pascal effect-allocation contract: first free slot wins,
//! type defaults are applied before the next effect tick, and a full arena is a
//! no-op. This initial slice implements only the C01-used types 2, 7, 8 and 16.
//! Rendering consumes immutable snapshots; the simulation never draws here.

use crate::pascal_replay::PascalRandomSource;
use std::fmt;

pub const MAX_EFFECT_INSTANCES: usize = 80;
pub const EFFECT_SIDE_RIGHT: u8 = 1;
pub const EFFECT_SIDE_LEFT: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EffectBounds {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectInstance {
    pub effect_type: u8,
    /// A/B selects the weak/strong source configuration where applicable.
    pub source_variant: Option<char>,
    pub owner: u8,
    pub x: f64,
    pub y: f64,
    pub bounds: EffectBounds,
    pub facing: u8,
    pub horizontal_velocity: i16,
    pub vertical_velocity: i16,
    pub damage: u8,
    pub guard_damage: u8,
    pub can_hit: bool,
    pub stage: bool,
    pub variant_flag: bool,
    pub terminating: bool,
    pub bottom_anchor: bool,
    pub frame: u8,
    pub duration: u8,
    pub sprite: u8,
    pub count: u8,
    pub count_limit: u8,
    pub reflect: bool,
    pub gravity: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectSpawnError<E> {
    Rng(E),
    UnsupportedType(u8),
    InvalidOwner(u8),
    InvalidFacing(u8),
    InvalidVariant { effect_type: u8 },
}

impl<E: fmt::Display> fmt::Display for EffectSpawnError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rng(error) => write!(f, "effect configuration RNG failed: {error}"),
            Self::UnsupportedType(effect_type) => {
                write!(f, "effect type {effect_type} is not implemented")
            }
            Self::InvalidOwner(owner) => write!(f, "invalid effect owner {owner}"),
            Self::InvalidFacing(facing) => write!(f, "invalid effect facing {facing}"),
            Self::InvalidVariant { effect_type } => {
                write!(f, "effect type {effect_type} requires a valid A/B variant")
            }
        }
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for EffectSpawnError<E> {}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EffectTickContext {
    pub flash_active: bool,
    pub hit_stop_active: bool,
    pub camera_x: f64,
    pub camera_y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectRenderSnapshot {
    /// Zero-based Rust slot, corresponding to Pascal slots 1..80.
    pub slot: usize,
    pub effect_type: u8,
    pub sprite: u8,
    pub x: f64,
    pub y: f64,
    pub facing: u8,
    pub stage: bool,
    pub bottom_anchor: bool,
    pub bounds: EffectBounds,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EffectTickResult {
    /// Ordered snapshots are captured before lifetime and culling cleanup.
    pub render_snapshots: Vec<EffectRenderSnapshot>,
    pub removed_slots: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectArena {
    slots: [Option<EffectInstance>; MAX_EFFECT_INSTANCES],
}

impl Default for EffectArena {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| None),
        }
    }
}

impl EffectArena {
    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_some()).count()
    }

    pub fn instance(&self, slot: usize) -> Option<&EffectInstance> {
        self.slots.get(slot).and_then(Option::as_ref)
    }

    /// Match Pascal ExisteEfeito(tipo, Jog) against live slots and 1-based owner IDs.
    pub fn contains_type_for_owner(&self, effect_type: u8, owner: u8) -> bool {
        self.slots
            .iter()
            .flatten()
            .any(|effect| effect.effect_type == effect_type && effect.owner == owner)
    }

    pub fn instances_in_slot_order(&self) -> impl Iterator<Item = (usize, &EffectInstance)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(slot, effect)| effect.as_ref().map(|effect| (slot, effect)))
    }

    pub fn remove(&mut self, slot: usize) -> bool {
        let Some(entry) = self.slots.get_mut(slot) else {
            return false;
        };
        entry.take().is_some()
    }

    /// Apply the audited Pascal allocation defaults and INI values to the first free slot.
    ///
    /// owner_facing is read only for source types whose INI declares Lado=dono.
    /// When no slot is free, the function returns Ok(None) without consuming RNG,
    /// matching the Pascal early exit before type configuration.
    pub fn spawn<R: PascalRandomSource>(
        &mut self,
        effect_type: u8,
        source_variant: Option<char>,
        owner: u8,
        owner_facing: u8,
        x: f64,
        y: f64,
        rng: &mut R,
    ) -> Result<Option<usize>, EffectSpawnError<R::Error>> {
        let Some(slot) = self.slots.iter().position(Option::is_none) else {
            return Ok(None);
        };

        if owner > 2 {
            return Err(EffectSpawnError::InvalidOwner(owner));
        }

        let mut effect = EffectInstance {
            effect_type,
            source_variant,
            owner,
            x,
            y,
            bounds: EffectBounds::default(),
            facing: EFFECT_SIDE_RIGHT,
            horizontal_velocity: 0,
            vertical_velocity: 0,
            damage: 0,
            guard_damage: 0,
            can_hit: false,
            stage: false,
            variant_flag: false,
            terminating: false,
            bottom_anchor: false,
            frame: 0,
            duration: 0,
            sprite: 0,
            count: 0,
            count_limit: 0,
            reflect: false,
            gravity: false,
        };

        match effect_type {
            // Dados/Efeitos.ini [Efeito2]: dust/cloud effect used by C01 Esp2 and C12 victory.
            2 => {
                if source_variant.is_some() {
                    return Err(EffectSpawnError::InvalidVariant { effect_type });
                }
                effect.duration = 15;
                effect.bottom_anchor = true; // Flag51
            }
            // [Efeito7]: C01 Esp1 weak/strong projectile.
            7 => {
                let variant = source_variant
                    .filter(|value| *value == 'A' || *value == 'B')
                    .ok_or(EffectSpawnError::InvalidVariant { effect_type })?;
                if owner == 0 {
                    return Err(EffectSpawnError::InvalidOwner(owner));
                }
                if !matches!(owner_facing, EFFECT_SIDE_RIGHT | EFFECT_SIDE_LEFT) {
                    return Err(EffectSpawnError::InvalidFacing(owner_facing));
                }
                effect.duration = 5;
                effect.can_hit = true;
                effect.bottom_anchor = true;
                effect.variant_flag = rng.draw_longint(5).map_err(EffectSpawnError::Rng)? == 0;
                effect.facing = owner_facing;
                effect.horizontal_velocity = if variant == 'A' { 8 } else { 11 };
                effect.damage = 12;
                effect.guard_damage = 3;
                effect.bounds = EffectBounds {
                    left: -25,
                    top: -38,
                    right: 25,
                    bottom: 0,
                };
            }
            // [Efeito8]: C01 final special. The type is terminating and damage-active.
            8 => {
                if source_variant.is_some() {
                    return Err(EffectSpawnError::InvalidVariant { effect_type });
                }
                if owner == 0 {
                    return Err(EffectSpawnError::InvalidOwner(owner));
                }
                if !matches!(owner_facing, EFFECT_SIDE_RIGHT | EFFECT_SIDE_LEFT) {
                    return Err(EffectSpawnError::InvalidFacing(owner_facing));
                }
                effect.duration = 23;
                effect.can_hit = true;
                effect.bottom_anchor = true;
                effect.damage = 40;
                effect.guard_damage = 3;
                effect.variant_flag = true;
                effect.terminating = true;
                effect.facing = owner_facing;
                effect.bounds = EffectBounds {
                    left: -30,
                    top: -50,
                    right: 30,
                    bottom: 0,
                };
            }
            // [Efeito16]: short-lived C01 Esp2 trail.
            16 => {
                if source_variant.is_some() {
                    return Err(EffectSpawnError::InvalidVariant { effect_type });
                }
                if owner == 0 {
                    return Err(EffectSpawnError::InvalidOwner(owner));
                }
                if !matches!(owner_facing, EFFECT_SIDE_RIGHT | EFFECT_SIDE_LEFT) {
                    return Err(EffectSpawnError::InvalidFacing(owner_facing));
                }
                effect.duration = 3;
                effect.facing = owner_facing;
            }
            _ => return Err(EffectSpawnError::UnsupportedType(effect_type)),
        }

        self.slots[slot] = Some(effect);
        Ok(Some(slot))
    }

    /// Advance live instances in ascending slot order and capture their draw state.
    ///
    /// This models the C01-used INI animation rules and the Pascal effect loop's
    /// move/snapshot/lifetime/cull order. Effect-vs-effect and effect-vs-fighter
    /// collision consequences remain a later combat-system integration.
    pub fn advance(&mut self, context: EffectTickContext) -> EffectTickResult {
        let frozen = context.flash_active || context.hit_stop_active;
        let mut result = EffectTickResult::default();

        for slot in 0..MAX_EFFECT_INSTANCES {
            let Some(effect_entry) = self.slots.get_mut(slot) else {
                continue;
            };
            let mut remove = false;

            {
                let Some(effect) = effect_entry.as_mut() else {
                    continue;
                };

                match effect.effect_type {
                    2 => {
                        match effect.frame {
                            0 => effect.sprite = 4,
                            3 => effect.sprite = 5,
                            6 => effect.sprite = 6,
                            9 => effect.sprite = 7,
                            12 => effect.sprite = 8,
                            _ => {}
                        }
                        // Anim6=*:SOBE; this motion is gated by flash/hit-stop.
                        if !frozen {
                            effect.y -= 1.0;
                        }
                    }
                    7 => match effect.frame {
                        0 | 1 => effect.sprite = 13,
                        2 => effect.sprite = 14,
                        3 => effect.frame = 0, // Anim4=Q0, before lifetime advancement.
                        _ => {}
                    },
                    8 => {
                        match effect.frame {
                            0 => effect.sprite = 15,
                            2 => {
                                effect.sprite = 16;
                                effect.bounds = EffectBounds {
                                    left: -45,
                                    top: -150,
                                    right: 45,
                                    bottom: 0,
                                };
                            }
                            5 => {
                                effect.sprite = 17;
                                effect.stage = true;
                                effect.bounds = EffectBounds {
                                    left: -45,
                                    top: -220,
                                    right: 45,
                                    bottom: 0,
                                };
                            }
                            _ => {}
                        }
                        // These source writes are not pause-gated and precede the draw.
                        if effect.frame == 6 || effect.frame == 12 {
                            effect.can_hit = true;
                            effect.frame = effect.frame.saturating_add(1);
                        }
                    }
                    16 => match effect.frame {
                        0 => effect.sprite = 41,
                        2 => effect.sprite = 42,
                        4 => effect.sprite = 43,
                        _ => {}
                    },
                    _ => continue,
                }

                // Generic horizontal movement follows animation and precedes the draw snapshot.
                if !frozen {
                    match effect.facing {
                        EFFECT_SIDE_RIGHT => {
                            effect.x += f64::from(effect.horizontal_velocity);
                        }
                        EFFECT_SIDE_LEFT => {
                            effect.x -= f64::from(effect.horizontal_velocity);
                        }
                        _ => {}
                    }
                }

                result.render_snapshots.push(EffectRenderSnapshot {
                    slot,
                    effect_type: effect.effect_type,
                    sprite: effect.sprite,
                    x: effect.x,
                    y: effect.y,
                    facing: effect.facing,
                    stage: effect.stage,
                    bottom_anchor: effect.bottom_anchor,
                    bounds: effect.bounds,
                });

                if !frozen && effect.duration != 99 {
                    effect.frame = effect.frame.saturating_add(1);
                    if effect.frame > effect.duration {
                        remove = true;
                    }
                }
                // Pascal culls after drawing and lifetime checks, so the current snapshot survives.
                if effect.x < context.camera_x - 50.0
                    || effect.x > context.camera_x + 450.0
                    || effect.y < context.camera_y - 50.0
                {
                    remove = true;
                }
            }

            if remove {
                *effect_entry = None;
                result.removed_slots.push(slot);
            }
        }

        result
    }
}
