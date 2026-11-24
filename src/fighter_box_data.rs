//! Source-ordered parser for the Pascal fighter frame and collision tables.
//!
//! The embedded input is extracted from C01.ini..C15.ini at the pinned
//! OpenStreetChaves revision. This module preserves attack-row order because
//! Pascal's AplicarCaixas evaluates force conditions as it walks the INI rows.

use crate::{HurtboxRecord, PascalAttackDefinition, RawBounds, ActionId};

const SOURCE_TABLES: &str = include_str!("../data/fighter-boxes.ini");

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FighterFrameBoxes {
    /// Sprite-centre offsets keyed by the player-one sprite frame.
    pub center_x: Vec<(u16, i16)>,
    /// Defined body rows. A missing frame means retain the current body box.
    pub hurtboxes: Vec<HurtboxRecord>,
    /// Attack rows in their original per-file order. Do not sort this vector.
    pub attacks: Vec<PascalAttackDefinition>,
}

/// Load and validate the source tables for one fighter (IDs 1 through 15).
pub fn load_fighter_frame_boxes(character_id: u8) -> Result<FighterFrameBoxes, String> {
    if !(1..=15).contains(&character_id) {
        return Err(format!("fighter frame tables do not contain C{character_id:02}"));
    }

    let prefix = format!("C{character_id:02}");
    let mut output = FighterFrameBoxes::default();
    let mut section = String::new();

    for (line_number, raw_line) in SOURCE_TABLES.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].to_owned();
            continue;
        }
        if !section.starts_with(&prefix) {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(format!(
                "fighter table line {} has no '=': {line}",
                line_number + 1
            ));
        };
        let key = key.trim();
        let value = value.trim();

        if section == format!("{prefix}.CentroX") {
            let frame = parse_number::<u16>(key, &section, line_number)?;
            let offset = parse_number::<i16>(value, &section, line_number)?;
            output.center_x.push((frame, offset));
        } else if section == format!("{prefix}.Corpo") {
            let frame = parse_number::<u16>(key, &section, line_number)?;
            let coordinates = parse_csv::<4>(value, &section, line_number)?;
            output.hurtboxes.push(HurtboxRecord {
                frame,
                bounds: RawBounds {
                    x1: coordinates[0],
                    y1: coordinates[1],
                    x2: coordinates[2],
                    y2: coordinates[3],
                },
            });
        } else if section == format!("{prefix}.Ataque") {
            let (action, timer, strength_condition) =
                parse_attack_key(key, &section, line_number)?;
            let values = parse_csv::<8>(value, &section, line_number)?;
            output.attacks.push(PascalAttackDefinition {
                action: ActionId(action),
                action_timer: timer,
                strength_condition,
                bounds: RawBounds {
                    x1: values[0],
                    y1: values[1],
                    x2: values[2],
                    y2: values[3],
                },
                active: values[4] != 0,
                assigned_strength: to_u8(values[5], &section, line_number)?,
                damage: to_u16(values[6], &section, line_number)?,
                guard_damage: to_u16(values[7], &section, line_number)?,
            });
        }
    }

    if output.center_x.is_empty() && output.hurtboxes.is_empty() && output.attacks.is_empty() {
        return Err(format!("no frame/box rows found for {prefix}"));
    }

    Ok(output)
}

fn parse_attack_key(
    key: &str,
    section: &str,
    line_number: usize,
) -> Result<(u16, u16, Option<u8>), String> {
    let mut parts = key.split('.');
    let action = parse_number::<u16>(
        parts.next().unwrap_or_default(),
        section,
        line_number,
    )?;
    let timer = parse_number::<u16>(
        parts.next().unwrap_or_default(),
        section,
        line_number,
    )?;
    let strength = match parts.next() {
        None => None,
        Some(suffix) if suffix.len() >= 2 && suffix[..1].eq_ignore_ascii_case("f") => {
            Some(parse_number::<u8>(&suffix[1..], section, line_number)?)
        }
        Some(suffix) => {
            return Err(format!(
                "{section} line {} has invalid attack suffix '{suffix}'",
                line_number + 1
            ))
        }
    };
    if parts.next().is_some() {
        return Err(format!(
            "{section} line {} has too many attack-key components",
            line_number + 1
        ));
    }
    Ok((action, timer, strength))
}

fn parse_csv<const N: usize>(
    value: &str,
    section: &str,
    line_number: usize,
) -> Result<[i16; N], String> {
    let parts: Vec<_> = value.split(',').map(str::trim).collect();
    if parts.len() != N {
        return Err(format!(
            "{section} line {} expected {N} comma-separated values, found {}",
            line_number + 1,
            parts.len()
        ));
    }
    let mut output = [0i16; N];
    for (index, part) in parts.iter().enumerate() {
        output[index] = parse_number::<i16>(part, section, line_number)?;
    }
    Ok(output)
}

fn parse_number<T>(value: &str, section: &str, line_number: usize) -> Result<T, String>
where
    T: std::str::FromStr,
{
    value.parse::<T>().map_err(|_| {
        format!(
            "{section} line {} contains invalid number '{value}'",
            line_number + 1
        )
    })
}

fn to_u8(value: i16, section: &str, line_number: usize) -> Result<u8, String> {
    u8::try_from(value).map_err(|_| {
        format!(
            "{section} line {} value {value} is outside Byte range",
            line_number + 1
        )
    })
}

fn to_u16(value: i16, section: &str, line_number: usize) -> Result<u16, String> {
    u16::try_from(value).map_err(|_| {
        format!(
            "{section} line {} value {value} is outside Word range",
            line_number + 1
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_fifteen_fighters_and_preserves_source_row_counts() {
        let mut hurtbox_count = 0;
        let mut attack_count = 0;
        for character_id in 1..=15 {
            let tables = load_fighter_frame_boxes(character_id).unwrap();
            assert!(!tables.center_x.is_empty(), "C{character_id:02}");
            assert!(!tables.hurtboxes.is_empty(), "C{character_id:02}");
            assert!(!tables.attacks.is_empty(), "C{character_id:02}");
            hurtbox_count += tables.hurtboxes.len();
            attack_count += tables.attacks.len();
        }
        assert_eq!(hurtbox_count, 305);
        assert_eq!(attack_count, 379);
    }

    #[test]
    fn retains_force_suffix_and_duplicate_selector_rows_in_source_order() {
        let c01 = load_fighter_frame_boxes(1).unwrap();
        let special = c01.attacks.iter().find(|row| {
            row.action == ActionId(26)
                && row.action_timer == 6
                && row.strength_condition == Some(1)
        });
        assert!(special.is_some());

        let c11 = load_fighter_frame_boxes(11).unwrap();
        let duplicate_rows: Vec<_> = c11
            .attacks
            .iter()
            .filter(|row| row.action == ActionId(27) && row.action_timer == 1)
            .collect();
        assert_eq!(duplicate_rows.len(), 2);
        assert_eq!(duplicate_rows[0].strength_condition, Some(1));
        assert_eq!(duplicate_rows[1].strength_condition, Some(2));
    }

    #[test]
    fn rejects_missing_character_tables() {
        assert!(load_fighter_frame_boxes(0).is_err());
        assert!(load_fighter_frame_boxes(16).is_err());
    }
}
