//! Conservative join between concrete Pascal schema-v2 traces and the
//! normalizer's static symbolic RNG/action-tick index.
//!
//! Direct Pascal .inc RNG call sites expose source columns; ROT expression
//! columns are still absent. This module preserves ambiguity instead of
//! choosing a static AST site based on event order or a guessed column.

use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, Read};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceJoinError {
    Io(String),
    Json { line: usize, message: String },
    InvalidRecord { line: usize, message: String },
    InvalidSymbolicIndex(String),
}

impl fmt::Display for TraceJoinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(f, "trace I/O error: {message}"),
            Self::Json { line, message } => {
                write!(f, "invalid JSON on line {line}: {message}")
            }
            Self::InvalidRecord { line, message } => {
                write!(f, "invalid trace record on line {line}: {message}")
            }
            Self::InvalidSymbolicIndex(message) => {
                write!(f, "invalid symbolic index: {message}")
            }
        }
    }
}

impl std::error::Error for TraceJoinError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDraw {
    pub api_call_index: u64,
    pub tick_index: Option<u64>,
    pub script_execution_id: Option<u64>,
    pub parent_script_execution_id: Option<u64>,
    pub script_draw_index: Option<u64>,
    pub fighter_index: Option<u8>,
    pub character_id: Option<u8>,
    pub action_id: Option<u32>,
    pub action_frame: Option<u32>,
    pub command_code: Option<i32>,
    pub source_kind: String,
    pub source_origin: Option<String>,
    pub source_line: Option<u32>,
    pub source_column: Option<u32>,
    pub width: String,
    pub range: i64,
    pub result: i64,
    pub first_mt_word_index: Option<u64>,
    pub raw_mt_words: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeControl {
    pub control_event_index: u64,
    pub tick_index: Option<u64>,
    pub script_execution_id: Option<u64>,
    pub source_line: u32,
    pub control_type: String,
    pub branch_taken: Option<bool>,
    pub iteration_index: Option<u64>,
    pub iteration_value: Option<i64>,
    pub iteration_count: Option<u64>,
    pub arm_index: Option<u64>,
    pub selector_value: Option<i64>,
    pub loop_terminates: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCapture {
    pub session_id: String,
    pub schema_version: u64,
    pub draws: Vec<RuntimeDraw>,
    pub controls: Vec<RuntimeControl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolicRngSite {
    pub fighter_id: u8,
    pub section: String,
    pub group_index: u32,
    pub event_sequence: u64,
    pub character: String,
    pub source_line: u32,
    pub source_column: u32,
    pub rng_site: String,
    pub range_expression: String,
    pub random_width: String,
    pub symbolic_action_guards: Vec<String>,
    pub symbolic_tick_guards: Vec<String>,
    pub guards: Vec<String>,
    pub loop_context: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteMatchStatus {
    Unique,
    Ambiguous,
    Unmatched,
    NonRotSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawReconciliation {
    pub draw: RuntimeDraw,
    pub status: SiteMatchStatus,
    pub candidates: Vec<SymbolicRngSite>,
    /// Decisions observed in the same concrete script invocation. These are
    /// execution context, not a claim that a guard string was independently
    /// evaluated or mapped to a particular AST column.
    pub invocation_controls: Vec<RuntimeControl>,
}

fn invalid_record(line: usize, message: impl Into<String>) -> TraceJoinError {
    TraceJoinError::InvalidRecord {
        line,
        message: message.into(),
    }
}

fn required_string(record: &Value, key: &str, line: usize) -> Result<String, TraceJoinError> {
    record
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| invalid_record(line, format!("{key} must be a string")))
}

fn required_u64(record: &Value, key: &str, line: usize) -> Result<u64, TraceJoinError> {
    record
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid_record(line, format!("{key} must be a non-negative integer")))
}

fn required_i64(record: &Value, key: &str, line: usize) -> Result<i64, TraceJoinError> {
    record
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| invalid_record(line, format!("{key} must be an integer")))
}

fn optional_u64(record: &Value, key: &str) -> Option<u64> {
    record.get(key).and_then(Value::as_u64)
}

fn optional_u32(record: &Value, key: &str) -> Option<u32> {
    record
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

fn optional_u8(record: &Value, key: &str) -> Option<u8> {
    record
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
}

fn optional_i32(record: &Value, key: &str) -> Option<i32> {
    record
        .get(key)
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
}

fn optional_i64(record: &Value, key: &str) -> Option<i64> {
    record.get(key).and_then(Value::as_i64)
}

fn optional_bool(record: &Value, key: &str) -> Option<bool> {
    record.get(key).and_then(Value::as_bool)
}

fn parse_draw(record: &Value, line: usize) -> Result<RuntimeDraw, TraceJoinError> {
    let source_line = optional_u32(record, "source_line");
    let first_mt_word_index = optional_u64(record, "first_mt_word_index");

    Ok(RuntimeDraw {
        api_call_index: required_u64(record, "api_call_index", line)?,
        tick_index: optional_u64(record, "tick_index"),
        script_execution_id: optional_u64(record, "script_execution_id"),
        parent_script_execution_id: optional_u64(record, "parent_script_execution_id"),
        script_draw_index: optional_u64(record, "script_draw_index"),
        fighter_index: optional_u8(record, "fighter_index"),
        character_id: optional_u8(record, "character_id"),
        action_id: optional_u32(record, "action_id"),
        action_frame: optional_u32(record, "action_frame"),
        command_code: optional_i32(record, "command_code"),
        source_kind: required_string(record, "source_kind", line)?,
        source_origin: record
            .get("source_origin")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        source_line,
        width: required_string(record, "width", line)?,
        range: required_i64(record, "range", line)?,
        result: required_i64(record, "result", line)?,
        first_mt_word_index,
        raw_mt_words: required_u64(record, "raw_mt_words", line)?,
    })
}

fn parse_control(record: &Value, line: usize) -> Result<RuntimeControl, TraceJoinError> {
    Ok(RuntimeControl {
        control_event_index: required_u64(record, "control_event_index", line)?,
        tick_index: optional_u64(record, "tick_index"),
        script_execution_id: optional_u64(record, "script_execution_id"),
        source_line: optional_u32(record, "source_line").ok_or_else(|| {
            invalid_record(line, "control source_line must be a non-negative integer")
        })?,
        control_type: required_string(record, "control_type", line)?,
        branch_taken: optional_bool(record, "branch_taken"),
        iteration_index: optional_u64(record, "iteration_index"),
        iteration_value: optional_i64(record, "iteration_value"),
        iteration_count: optional_u64(record, "iteration_count"),
        arm_index: optional_u64(record, "arm_index"),
        selector_value: optional_i64(record, "selector_value"),
        loop_terminates: optional_bool(record, "loop_terminates"),
    })
}

/// Read a schema-v2 JSONL file. Session/tick/script records are ignored here;
/// draws and interpreter control outcomes are retained in source order.
pub fn load_runtime_capture<R: BufRead>(reader: R) -> Result<RuntimeCapture, TraceJoinError> {
    let mut session_id = None;
    let mut schema_version = None;
    let mut draws = Vec::new();
    let mut controls = Vec::new();

    for (index, raw_line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let raw_line = raw_line.map_err(|error| TraceJoinError::Io(error.to_string()))?;
        if raw_line.trim().is_empty() {
            return Err(invalid_record(
                line_number,
                "blank JSONL lines are not allowed",
            ));
        }
        let record: Value =
            serde_json::from_str(&raw_line).map_err(|error| TraceJoinError::Json {
                line: line_number,
                message: error.to_string(),
            })?;
        let record_type = required_string(&record, "record_type", line_number)?;

        match record_type.as_str() {
            "session" => {
                if session_id.is_some() || line_number != 1 {
                    return Err(invalid_record(
                        line_number,
                        "exactly one session record must appear first",
                    ));
                }
                schema_version = Some(required_u64(&record, "schema_version", line_number)?);
                session_id = Some(required_string(&record, "session_id", line_number)?);
            }
            "draw" => draws.push(parse_draw(&record, line_number)?),
            "control" => controls.push(parse_control(&record, line_number)?),
            _ => {}
        }
    }

    let session_id = session_id.ok_or_else(|| {
        TraceJoinError::InvalidSymbolicIndex("runtime trace has no session header".to_owned())
    })?;
    let schema_version = schema_version.unwrap_or_default();
    if schema_version != 2 {
        return Err(TraceJoinError::InvalidSymbolicIndex(format!(
            "expected runtime schema version 2, found {schema_version}"
        )));
    }

    Ok(RuntimeCapture {
        session_id,
        schema_version,
        draws,
        controls,
    })
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

/// Load RNG sites together with the exact symbolic group and guard context
/// emitted by tools/rot-normalize.py.
pub fn load_symbolic_rng_sites<R: Read>(reader: R) -> Result<Vec<SymbolicRngSite>, TraceJoinError> {
    let root: Value = serde_json::from_reader(reader)
        .map_err(|error| TraceJoinError::InvalidSymbolicIndex(error.to_string()))?;
    if root.get("schema").and_then(Value::as_str)
        != Some("street-chaves-symbolic-action-tick-index/v1")
    {
        return Err(TraceJoinError::InvalidSymbolicIndex(
            "unexpected symbolic action/tick index schema".to_owned(),
        ));
    }

    let fighters = root
        .get("fighters")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TraceJoinError::InvalidSymbolicIndex("fighters must be an array".to_owned())
        })?;
    let mut sites = Vec::new();

    for fighter in fighters {
        let fighter_id = fighter
            .get("fighter_id")
            .and_then(Value::as_u64)
            .and_then(|value| u8::try_from(value).ok())
            .ok_or_else(|| {
                TraceJoinError::InvalidSymbolicIndex("fighter_id must fit in u8".to_owned())
            })?;
        let sections = fighter
            .get("sections")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                TraceJoinError::InvalidSymbolicIndex(
                    "fighter sections must be an object".to_owned(),
                )
            })?;

        for (section_name, section) in sections {
            let groups = section
                .get("groups")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    TraceJoinError::InvalidSymbolicIndex(format!(
                        "groups must be an array for fighter {fighter_id} section {section_name}"
                    ))
                })?;

            for group in groups {
                let group_index = group
                    .get("group_index")
                    .and_then(Value::as_u64)
                    .and_then(|value| u32::try_from(value).ok())
                    .ok_or_else(|| {
                        TraceJoinError::InvalidSymbolicIndex(
                            "group_index must fit in u32".to_owned(),
                        )
                    })?;
                let symbolic_action_guards = string_array(group.get("symbolic_action_guards"));
                let symbolic_tick_guards = string_array(group.get("symbolic_tick_guards"));
                let events = group
                    .get("events")
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        TraceJoinError::InvalidSymbolicIndex(
                            "symbolic group events must be an array".to_owned(),
                        )
                    })?;

                for event in events {
                    if event.get("kind").and_then(Value::as_str) != Some("rng_draw") {
                        continue;
                    }
                    let event_sequence =
                        event
                            .get("sequence")
                            .and_then(Value::as_u64)
                            .ok_or_else(|| {
                                TraceJoinError::InvalidSymbolicIndex(
                                    "RNG event sequence must be an integer".to_owned(),
                                )
                            })?;
                    let source_line = event
                        .get("source_line")
                        .and_then(Value::as_u64)
                        .and_then(|value| u32::try_from(value).ok())
                        .ok_or_else(|| {
                            TraceJoinError::InvalidSymbolicIndex(
                                "RNG event source_line must fit in u32".to_owned(),
                            )
                        })?;
                    let source_column = event
                        .get("source_column")
                        .and_then(Value::as_u64)
                        .and_then(|value| u32::try_from(value).ok())
                        .ok_or_else(|| {
                            TraceJoinError::InvalidSymbolicIndex(
                                "RNG event source_column must fit in u32".to_owned(),
                            )
                        })?;

                    sites.push(SymbolicRngSite {
                        fighter_id,
                        section: section_name.to_ascii_lowercase(),
                        group_index,
                        event_sequence,
                        character: event
                            .get("character")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                        source_line,
                        source_column,
                        rng_site: event
                            .get("rng_site")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                        range_expression: event
                            .get("range_expression")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                        random_width: event
                            .get("random_width")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                        symbolic_action_guards: symbolic_action_guards.clone(),
                        symbolic_tick_guards: symbolic_tick_guards.clone(),
                        guards: string_array(event.get("guards")),
                        loop_context: string_array(event.get("loop_context")),
                    });
                }
            }
        }
    }

    Ok(sites)
}

fn parse_origin(origin: &str) -> Option<(String, String)> {
    let (file_name, section_with_bracket) = origin.rsplit_once(" [")?;
    let section = section_with_bracket.strip_suffix(']')?;
    let file_name = file_name.rsplit('/').next()?;
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(stem, _)| stem);
    let character = if stem.eq_ignore_ascii_case("Comum") {
        "Common"
    } else {
        stem
    };
    Some((character.to_owned(), section.to_ascii_lowercase()))
}

fn expected_static_width(runtime_width: &str) -> Option<&'static str> {
    match runtime_width {
        "LongInt" => Some("int32"),
        "Int64" => Some("int64"),
        _ => None,
    }
}

fn candidate_matches(draw: &RuntimeDraw, site: &SymbolicRngSite) -> bool {
    let Some(origin) = draw.source_origin.as_deref() else {
        return false;
    };
    let Some((character, section)) = parse_origin(origin) else {
        return false;
    };
    let Some(character_id) = draw.character_id else {
        return false;
    };
    let Some(line) = draw.source_line else {
        return false;
    };
    let Some(width) = expected_static_width(&draw.width) else {
        return false;
    };

    if site.fighter_id != character_id
        || !site.character.eq_ignore_ascii_case(&character)
        || site.section != section
        || site.source_line != line
        || site.random_width != width
    {
        return false;
    }

    // Numeric static ranges can be checked exactly. Expressions such as
    // Round(30 * Fator) stay unconstrained here because the runtime value is
    // context-dependent.
    if let Ok(static_literal) = site.range_expression.trim().parse::<i64>() {
        if static_literal != draw.range {
            return false;
        }
    }
    true
}

/// Join runtime draws to symbolic RNG events conservatively.
///
/// The join key is fighter ID + source file/section + line + overload width,
/// plus exact range equality when the symbolic range is a numeric literal.
/// Multiple matches stay Ambiguous because the runtime trace has no column.
pub fn reconcile_capture(
    capture: &RuntimeCapture,
    symbolic_sites: &[SymbolicRngSite],
) -> Vec<DrawReconciliation> {
    let mut controls_by_invocation: HashMap<(Option<u64>, Option<u64>), Vec<RuntimeControl>> =
        HashMap::new();
    for control in &capture.controls {
        controls_by_invocation
            .entry((control.tick_index, control.script_execution_id))
            .or_default()
            .push(control.clone());
    }

    capture
        .draws
        .iter()
        .cloned()
        .map(|draw| {
            let invocation_controls = controls_by_invocation
                .get(&(draw.tick_index, draw.script_execution_id))
                .cloned()
                .unwrap_or_default();

            if draw.source_kind != "rot_expression" {
                return DrawReconciliation {
                    draw,
                    status: SiteMatchStatus::NonRotSource,
                    candidates: Vec::new(),
                    invocation_controls,
                };
            }

            let candidates: Vec<_> = symbolic_sites
                .iter()
                .filter(|site| candidate_matches(&draw, site))
                .cloned()
                .collect();
            let status = match candidates.len() {
                0 => SiteMatchStatus::Unmatched,
                1 => SiteMatchStatus::Unique,
                _ => SiteMatchStatus::Ambiguous,
            };

            DrawReconciliation {
                draw,
                status,
                candidates,
                invocation_controls,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    const PASCAL_SAMPLE: &str =
        include_str!("../tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl");
    const SYMBOLIC_SAMPLE: &str =
        include_str!("../tests/fixtures/symbolic-rng-site-common-ia-293.json");

    fn load_samples() -> (RuntimeCapture, Vec<SymbolicRngSite>) {
        let capture = load_runtime_capture(BufReader::new(PASCAL_SAMPLE.as_bytes())).unwrap();
        let sites = load_symbolic_rng_sites(SYMBOLIC_SAMPLE.as_bytes()).unwrap();
        (capture, sites)
    }

    #[test]
    fn real_pascal_draw_joins_to_the_static_symbolic_group() {
        let (capture, sites) = load_samples();
        let joined = reconcile_capture(&capture, &sites);

        assert_eq!(capture.session_id, "gravar-seed-20261026");
        assert_eq!(joined.len(), 1);
        let result = &joined[0];
        assert_eq!(result.status, SiteMatchStatus::Unique);
        assert_eq!(result.draw.tick_index, Some(316));
        assert_eq!(result.draw.api_call_index, 0);
        assert_eq!(result.draw.result, 12);
        assert_eq!(result.draw.width, "LongInt");
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].rng_site, "Common.IA.L293.C4");
        assert_eq!(result.candidates[0].group_index, 0);
        assert!(result.candidates[0].guards.is_empty());
        assert!(result.candidates[0].loop_context.is_empty());
        assert_eq!(result.invocation_controls.len(), 1);
        assert_eq!(result.invocation_controls[0].source_line, 293);
        assert_eq!(result.invocation_controls[0].branch_taken, Some(false));
    }

    #[test]
    fn ambiguous_source_line_matches_are_never_forced_to_one_column() {
        let (capture, sites) = load_samples();
        let mut ambiguous_sites = sites.clone();
        let mut duplicate = sites[0].clone();
        duplicate.source_column = 99;
        duplicate.group_index = 1;
        ambiguous_sites.push(duplicate);

        let joined = reconcile_capture(&capture, &ambiguous_sites);
        assert_eq!(joined[0].status, SiteMatchStatus::Ambiguous);
        assert_eq!(joined[0].candidates.len(), 2);
    }

    #[test]
    fn helper_draws_are_not_misrepresented_as_static_rot_sites() {
        let (mut capture, sites) = load_samples();
        capture.draws[0].source_kind = "pascal_helper".to_owned();

        let joined = reconcile_capture(&capture, &sites);
        assert_eq!(joined[0].status, SiteMatchStatus::NonRotSource);
        assert!(joined[0].candidates.is_empty());
    }
}
