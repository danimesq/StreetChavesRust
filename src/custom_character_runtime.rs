//! Bounded, data-only runtime definitions for user-installed fighters.
//!
//! Character packages never ship executable ROT, Rust, JavaScript or WASM code.
//! This parser accepts a deliberately small vocabulary of frame tables and
//! move events which the fight simulation interprets directly.

use crate::custom_character::{CharacterManifest, CustomCharacterPackage};
use crate::fighter_box_data::FighterFrameBoxes;
use crate::{ActionId, HurtboxRecord, PascalAttackDefinition, RawBounds};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct CustomCharacterDefinition {
    pub package_id: String,
    pub runtime_id: u8,
    pub display_name: String,
    pub sprite_height: i32,
    pub walk_speed: f64,
    pub frame_boxes: FighterFrameBoxes,
    pub moves: Vec<CustomMoveDefinition>,
    pub on_hit: Vec<CustomHitRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomMoveDefinition {
    pub id: String,
    pub name: String,
    pub action_id: u16,
    pub duration_weak: u16,
    pub duration_strong: u16,
    pub sprite_frames_weak: Vec<u16>,
    pub sprite_frames_strong: Vec<u16>,
    pub push_windows: Vec<CustomPushWindow>,
    pub vertical_delta: f64,
    pub is_kick: bool,
    pub self_invulnerable_until: Option<u16>,
    pub opponent_invulnerability: Vec<CustomInvulnerabilityRule>,
    pub effects: Vec<CustomEffectEvent>,
    pub sounds: Vec<CustomSoundEvent>,
    pub screen_shake: Option<(u16, u16)>,
    pub charge_multiplier: f64,
    pub requires_low_health_or_power: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CustomPushWindow {
    pub start_frame: u16,
    pub end_frame: u16,
    pub weak_distance: f64,
    pub strong_distance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomInvulnerabilityRule {
    pub start_frame: u16,
    pub end_frame: u16,
    pub min_hit_count: u8,
    pub max_hit_count: Option<u8>,
    pub value: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomEffectEvent {
    pub frame: u16,
    pub effect_id: u8,
    pub weak_variant: Option<char>,
    pub strong_variant: Option<char>,
    pub offset_x: i16,
    pub offset_y: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomSoundEvent {
    pub frame: u16,
    pub sound_id: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomHitRule {
    pub action_id: u16,
    pub force_strength: Option<u8>,
    pub opponent_recoil: u8,
}

impl CustomCharacterDefinition {
    pub fn from_package(package: &CustomCharacterPackage) -> Result<Self, String> {
        Self::from_manifest(&package.manifest)
    }

    pub fn from_manifest(manifest: &CharacterManifest) -> Result<Self, String> {
        let fighter = object(&manifest.fighter, "fighter")?;
        let runtime_id = required_u8(fighter, "runtime_id")?;
        if runtime_id < 16 {
            return Err("custom runtime_id must be in the range 16..=255".to_owned());
        }

        let attributes = object(required(fighter, "attributes")?, "fighter.attributes")?;
        let sprite_height = required_i32(attributes, "sprite_height")?;
        if !(1..=1024).contains(&sprite_height) {
            return Err("sprite_height must be between 1 and 1024".to_owned());
        }
        let walk_speed = required_f64(attributes, "walk_speed")?;
        if !walk_speed.is_finite() || !(0.0..=50.0).contains(&walk_speed) {
            return Err("walk_speed must be finite and between 0 and 50".to_owned());
        }

        let frame_boxes = parse_frame_boxes(required(fighter, "frame_boxes")?)?;
        let moves_value = required(fighter, "moves")?
            .as_array()
            .ok_or_else(|| "fighter.moves must be an array".to_owned())?;
        if moves_value.is_empty() || moves_value.len() > 64 {
            return Err("fighter.moves must contain 1..=64 moves".to_owned());
        }
        let mut moves = Vec::with_capacity(moves_value.len());
        for value in moves_value {
            moves.push(parse_move(value)?);
        }
        let mut move_ids = std::collections::BTreeSet::new();
        let mut action_ids = std::collections::BTreeSet::new();
        for movement in &moves {
            if !move_ids.insert(movement.id.clone()) {
                return Err(format!("duplicate custom move id {:?}", movement.id));
            }
            if !action_ids.insert(movement.action_id) {
                return Err(format!("duplicate custom action id {}", movement.action_id));
            }
        }

        let on_hit = parse_hit_rules(required(fighter, "on_hit")?)?;
        Ok(Self {
            package_id: manifest.id.clone(),
            runtime_id,
            display_name: manifest.display_name.clone(),
            sprite_height,
            walk_speed,
            frame_boxes,
            moves,
            on_hit,
        })
    }

    pub fn move_by_id(&self, id: &str) -> Option<&CustomMoveDefinition> {
        self.moves.iter().find(|movement| movement.id == id)
    }

    pub fn move_by_action(&self, action_id: u16) -> Option<&CustomMoveDefinition> {
        self.moves
            .iter()
            .find(|movement| movement.action_id == action_id)
    }

    pub fn opponent_recoil(&self, action_id: u16, force_strength: u8) -> Option<u8> {
        self.on_hit
            .iter()
            .find(|rule| {
                rule.action_id == action_id
                    && rule
                        .force_strength
                        .map_or(true, |required| required == force_strength)
            })
            .map(|rule| rule.opponent_recoil)
    }
}

fn parse_frame_boxes(value: &Value) -> Result<FighterFrameBoxes, String> {
    let object = object(value, "fighter.frame_boxes")?;
    let mut boxes = FighterFrameBoxes::default();

    let centers = required(object, "center_x")?
        .as_array()
        .ok_or_else(|| "frame_boxes.center_x must be an array".to_owned())?;
    for row in centers {
        let values = number_array::<2>(row, "frame_boxes.center_x row")?;
        let frame = u16::try_from(values[0])
            .map_err(|_| "center_x frame must be a positive Word".to_owned())?;
        let offset =
            i16::try_from(values[1]).map_err(|_| "center_x offset must fit SmallInt".to_owned())?;
        boxes.center_x.push((frame, offset));
    }

    let hurtboxes = required(object, "hurtboxes")?
        .as_array()
        .ok_or_else(|| "frame_boxes.hurtboxes must be an array".to_owned())?;
    for row in hurtboxes {
        let values = number_array::<5>(row, "frame_boxes.hurtboxes row")?;
        let bounds = [
            i16::try_from(values[1]).map_err(|_| "hurtbox x1 is outside SmallInt range")?,
            i16::try_from(values[2]).map_err(|_| "hurtbox y1 is outside SmallInt range")?,
            i16::try_from(values[3]).map_err(|_| "hurtbox x2 is outside SmallInt range")?,
            i16::try_from(values[4]).map_err(|_| "hurtbox y2 is outside SmallInt range")?,
        ];
        boxes.hurtboxes.push(HurtboxRecord {
            frame: u16::try_from(values[0])
                .map_err(|_| "hurtbox frame must be a positive Word".to_owned())?,
            bounds: bounds_from(bounds)?,
        });
    }

    let attacks = required(object, "attacks")?
        .as_array()
        .ok_or_else(|| "frame_boxes.attacks must be an array".to_owned())?;
    for row in attacks {
        let item = object(row, "frame_boxes.attacks entry")?;
        let action = required_u16(item, "action")?;
        let action_timer = required_u16(item, "frame")?;
        let strength_condition = optional_u8(item, "strength_condition")?;
        let bounds = bounds_from(required_i16_array::<4>(item, "bounds")?)?;
        let active = optional_bool(item, "active")?.unwrap_or(true);
        let assigned_strength = optional_u8(item, "assigned_strength")?.unwrap_or(0);
        let damage = required_u16(item, "damage")?;
        let guard_damage = required_u16(item, "guard_damage")?;
        boxes.attacks.push(PascalAttackDefinition {
            action: ActionId(action),
            action_timer,
            strength_condition,
            bounds,
            active,
            assigned_strength,
            damage,
            guard_damage,
        });
    }

    if boxes.hurtboxes.is_empty() {
        return Err("frame_boxes.hurtboxes must not be empty".to_owned());
    }
    Ok(boxes)
}

fn parse_move(value: &Value) -> Result<CustomMoveDefinition, String> {
    let item = object(value, "fighter.moves entry")?;
    let id = required_string(item, "id")?;
    if id.is_empty() || id.len() > 48 {
        return Err("custom move id must contain 1..=48 characters".to_owned());
    }
    let name = required_string(item, "name")?;
    let action_id = required_u16(item, "action_id")?;
    if !(25..=29).contains(&action_id) {
        return Err(format!(
            "custom move {id:?} must use a special action id from 25 through 29"
        ));
    }
    let duration_weak = required_u16(item, "duration_weak")?;
    let duration_strong = required_u16(item, "duration_strong")?;
    if duration_weak == 0 || duration_strong == 0 || duration_weak > 600 || duration_strong > 600 {
        return Err(format!("custom move {id:?} has an invalid duration"));
    }
    let sprite_frames_weak = parse_sprite_frames(required(item, "sprite_frames_weak")?)?;
    let sprite_frames_strong = parse_sprite_frames(required(item, "sprite_frames_strong")?)?;
    if sprite_frames_weak.len() < usize::from(duration_weak)
        || sprite_frames_strong.len() < usize::from(duration_strong)
    {
        return Err(format!(
            "custom move {id:?} needs one sprite frame for every weak/strong action tick"
        ));
    }

    let mut push_windows = Vec::new();
    for row in required(item, "push_windows")?
        .as_array()
        .ok_or_else(|| "move.push_windows must be an array".to_owned())?
    {
        let push = object(row, "move.push_windows entry")?;
        let window = CustomPushWindow {
            start_frame: required_u16(push, "start_frame")?,
            end_frame: required_u16(push, "end_frame")?,
            weak_distance: required_f64(push, "weak_distance")?,
            strong_distance: required_f64(push, "strong_distance")?,
        };
        if window.start_frame > window.end_frame
            || window.weak_distance.abs() > 100.0
            || window.strong_distance.abs() > 100.0
        {
            return Err(format!("custom move {id:?} has an invalid push window"));
        }
        push_windows.push(window);
    }

    let mut opponent_invulnerability = Vec::new();
    for row in required(item, "opponent_invulnerability")?
        .as_array()
        .ok_or_else(|| "move.opponent_invulnerability must be an array".to_owned())?
    {
        let rule = object(row, "move.opponent_invulnerability entry")?;
        let start_frame = required_u16(rule, "start_frame")?;
        let end_frame = optional_u16(rule, "end_frame")?.unwrap_or(start_frame);
        let min_hit_count = optional_u8(rule, "min_hit_count")?.unwrap_or(0);
        let max_hit_count = optional_u8(rule, "max_hit_count")?;
        let value = required_bool(rule, "value")?;
        if start_frame > end_frame {
            return Err(format!(
                "custom move {id:?} has an invalid invulnerability window"
            ));
        }
        opponent_invulnerability.push(CustomInvulnerabilityRule {
            start_frame,
            end_frame,
            min_hit_count,
            max_hit_count,
            value,
        });
    }

    let mut effects = Vec::new();
    for row in required(item, "effects")?
        .as_array()
        .ok_or_else(|| "move.effects must be an array".to_owned())?
    {
        let event = object(row, "move.effects entry")?;
        let offset = number_array::<2>(required(event, "offset")?, "effect offset")?;
        effects.push(CustomEffectEvent {
            frame: required_u16(event, "frame")?,
            effect_id: required_u8(event, "effect_id")?,
            weak_variant: optional_char(event, "weak_variant")?,
            strong_variant: optional_char(event, "strong_variant")?,
            offset_x: i16::try_from(offset[0])
                .map_err(|_| "effect offset x must fit SmallInt".to_owned())?,
            offset_y: i16::try_from(offset[1])
                .map_err(|_| "effect offset y must fit SmallInt".to_owned())?,
        });
    }

    let mut sounds = Vec::new();
    for row in required(item, "sounds")?
        .as_array()
        .ok_or_else(|| "move.sounds must be an array".to_owned())?
    {
        let event = object(row, "move.sounds entry")?;
        sounds.push(CustomSoundEvent {
            frame: required_u16(event, "frame")?,
            sound_id: required_u16(event, "sound_id")?,
        });
    }

    let screen_shake = optional_object(item, "screen_shake")?
        .map(|event| {
            Ok((
                required_u16(event, "frame")?,
                required_u16(event, "duration")?,
            ))
        })
        .transpose()?;

    let vertical_delta = optional_f64(item, "vertical_delta")?.unwrap_or(0.0);
    if !vertical_delta.is_finite() || vertical_delta.abs() > 100.0 {
        return Err(format!("custom move {id:?} has an invalid vertical delta"));
    }

    Ok(CustomMoveDefinition {
        id,
        name,
        action_id,
        duration_weak,
        duration_strong,
        sprite_frames_weak,
        sprite_frames_strong,
        push_windows,
        vertical_delta,
        is_kick: optional_bool(item, "is_kick")?.unwrap_or(false),
        self_invulnerable_until: optional_u16(item, "self_invulnerable_until")?,
        opponent_invulnerability,
        effects,
        sounds,
        screen_shake,
        charge_multiplier: optional_f64(item, "charge_multiplier")?.unwrap_or(0.0),
        requires_low_health_or_power: optional_bool(item, "requires_low_health_or_power")?
            .unwrap_or(false),
    })
}

fn parse_sprite_frames(value: &Value) -> Result<Vec<u16>, String> {
    let values = value
        .as_array()
        .ok_or_else(|| "sprite frame timeline must be an array".to_owned())?;
    let mut frames = Vec::with_capacity(values.len());
    for value in values {
        let frame = value
            .as_u64()
            .ok_or_else(|| "sprite frame ids must be unsigned integers".to_owned())?;
        let frame = u16::try_from(frame)
            .map_err(|_| "sprite frame id is outside the Word range".to_owned())?;
        if !(1..=255).contains(&frame) {
            return Err("custom sprite frame ids must be in the range 1..=255".to_owned());
        }
        frames.push(frame);
    }
    Ok(frames)
}

fn parse_hit_rules(value: &Value) -> Result<Vec<CustomHitRule>, String> {
    let rows = value
        .as_array()
        .ok_or_else(|| "fighter.on_hit must be an array".to_owned())?;
    let mut rules = Vec::with_capacity(rows.len());
    for row in rows {
        let item = object(row, "fighter.on_hit entry")?;
        let action_id = required_u16(item, "action_id")?;
        let force_strength = optional_u8(item, "force_strength")?;
        let opponent_recoil = required_u8(item, "opponent_recoil")?;
        rules.push(CustomHitRule {
            action_id,
            force_strength,
            opponent_recoil,
        });
    }
    Ok(rules)
}

fn object<'a>(
    value: &'a Value,
    context: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{context} must be an object"))
}

fn required<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a Value, String> {
    object
        .get(key)
        .ok_or_else(|| format!("missing required field {key:?}"))
}

fn required_string(object: &serde_json::Map<String, Value>, key: &str) -> Result<String, String> {
    required(object, key)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{key} must be a string"))
}

fn required_u8(object: &serde_json::Map<String, Value>, key: &str) -> Result<u8, String> {
    required(object, key)?
        .as_u64()
        .and_then(|value| u8::try_from(value).ok())
        .ok_or_else(|| format!("{key} must be a Byte"))
}

fn required_u16(object: &serde_json::Map<String, Value>, key: &str) -> Result<u16, String> {
    required(object, key)?
        .as_u64()
        .and_then(|value| u16::try_from(value).ok())
        .ok_or_else(|| format!("{key} must be a Word"))
}

fn required_i32(object: &serde_json::Map<String, Value>, key: &str) -> Result<i32, String> {
    required(object, key)?
        .as_i64()
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| format!("{key} must fit LongInt"))
}

fn required_f64(object: &serde_json::Map<String, Value>, key: &str) -> Result<f64, String> {
    required(object, key)?
        .as_f64()
        .ok_or_else(|| format!("{key} must be numeric"))
}

fn optional_f64(object: &serde_json::Map<String, Value>, key: &str) -> Result<Option<f64>, String> {
    object
        .get(key)
        .map(|value| {
            value
                .as_f64()
                .ok_or_else(|| format!("{key} must be numeric"))
        })
        .transpose()
}

fn optional_u8(object: &serde_json::Map<String, Value>, key: &str) -> Result<Option<u8>, String> {
    object
        .get(key)
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| u8::try_from(value).ok())
                .ok_or_else(|| format!("{key} must be a Byte"))
        })
        .transpose()
}

fn optional_u16(object: &serde_json::Map<String, Value>, key: &str) -> Result<Option<u16>, String> {
    object
        .get(key)
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| u16::try_from(value).ok())
                .ok_or_else(|| format!("{key} must be a Word"))
        })
        .transpose()
}

fn required_bool(object: &serde_json::Map<String, Value>, key: &str) -> Result<bool, String> {
    required(object, key)?
        .as_bool()
        .ok_or_else(|| format!("{key} must be a boolean"))
}

fn optional_bool(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<bool>, String> {
    object
        .get(key)
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| format!("{key} must be a boolean"))
        })
        .transpose()
}

fn optional_object<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<&'a serde_json::Map<String, Value>>, String> {
    object
        .get(key)
        .map(|value| object_value(value, key))
        .transpose()
}

fn object_value<'a>(
    value: &'a Value,
    context: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{context} must be an object"))
}

fn optional_char(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<char>, String> {
    object
        .get(key)
        .map(|value| {
            let value = value
                .as_str()
                .ok_or_else(|| format!("{key} must be a one-character string"))?;
            let mut chars = value.chars();
            let result = chars
                .next()
                .ok_or_else(|| format!("{key} must not be empty"))?;
            if chars.next().is_some() {
                return Err(format!("{key} must contain one character"));
            }
            Ok(result)
        })
        .transpose()
}

fn required_i16_array<const N: usize>(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<[i16; N], String> {
    number_array::<N>(required(object, key)?, key).and_then(|values| {
        values
            .into_iter()
            .map(|value| {
                i16::try_from(value).map_err(|_| format!("{key} value is outside SmallInt range"))
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| format!("{key} must contain exactly {N} values"))
    })
}

fn number_array<const N: usize>(value: &Value, context: &str) -> Result<[i64; N], String> {
    let values = value
        .as_array()
        .ok_or_else(|| format!("{context} must be an array"))?;
    if values.len() != N {
        return Err(format!("{context} must contain exactly {N} values"));
    }
    let parsed = values
        .iter()
        .map(|value| {
            value
                .as_i64()
                .ok_or_else(|| format!("{context} values must be integers"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    parsed
        .try_into()
        .map_err(|_| format!("{context} must contain exactly {N} values"))
}

fn bounds_from(values: [i16; 4]) -> Result<RawBounds, String> {
    let bounds = RawBounds {
        x1: values[0],
        y1: values[1],
        x2: values[2],
        y2: values[3],
    };
    if bounds.x1 > bounds.x2 || bounds.y1 > bounds.y2 {
        return Err("custom hitbox/hurtbox bounds must be ordered".to_owned());
    }
    Ok(bounds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_custom_runtime_ids_inside_the_builtin_roster() {
        let manifest = CharacterManifest {
            schema_version: 1,
            id: "chapolin".to_owned(),
            display_name: "Chapolin Colorado".to_owned(),
            version: "1.0.0".to_owned(),
            author: None,
            fighter: serde_json::json!({
                "runtime_id": 15,
                "attributes": {"sprite_height": 199, "walk_speed": 9.0},
                "frame_boxes": {
                    "center_x": [[1, 30]],
                    "hurtboxes": [[1, -12, 65, 19, 200]],
                    "attacks": []
                },
                "moves": [],
                "on_hit": []
            }),
            assets: Default::default(),
        };
        assert!(CustomCharacterDefinition::from_manifest(&manifest).is_err());
    }

    #[test]
    fn parses_the_complete_chapolin_c16_package_definition() {
        use crate::custom_character::{CustomCharacterPackage, PackageFile};

        let package = CustomCharacterPackage::from_files([PackageFile {
            path: "character.json".to_owned(),
            bytes: include_bytes!("../examples/custom-characters/chapolin/character.json").to_vec(),
        }])
        .unwrap();
        let definition = CustomCharacterDefinition::from_package(&package).unwrap();

        assert_eq!(definition.runtime_id, 16);
        assert_eq!(definition.sprite_height, 199);
        assert_eq!(definition.walk_speed, 9.0);
        assert_eq!(definition.frame_boxes.center_x.len(), 50);
        assert_eq!(definition.frame_boxes.hurtboxes.len(), 50);
        assert_eq!(definition.frame_boxes.attacks.len(), 20);
        assert_eq!(definition.moves.len(), 5);

        let hammer = definition.move_by_id("special_2").unwrap();
        assert_eq!(hammer.action_id, 26);
        assert_eq!(hammer.duration_weak, 16);
        assert_eq!(hammer.duration_strong, 24);
        assert_eq!(definition.opponent_recoil(26, 2), Some(20));
        assert_eq!(definition.opponent_recoil(28, 1), Some(15));
        assert_eq!(definition.opponent_recoil(26, 1), None);
    }

    #[test]
    fn custom_package_frames_and_bounds_are_validated() {
        use crate::custom_character::{CustomCharacterPackage, PackageFile};

        let package = CustomCharacterPackage::from_files([PackageFile {
            path: "character.json".to_owned(),
            bytes: include_bytes!("../examples/custom-characters/chapolin/character.json").to_vec(),
        }])
        .unwrap();
        let definition = CustomCharacterDefinition::from_package(&package).unwrap();

        assert_eq!(definition.frame_boxes.hurtboxes[0].frame, 1);
        assert_eq!(definition.frame_boxes.hurtboxes[0].bounds.x1, -12);
        assert_eq!(definition.frame_boxes.hurtboxes[0].bounds.y2, 200);
        assert_eq!(
            definition.frame_boxes.attacks[12].strength_condition,
            Some(1)
        );
        assert_eq!(
            definition.frame_boxes.attacks[13].strength_condition,
            Some(2)
        );
    }
}
