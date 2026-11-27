//! Fixed-capacity, source-ordered live effect instances for the combat core.
//!
//! The arena follows the Pascal effect-allocation contract: first free slot wins,
//! type defaults are applied before the next effect tick, and a full arena is a
//! no-op. The ordered catalog below loads all 40 source-defined effect types.
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
    /// World positions and original special-action state for owners 1 and 2.
    pub owner_positions: [(f64, f64); 2],
    pub owner_special2: [bool; 2],
    pub owner_special5: [bool; 2],
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
    catalog: EffectCatalog,
}

/// Ordered, data-driven view of the original Pascal [EfeitoN] sections.
/// Entries remain in source order because duplicate keys and Random() call order
/// are observable parts of the original game.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EffectCatalog {
    definitions: std::collections::BTreeMap<u8, Vec<(String, String)>>,
}
impl EffectCatalog {
    pub fn from_ini(source: &str) -> Result<Self, String> {
        let mut catalog = Self::default();
        let mut current: Option<u8> = None;
        for (line_number, raw) in source.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                let name = line[1..line.len() - 1].trim();
                let number = name
                    .strip_prefix("Efeito")
                    .or_else(|| name.strip_prefix("efeito"))
                    .and_then(|value| value.parse::<u8>().ok());
                let Some(number) = number else {
                    current = None;
                    continue;
                };
                if !(1..=40).contains(&number) {
                    return Err(format!(
                        "effect section [{name}] at line {} is outside 1..=40",
                        line_number + 1
                    ));
                }
                current = Some(number);
                catalog.definitions.entry(number).or_default();
                continue;
            }
            let Some(number) = current else {
                continue;
            };
            let Some((key, value)) = line.split_once('=') else {
                return Err(format!(
                    "invalid effect INI entry at line {}",
                    line_number + 1
                ));
            };
            catalog
                .definitions
                .entry(number)
                .or_default()
                .push((key.trim().to_owned(), value.trim().to_owned()));
        }
        if catalog.definitions.len() != 40 {
            return Err(format!(
                "expected 40 Pascal effect definitions, found {}",
                catalog.definitions.len()
            ));
        }
        Ok(catalog)
    }
    pub fn definition_count(&self) -> usize {
        self.definitions.len()
    }
    pub fn contains(&self, effect_type: u8) -> bool {
        self.definitions.contains_key(&effect_type)
    }
    fn definition(&self, effect_type: u8) -> Option<&[(String, String)]> {
        self.definitions.get(&effect_type).map(Vec::as_slice)
    }
}
impl Default for EffectArena {
    fn default() -> Self {
        let catalog =
            EffectCatalog::from_ini(include_str!("../research/pascal-effects/Efeitos.ini"))
                .expect("bundled Pascal effect catalog must contain Efeito1..Efeito40");
        Self {
            slots: std::array::from_fn(|_| None),
            catalog,
        }
    }
}

fn eval_random_expression<R: PascalRandomSource>(
    expression: &str,
    rng: &mut R,
) -> Result<i32, EffectSpawnError<R::Error>> {
    let compact = expression.replace(' ', "");
    if let Some((_, denominator)) = compact.split_once("em") {
        let range = denominator.parse::<i32>().unwrap_or(1).max(1);
        return Ok(i32::from(
            rng.draw_longint(range).map_err(EffectSpawnError::Rng)? == 0,
        ));
    }
    let mut total = 0_i32;
    let mut index = 0_usize;
    let bytes = compact.as_bytes();
    while index < bytes.len() {
        let mut sign = 1_i32;
        if bytes[index] == b'+' {
            index += 1;
        } else if bytes[index] == b'-' {
            sign = -1;
            index += 1;
        }
        let start = index;
        while index < bytes.len() && bytes[index] != b'+' && bytes[index] != b'-' {
            index += 1;
        }
        let term = &compact[start..index];
        if term.is_empty() {
            continue;
        }
        let value = if let Some((coefficient, random)) = term.split_once("*R") {
            let coefficient = coefficient.parse::<i32>().unwrap_or(1);
            let range = random.parse::<i32>().unwrap_or(1).max(1);
            coefficient.saturating_mul(rng.draw_longint(range).map_err(EffectSpawnError::Rng)?)
        } else if let Some(random) = term.strip_prefix('R') {
            let range = random.parse::<i32>().unwrap_or(1).max(1);
            rng.draw_longint(range).map_err(EffectSpawnError::Rng)?
        } else {
            term.parse::<i32>()
                .map_err(|_| EffectSpawnError::UnsupportedType(0))?
        };
        total = total.saturating_add(sign.saturating_mul(value));
    }
    Ok(total)
}
fn eval_condition_value(value: &str, effect: &EffectInstance) -> Option<i32> {
    let compact = value.replace(' ', "");
    let mut total = 0_i32;
    let mut index = 0_usize;
    let bytes = compact.as_bytes();
    while index < bytes.len() {
        let mut sign = 1_i32;
        if bytes[index] == b'+' {
            index += 1;
        } else if bytes[index] == b'-' {
            sign = -1;
            index += 1;
        }
        let start = index;
        while index < bytes.len() && bytes[index] != b'+' && bytes[index] != b'-' {
            index += 1;
        }
        let term = &compact[start..index];
        if term.is_empty() {
            continue;
        }
        let part = match term {
            "Q" => i32::from(effect.frame),
            "C" => i32::from(effect.count),
            "D" => i32::from(effect.duration),
            "L" => i32::from(effect.count_limit),
            _ => term.parse::<i32>().ok()?,
        };
        total = total.saturating_add(sign.saturating_mul(part));
    }
    Some(total)
}
fn effect_condition_matches(condition: &str, effect: &EffectInstance) -> bool {
    let condition = condition.trim();
    if condition == "*" {
        return true;
    }
    for clause in condition.split('&') {
        let clause = clause.trim();
        let (left, operator, right) = if let Some((a, b)) = clause.split_once(">=") {
            (a, ">=", b)
        } else if let Some((a, b)) = clause.split_once("<=") {
            (a, "<=", b)
        } else if let Some((a, b)) = clause.split_once('>') {
            (a, ">", b)
        } else if let Some((a, b)) = clause.split_once('<') {
            (a, "<", b)
        } else if let Some((a, b)) = clause.split_once('=') {
            (a, "=", b)
        } else {
            return false;
        };
        let (Some(left), Some(right)) = (
            eval_condition_value(left, effect),
            eval_condition_value(right, effect),
        ) else {
            return false;
        };
        let matches = match operator {
            ">=" => left >= right,
            "<=" => left <= right,
            ">" => left > right,
            "<" => left < right,
            "=" => left == right,
            _ => false,
        };
        if !matches {
            return false;
        }
    }
    true
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
        let Some(definition) = self.catalog.definition(effect_type) else {
            return Err(EffectSpawnError::UnsupportedType(effect_type));
        };
        let has_variant_fields = definition
            .iter()
            .any(|(key, _)| key.ends_with(".A") || key.ends_with(".B"));
        if has_variant_fields && !matches!(source_variant, Some('A' | 'B')) {
            return Err(EffectSpawnError::InvalidVariant { effect_type });
        }
        if source_variant.is_some() && !has_variant_fields {
            return Err(EffectSpawnError::InvalidVariant { effect_type });
        }
        for (raw_key, value) in definition {
            if raw_key.starts_with("Anim") {
                continue;
            }
            let mut key = raw_key.as_str();
            if let Some((base, variant)) = key.rsplit_once('.') {
                if variant == "A" || variant == "B" {
                    if Some(variant.chars().next().unwrap()) != source_variant {
                        continue;
                    }
                    key = base;
                }
            }
            let key = key.split('#').next().unwrap_or(key);
            match key {
                "Duracao" => {
                    effect.duration = eval_random_expression(value, rng)?.clamp(0, 255) as u8
                }
                "Vel" => {
                    effect.horizontal_velocity =
                        eval_random_expression(value, rng)?.clamp(-32768, 32767) as i16
                }
                "VelY" => {
                    effect.vertical_velocity =
                        eval_random_expression(value, rng)?.clamp(-32768, 32767) as i16
                }
                "Dano" => effect.damage = eval_random_expression(value, rng)?.clamp(0, 255) as u8,
                "DanoDefesa" => {
                    effect.guard_damage = eval_random_expression(value, rng)?.clamp(0, 255) as u8
                }
                "Sprite" => effect.sprite = eval_random_expression(value, rng)?.clamp(0, 255) as u8,
                "LimiteContagem" => {
                    effect.count_limit = eval_random_expression(value, rng)?.clamp(0, 255) as u8
                }
                "Lado" => {
                    if value.eq_ignore_ascii_case("dono") {
                        if !matches!(owner_facing, EFFECT_SIDE_RIGHT | EFFECT_SIDE_LEFT) {
                            return Err(EffectSpawnError::InvalidFacing(owner_facing));
                        }
                        effect.facing = owner_facing;
                    } else {
                        let side = eval_random_expression(value, rng)?;
                        if side != i32::from(EFFECT_SIDE_RIGHT)
                            && side != i32::from(EFFECT_SIDE_LEFT)
                        {
                            return Err(EffectSpawnError::InvalidFacing(side.clamp(0, 255) as u8));
                        }
                        effect.facing = side as u8;
                    }
                }
                "Caixa" => {
                    let values = value
                        .split(',')
                        .map(|part| part.trim().parse::<i16>())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| EffectSpawnError::UnsupportedType(effect_type))?;
                    if values.len() != 4 {
                        return Err(EffectSpawnError::UnsupportedType(effect_type));
                    }
                    effect.bounds = EffectBounds {
                        left: values[0],
                        top: values[1],
                        right: values[2],
                        bottom: values[3],
                    };
                }
                "PodeAcertar" => effect.can_hit = value == "1",
                "Terminando" => effect.terminating = value == "1",
                "Flag50" => effect.reflect = value == "1",
                "Flag51" => effect.bottom_anchor = value == "1",
                "Flag52" => effect.gravity = value == "1",
                "Variante" => effect.variant_flag = eval_random_expression(value, rng)? != 0,
                _ => {}
            }
        }
        self.slots[slot] = Some(effect);
        Ok(Some(slot))
    }

    /// Advance live instances in ascending slot order and capture their draw state.
    ///
    /// This models the supported INI animation rules and the Pascal effect loop's
    /// move/snapshot/lifetime/cull order. Effect-vs-effect and effect-vs-fighter
    /// collision consequences remain a later combat-system integration.
    pub fn advance(&mut self, context: EffectTickContext) -> EffectTickResult {
        let frozen = context.flash_active || context.hit_stop_active;
        let catalog = &self.catalog;
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

                let Some(definition) = catalog.definition(effect.effect_type) else {
                    continue;
                };
                for (key, instruction) in definition {
                    if !key.starts_with("Anim") {
                        continue;
                    }
                    let Some((condition, action)) = instruction.split_once(':') else {
                        continue;
                    };
                    if !effect_condition_matches(condition, effect) {
                        continue;
                    }
                    match action.trim() {
                        "Q0" => effect.frame = 0,
                        "E1" => effect.stage = true,
                        "SOBE" if !frozen => effect.y -= 1.0,
                        sprite if sprite.starts_with('S') => {
                            if let Ok(number) = sprite[1..].parse::<u8>() {
                                effect.sprite = number;
                            }
                        }
                        _ => {}
                    }
                }

                match effect.effect_type {
                    // Source Comum_Sprites.inc: the two-frame projectile hitbox transition.
                    8 => {
                        if effect.frame == 2 {
                            effect.bounds = EffectBounds {
                                left: -45, top: -150, right: 45, bottom: 0
                            };
                        } else if effect.frame == 5 {
                            effect.bounds = EffectBounds {
                                left: -45, top: -220, right: 45, bottom: 0
                            };
                            effect.stage = true;
                        }
                        if effect.frame == 6 || effect.frame == 12 {
                            effect.can_hit = true;
                            effect.frame = effect.frame.saturating_add(1);
                        }
                    }
                    // Source uses a three-step pulse cadence for this repeated hitbox.
                    28 => {
                        if matches!(effect.count, 3 | 6 | 9) {
                            effect.can_hit = true;
                        }
                        if !frozen {
                            effect.count = effect.count.saturating_add(1);
                        }
                    }
                    // Effects 30-32 are owned by special-action state, not by a timer.
                    30 => {
                        let owner_index = usize::from(effect.owner.saturating_sub(1));
                        if effect.owner == 0 || !context.owner_special2[owner_index.min(1)] {
                            remove = true;
                        }
                    }
                    31 | 32 => {
                        let owner_index = usize::from(effect.owner.saturating_sub(1)).min(1);
                        if effect.owner == 0 {
                            remove = true;
                        } else {
                            if effect.count % 10 == 0 {
                                if !context.owner_special5[owner_index] {
                                    remove = true;
                                }
                                effect.can_hit = true;
                                effect.count = effect.count.saturating_add(1);
                            }
                            let (owner_x, owner_y) = context.owner_positions[owner_index];
                            let orbit = (f64::from(effect.count) / 3.0).cos() * 70.0;
                            let offset = orbit.round();
                            effect.x = if effect.effect_type == 31 {
                                owner_x + offset
                            } else {
                                owner_x - offset
                            };
                            effect.y = offset + owner_y - 80.0;
                        }
                    }
                    // Source animation 24 drifts while its frame-specific sprite animates.
                    24 if !frozen => {
                        effect.x += (f64::from(effect.frame) / 2.0).cos() * 2.0;
                        effect.y -= 1.0;
                    }
                    19 if !frozen => {
                        effect.y += f64::from(effect.horizontal_velocity);
                    }
                    34 if effect.frame >= 14 => {
                        effect.can_hit = false;
                    }
                    _ => {}
                }

                // Generic horizontal movement follows animation and precedes the draw snapshot.
                if !frozen {
                    effect.y += f64::from(effect.vertical_velocity);
                    if effect.gravity {
                        effect.vertical_velocity = effect.vertical_velocity.saturating_add(4);
                    }
                    if effect.count_limit > 0 && effect.effect_type != 28 && !matches!(effect.effect_type, 31 | 32) {
                        effect.count = effect.count.saturating_add(1);
                    }
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

                if !remove {
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
                }

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

#[cfg(test)]
mod catalog_tests {
    use super::*;

    struct ZeroRng;
    impl PascalRandomSource for ZeroRng {
        type Error = &'static str;
        fn draw_longint(&mut self, _range: i32) -> Result<i32, Self::Error> {
            Ok(0)
        }
        fn draw_int64(&mut self, _range: i64) -> Result<i64, Self::Error> {
            Ok(0)
        }
    }

    #[test]
    fn bundled_ini_loads_all_forty_original_effect_sections() {
        let arena = EffectArena::default();
        assert_eq!(arena.catalog.definition_count(), 40);
        for effect_type in 1..=40 {
            assert!(arena.catalog.contains(effect_type));
        }
    }

    #[test]
    fn all_forty_effects_accept_source_configuration_and_allocate() {
        let mut arena = EffectArena::default();
        let mut rng = ZeroRng;
        for effect_type in 1..=40 {
            let definition = arena.catalog.definition(effect_type).unwrap();
            let has_a = definition.iter().any(|(key, _)| key.ends_with(".A"));
            let has_b = definition.iter().any(|(key, _)| key.ends_with(".B"));
            let variant = if has_a {
                Some('A')
            } else if has_b {
                Some('B')
            } else {
                None
            };
            let slot = arena
                .spawn(
                    effect_type,
                    variant,
                    1,
                    EFFECT_SIDE_RIGHT,
                    100.0,
                    100.0,
                    &mut rng,
                )
                .unwrap_or_else(|error| panic!("Efeito{effect_type}: {error}"));
            assert!(slot.is_some());
            arena.remove(slot.unwrap());
        }
    }

    #[test]
    fn animation_rules_follow_pascal_ini_order_and_frame_conditions() {
        let mut arena = EffectArena::default();
        let mut rng = ZeroRng;
        arena
            .spawn(1, None, 0, EFFECT_SIDE_RIGHT, 100.0, 100.0, &mut rng)
            .unwrap();
        let first = arena.advance(EffectTickContext::default());
        assert_eq!(first.render_snapshots[0].sprite, 1);
        let second = arena.advance(EffectTickContext::default());
        assert_eq!(second.render_snapshots[0].sprite, 2);
    }
}
