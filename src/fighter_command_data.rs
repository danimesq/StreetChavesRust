//! Compile-time snapshot of the character [Botoes] and [Sequencias] rules.
//! Values are extracted verbatim from the pinned Pascal INI files.
use std::collections::BTreeMap;

use crate::command_recognition::{
    parse_condition, AttackButton, BoxCondition, CommandRule, RuleParseError, SequenceRule,
};

/// Pinned combined source containing the command sections for all 15 roster fighters.
pub const CHARACTER_COMMAND_DATA: &str = include_str!("../data/fighter-command-rules.ini");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IgnoredCommandRule {
    pub section: String,
    pub index: usize,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterCommandRules {
    pub character_id: u8,
    pub sequences: Vec<SequenceRule>,
    pub buttons: Vec<CommandRule>,
    /// Pascal ignores a complete rule when one of its terms is unknown.
    pub ignored_rules: Vec<IgnoredCommandRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDataError {
    pub character_id: u8,
    pub detail: String,
}

impl std::fmt::Display for CommandDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "character C{:02}: {}", self.character_id, self.detail)
    }
}
impl std::error::Error for CommandDataError {}

fn translate_condition(token: &str) -> Result<BoxCondition, RuleParseError> {
    let trimmed = token.trim();
    let (negated, atom) = trimmed
        .strip_prefix('!')
        .map_or((false, trimmed), |value| (true, value.trim()));
    let shorthand = match atom {
        "Baixo" => "B".to_owned(),
        "Frente" => "F".to_owned(),
        "Tras" => "T".to_owned(),
        "Cima" => "C".to_owned(),
        "NoChao" => "H".to_owned(),
        "NoAr" => "A".to_owned(),
        _ if atom.starts_with("TempoBaixo>=") => {
            format!("b{}", &atom["TempoBaixo>=".len()..])
        }
        _ if atom.starts_with("TempoTras>=") => {
            format!("t{}", &atom["TempoTras>=".len()..])
        }
        _ if atom.starts_with("Passo=") => format!("P{}", &atom["Passo=".len()..]),
        _ if atom.starts_with("CargaSoco>=") => {
            format!("s{}", &atom["CargaSoco>=".len()..])
        }
        _ if atom.starts_with("CargaChute>=") => {
            format!("k{}", &atom["CargaChute>=".len()..])
        }
        _ => atom.to_owned(),
    };
    let parsed = parse_condition(&shorthand)?;
    Ok(BoxCondition { negated, ..parsed })
}

fn parse_conditions(raw: &str) -> Result<Vec<BoxCondition>, String> {
    raw.split(',')
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(|term| translate_condition(term).map_err(|error| {
            format!("unknown or invalid condition {:?}: {}", term, error.token)
        }))
        .collect()
}

fn attack_button(raw: &str) -> Option<AttackButton> {
    match raw.trim() {
        "SocoFraco" => Some(AttackButton::WeakPunch),
        "ChuteFraco" => Some(AttackButton::WeakKick),
        "SocoForte" => Some(AttackButton::StrongPunch),
        "ChuteForte" => Some(AttackButton::StrongKick),
        _ => None,
    }
}

/// The exact NomesComando order from ChavesTipos.pas (Pascal command IDs are 1-based).
fn command_code(raw: &str) -> Option<u8> {
    const NAMES: [&str; 26] = [
        "Esquerda", "Direita", "SocoFraco", "ChuteFraco", "SocoForte", "ChuteForte",
        "Agachar", "ApanharAlto", "ApanharBaixo", "Derrubar", "BaterNoChao", "Defender",
        "DefenderBaixo", "Vitoria", "Pose", "Esp1Fraco", "Esp1Forte", "Esp2Fraco",
        "Esp2Forte", "Esp3Fraco", "Esp3Forte", "Esp4Fraco", "Esp4Forte", "EspFinal",
        "EspExtra1", "EspExtra2",
    ];
    NAMES
        .iter()
        .position(|name| *name == raw.trim())
        .map(|index| (index + 1) as u8)
}

/// Parse only the two command sections, retaining Pascal's numbered-row order.
/// Missing numeric keys terminate a section, as in LerRegrasComando.
pub fn parse_character_command_ini(
    character_id: u8,
    source: &str,
) -> Result<CharacterCommandRules, CommandDataError> {
    if !(1..=15).contains(&character_id) {
        return Err(CommandDataError {
            character_id,
            detail: "fighter ID is outside the pinned 15-character roster".into(),
        });
    }
    let prefix = format!("C{:02}", character_id);
    let mut section = String::new();
    let mut sections: BTreeMap<String, BTreeMap<usize, String>> = BTreeMap::new();

    for (line_index, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_owned();
            continue;
        }
        if !section.starts_with(&prefix) {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if let Ok(index) = key.trim().parse::<usize>() {
            sections
                .entry(section.clone())
                .or_default()
                .insert(index, value.trim().to_owned());
        } else {
            return Err(CommandDataError {
                character_id,
                detail: format!("non-numeric rule key on data line {}", line_index + 1),
            });
        }
    }

    let mut output = CharacterCommandRules {
        character_id,
        sequences: Vec::new(),
        buttons: Vec::new(),
        ignored_rules: Vec::new(),
    };

    let sequence_section = format!("{prefix}.Sequencias");
    if let Some(rows) = sections.get(&sequence_section) {
        let mut index = 1;
        while let Some(raw) = rows.get(&index) {
            let parsed = (|| {
                let (conditions, transition) = raw
                    .split_once(':')
                    .ok_or_else(|| "missing ':' before sequence transition".to_owned())?;
                let (from, to) = transition
                    .trim()
                    .split_once('>')
                    .ok_or_else(|| "missing '>' in sequence transition".to_owned())?;
                let from_state = from.trim().parse::<u8>()
                    .map_err(|_| "invalid sequence source state".to_owned())?;
                let to_state = to.trim().parse::<u8>()
                    .map_err(|_| "invalid sequence target state".to_owned())?;
                Ok::<_, String>(SequenceRule {
                    from_state,
                    to_state,
                    conditions: parse_conditions(conditions)?,
                })
            })();
            match parsed {
                Ok(rule) => output.sequences.push(rule),
                Err(reason) => output.ignored_rules.push(IgnoredCommandRule {
                    section: "Sequencias".into(),
                    index,
                    reason,
                }),
            }
            index += 1;
        }
    }

    let button_section = format!("{prefix}.Botoes");
    if let Some(rows) = sections.get(&button_section) {
        let mut index = 1;
        while let Some(raw) = rows.get(&index) {
            let parsed = (|| {
                let (input, command) = raw
                    .split_once(':')
                    .ok_or_else(|| "missing ':' before Pascal command name".to_owned())?;
                let mut parts = input.split(',');
                let button_name = parts.next().unwrap_or_default().trim();
                let button = attack_button(button_name)
                    .ok_or_else(|| format!("unknown attack button {:?}", button_name))?;
                let conditions_raw = parts.collect::<Vec<_>>().join(",");
                let command = command_code(command)
                    .ok_or_else(|| format!("unknown Pascal command {:?}", command.trim()))?;
                Ok::<_, String>(CommandRule {
                    attack_button: button,
                    command_code: command,
                    conditions: parse_conditions(&conditions_raw)?,
                })
            })();
            match parsed {
                Ok(rule) => output.buttons.push(rule),
                Err(reason) => output.ignored_rules.push(IgnoredCommandRule {
                    section: "Botoes".into(),
                    index,
                    reason,
                }),
            }
            index += 1;
        }
    }

    Ok(output)
}

/// Load the pinned command rules for one character ID (C01..C15).
pub fn embedded_character_command_rules(
    character_id: u8,
) -> Result<CharacterCommandRules, CommandDataError> {
    parse_character_command_ini(character_id, CHARACTER_COMMAND_DATA)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_command_rules_load_for_the_entire_roster_in_ini_order() {
        for character_id in 1..=15 {
            let rules = embedded_character_command_rules(character_id).unwrap();
            assert!(!rules.sequences.is_empty(), "C{character_id:02} sequences");
            assert!(!rules.buttons.is_empty(), "C{character_id:02} buttons");
            assert!(rules.ignored_rules.is_empty(), "C{character_id:02}: {:?}", rules.ignored_rules);
        }
    }

    #[test]
    fn parser_preserves_charge_air_and_extra_command_terms() {
        let c05 = embedded_character_command_rules(5).unwrap();
        let c05_rule = c05.buttons.iter().find(|rule| {
            rule.attack_button == AttackButton::WeakKick && rule.command_code == 22
        }).unwrap();
        assert_eq!(c05_rule.conditions, vec![translate_condition("CargaChute>=6").unwrap()]);

        let c08 = embedded_character_command_rules(8).unwrap();
        assert!(c08.buttons.iter().any(|rule| {
            rule.command_code == 18
                && rule.conditions.contains(&translate_condition("NoAr").unwrap())
        }));

        let c13 = embedded_character_command_rules(13).unwrap();
        assert!(c13.buttons.iter().any(|rule| rule.command_code == 25));
        assert!(c13.buttons.iter().any(|rule| rule.command_code == 26));
    }

    #[test]
    fn unknown_terms_skip_the_complete_row_without_reordering_later_rules() {
        let source = "[C01.Botoes]\n1=SocoFraco, CondicaoInexistente: Esp1Fraco\n2=SocoForte: Esp1Forte\n";
        let rules = parse_character_command_ini(1, source).unwrap();
        assert_eq!(rules.buttons.len(), 1);
        assert_eq!(rules.buttons[0].command_code, 17);
        assert_eq!(rules.ignored_rules.len(), 1);
        assert_eq!(rules.ignored_rules[0].index, 1);
    }
}
