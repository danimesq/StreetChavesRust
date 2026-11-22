//! First-divergence comparison for Pascal and Rust per-tick checkpoints.
//!
//! Both sides use the same compact JSONL shape: tick_start/tick_end records.
//! This module compares only fields actually emitted by the Pascal harness;
//! it does not imply a full-memory comparison.

use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;
use std::io::BufRead;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CheckpointPhase {
    Start,
    End,
}

impl CheckpointPhase {
    fn as_str(self) -> &'static str {
        match self { Self::Start => "tick_start", Self::End => "tick_end" }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickCheckpoint {
    pub tick_index: u64,
    pub phase: CheckpointPhase,
    pub executed: Option<bool>,
    pub game_state: Option<i64>,
    pub p1_character_id: Option<i64>,
    pub p1_action_id: Option<i64>,
    pub p1_action_frame: Option<i64>,
    pub p1_action_duration: Option<i64>,
    pub p2_character_id: Option<i64>,
    pub p2_action_id: Option<i64>,
    pub p2_action_frame: Option<i64>,
    pub p2_action_duration: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointError {
    Io(String),
    Json { line: usize, message: String },
    InvalidRecord { line: usize, message: String },
    Duplicate { tick_index: u64, phase: CheckpointPhase },
}

impl fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(f, "checkpoint I/O error: {message}"),
            Self::Json { line, message } => write!(f, "invalid checkpoint JSON on line {line}: {message}"),
            Self::InvalidRecord { line, message } => write!(f, "invalid checkpoint record on line {line}: {message}"),
            Self::Duplicate { tick_index, phase } => write!(f, "duplicate {} for tick {tick_index}", phase.as_str()),
        }
    }
}
impl std::error::Error for CheckpointError {}

fn optional_i64(v: &Value, key: &str) -> Option<i64> {
    v.get(key).and_then(Value::as_i64)
}
fn optional_bool(v: &Value, key: &str) -> Option<bool> {
    v.get(key).and_then(Value::as_bool)
}

fn parse_checkpoint(v: &Value, line: usize) -> Result<TickCheckpoint, CheckpointError> {
    let kind = v.get("record_type").and_then(Value::as_str).ok_or_else(|| CheckpointError::InvalidRecord {
        line, message: "record_type must be a string".into()
    })?;
    let phase = match kind {
        "tick_start" => CheckpointPhase::Start,
        "tick_end" => CheckpointPhase::End,
        _ => return Err(CheckpointError::InvalidRecord { line, message: format!("unsupported record_type {kind}") }),
    };
    let tick_index = v.get("tick_index").and_then(Value::as_u64).ok_or_else(|| CheckpointError::InvalidRecord {
        line, message: "tick_index must be a non-negative integer".into()
    })?;
    Ok(TickCheckpoint {
        tick_index, phase,
        executed: optional_bool(v, "executed"),
        game_state: optional_i64(v, "game_state"),
        p1_character_id: optional_i64(v, "p1_character_id"),
        p1_action_id: optional_i64(v, "p1_action_id"),
        p1_action_frame: optional_i64(v, "p1_action_frame"),
        p1_action_duration: optional_i64(v, "p1_action_duration"),
        p2_character_id: optional_i64(v, "p2_character_id"),
        p2_action_id: optional_i64(v, "p2_action_id"),
        p2_action_frame: optional_i64(v, "p2_action_frame"),
        p2_action_duration: optional_i64(v, "p2_action_duration"),
    })
}

/// Read Pascal-compatible tick_start/tick_end records from a JSONL stream.
pub fn load_tick_checkpoints<R: BufRead>(reader: R) -> Result<Vec<TickCheckpoint>, CheckpointError> {
    let mut rows = BTreeMap::new();
    for (index, raw) in reader.lines().enumerate() {
        let line = index + 1;
        let raw = raw.map_err(|e| CheckpointError::Io(e.to_string()))?;
        if raw.trim().is_empty() { continue; }
        let value: Value = serde_json::from_str(&raw).map_err(|e| CheckpointError::Json { line, message: e.to_string() })?;
        let kind = value.get("record_type").and_then(Value::as_str).unwrap_or("");
        if kind != "tick_start" && kind != "tick_end" { continue; }
        let checkpoint = parse_checkpoint(&value, line)?;
        let key = (checkpoint.tick_index, checkpoint.phase);
        if rows.insert(key, checkpoint.clone()).is_some() {
            return Err(CheckpointError::Duplicate { tick_index: checkpoint.tick_index, phase: checkpoint.phase });
        }
    }
    Ok(rows.into_values().collect())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstDivergence {
    pub tick_index: u64,
    pub phase: CheckpointPhase,
    pub field: String,
    pub pascal: Option<String>,
    pub rust: Option<String>,
}

/// Return the first unequal field in tick/phase/source-field order.
/// A missing checkpoint is reported as a divergence, never silently skipped.
pub fn first_divergence(pascal: &[TickCheckpoint], rust: &[TickCheckpoint]) -> Option<FirstDivergence> {
    let mut p = BTreeMap::new();
    let mut r = BTreeMap::new();
    for row in pascal { p.insert((row.tick_index, row.phase), row); }
    for row in rust { r.insert((row.tick_index, row.phase), row); }
    let keys: std::collections::BTreeSet<_> = p.keys().chain(r.keys()).copied().collect();
    for (tick, phase) in keys {
        let (Some(a), Some(b)) = (p.get(&(tick, phase)), r.get(&(tick, phase))) else {
            return Some(FirstDivergence {
                tick_index: tick, phase, field: "checkpoint_presence".into(),
                pascal: p.get(&(tick, phase)).map(|_| "present".into()),
                rust: r.get(&(tick, phase)).map(|_| "present".into()),
            });
        };
        macro_rules! compare {
            ($field:ident) => {
                if a.$field != b.$field {
                    return Some(FirstDivergence {
                        tick_index: tick, phase, field: stringify!($field).into(),
                        pascal: a.$field.map(|v| v.to_string()),
                        rust: b.$field.map(|v| v.to_string()),
                    });
                }
            }
        }
        compare!(executed); compare!(game_state);
        compare!(p1_character_id); compare!(p1_action_id); compare!(p1_action_frame);
        compare!(p1_action_duration); compare!(p2_character_id); compare!(p2_action_id);
        compare!(p2_action_frame); compare!(p2_action_duration);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const PASCAL: &str = concat!(
        "{\"record_type\":\"tick_start\",\"tick_index\":0,\"executed\":true,\"game_state\":7,\"p1_character_id\":2,\"p1_action_id\":1,\"p1_action_frame\":0,\"p1_action_duration\":9,\"p2_character_id\":1,\"p2_action_id\":3,\"p2_action_frame\":2,\"p2_action_duration\":8}\n",
        "{\"record_type\":\"tick_end\",\"tick_index\":0,\"game_state\":7,\"p1_character_id\":2,\"p1_action_id\":1,\"p1_action_frame\":1,\"p2_character_id\":1,\"p2_action_id\":3,\"p2_action_frame\":3}\n"
    );

    #[test]
    fn identical_checkpoints_have_no_divergence() {
        let p = load_tick_checkpoints(Cursor::new(PASCAL)).unwrap();
        assert_eq!(first_divergence(&p, &p), None);
    }

    #[test]
    fn reports_first_different_field_and_tick() {
        let p = load_tick_checkpoints(Cursor::new(PASCAL)).unwrap();
        let mut r = p.clone();
        r[1].p2_action_frame = Some(99);
        let d = first_divergence(&p, &r).unwrap();
        assert_eq!((d.tick_index, d.phase, d.field.as_str()), (0, CheckpointPhase::End, "p2_action_frame"));
        assert_eq!(d.pascal.as_deref(), Some("3"));
        assert_eq!(d.rust.as_deref(), Some("99"));
    }

    #[test]
    fn missing_checkpoint_is_a_divergence() {
        let p = load_tick_checkpoints(Cursor::new(PASCAL)).unwrap();
        assert_eq!(first_divergence(&p, &p[..1]).unwrap().field, "checkpoint_presence");
    }
}
