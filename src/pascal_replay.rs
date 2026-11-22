//! Strict replay of concrete RNG requests captured by the Pascal schema-v2 exporter.
//!
//! This adapter replays recorded *results* in exact API order. It does not
//! regenerate MTwist values or claim coverage of direct System.Random calls
//! outside SortearValor.

use crate::trace_join::{load_runtime_capture, RuntimeCapture};
use std::fmt;
use std::io::BufRead;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PascalReplayWidth {
    LongInt,
    Int64,
}

impl PascalReplayWidth {
    fn as_str(self) -> &'static str {
        match self {
            Self::LongInt => "LongInt",
            Self::Int64 => "Int64",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PascalReplayError {
    Trace(String),
    Exhausted {
        expected_width: PascalReplayWidth,
        expected_range: i64,
    },
    RequestMismatch {
        api_call_index: u64,
        expected_width: String,
        actual_width: PascalReplayWidth,
        expected_range: i64,
        actual_range: i64,
    },
    InvalidRecordedDraw {
        api_call_index: u64,
        reason: String,
    },
    Unconsumed {
        remaining: usize,
        next_api_call_index: u64,
    },
}

impl fmt::Display for PascalReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trace(message) => write!(f, "invalid Pascal trace: {message}"),
            Self::Exhausted { expected_width, expected_range } =>
                write!(f, "Pascal replay exhausted (requested {} range {})", expected_width.as_str(), expected_range),
            Self::RequestMismatch { api_call_index, expected_width, actual_width, expected_range, actual_range } =>
                write!(f, "Pascal replay request mismatch at call {api_call_index}: expected {expected_width} range {expected_range}, got {} range {actual_range}", actual_width.as_str()),
            Self::InvalidRecordedDraw { api_call_index, reason } =>
                write!(f, "invalid recorded draw {api_call_index}: {reason}"),
            Self::Unconsumed { remaining, next_api_call_index } =>
                write!(f, "{remaining} Pascal draws remain; next API call is {next_api_call_index}"),
        }
    }
}

impl std::error::Error for PascalReplayError {}

/// Ordered recorded draws plus the next request cursor.
#[derive(Debug, Clone)]
pub struct PascalDrawReplay {
    capture: RuntimeCapture,
    cursor: usize,
}

impl PascalDrawReplay {
    /// Parse a real schema-v2 JSONL capture and validate its draw sequence.
    pub fn from_reader<R: BufRead>(reader: R) -> Result<Self, PascalReplayError> {
        let capture = load_runtime_capture(reader)
            .map_err(|error| PascalReplayError::Trace(error.to_string()))?;

        for (position, draw) in capture.draws.iter().enumerate() {
            if draw.api_call_index != position as u64 {
                return Err(PascalReplayError::InvalidRecordedDraw {
                    api_call_index: draw.api_call_index,
                    reason: format!("expected sequential API index {position}"),
                });
            }
            let expected_words = match draw.width.as_str() {
                "LongInt" => 1,
                "Int64" => 2,
                other => {
                    return Err(PascalReplayError::InvalidRecordedDraw {
                        api_call_index: draw.api_call_index,
                        reason: format!("unknown Pascal Random width {other}"),
                    })
                }
            };
            if draw.range <= 0 || draw.result < 0 || draw.result >= draw.range {
                return Err(PascalReplayError::InvalidRecordedDraw {
                    api_call_index: draw.api_call_index,
                    reason: format!("result {} is outside [0, {})", draw.result, draw.range),
                });
            }
            if draw.raw_mt_words != expected_words {
                return Err(PascalReplayError::InvalidRecordedDraw {
                    api_call_index: draw.api_call_index,
                    reason: format!(
                        "{} requires {expected_words} raw MT words, trace says {}",
                        draw.width, draw.raw_mt_words
                    ),
                });
            }
        }

        Ok(Self { capture, cursor: 0 })
    }

    /// Consume the next captured request. Width and range must match exactly.
    pub fn draw(&mut self, width: PascalReplayWidth, range: i64) -> Result<i64, PascalReplayError> {
        let Some(expected) = self.capture.draws.get(self.cursor) else {
            return Err(PascalReplayError::Exhausted {
                expected_width: width,
                expected_range: range,
            });
        };
        if expected.width != width.as_str() || expected.range != range {
            return Err(PascalReplayError::RequestMismatch {
                api_call_index: expected.api_call_index,
                expected_width: expected.width.clone(),
                actual_width: width,
                expected_range: expected.range,
                actual_range: range,
            });
        }
        self.cursor += 1;
        Ok(expected.result)
    }

    #[rustfmt::skip]
    /// Replay one Pascal Random(LongInt) request from the captured stream.
    /// The simulator should call this at the original source call site.
    pub fn random_longint(&mut self, range: i32) -> Result<i32, PascalReplayError> {
        let value = self.draw(PascalReplayWidth::LongInt, range as i64)?;
        i32::try_from(value).map_err(|_| PascalReplayError::InvalidRecordedDraw {
            api_call_index: self.cursor.saturating_sub(1) as u64,
            reason: format!("LongInt result {value} does not fit i32"),
        })
    }

    #[rustfmt::skip]
    /// Replay one Pascal Random(Int64) request from the captured stream.
    pub fn random_int64(&mut self, range: i64) -> Result<i64, PascalReplayError> {
        self.draw(PascalReplayWidth::Int64, range)
    }

    /// Number of recorded wrapper calls consumed so far.
    pub fn draws_consumed(&self) -> usize {
        self.cursor
    }

    /// Number of recorded wrapper calls still waiting to be replayed.
    pub fn draws_remaining(&self) -> usize {
        self.capture.draws.len().saturating_sub(self.cursor)
    }

    /// Require complete consumption; leftover draws indicate divergent control flow.
    pub fn finish(self) -> Result<(), PascalReplayError> {
        if let Some(next) = self.capture.draws.get(self.cursor) {
            Err(PascalReplayError::Unconsumed {
                remaining: self.draws_remaining(),
                next_api_call_index: next.api_call_index,
            })
        } else {
            Ok(())
        }
    }

    /// Access capture metadata without exposing mutable replay internals.
    pub fn capture(&self) -> &RuntimeCapture {
        &self.capture
    }
}

/// Shared overload-aware RNG interface for the action simulator.
/// Both the independent FPC generator and recorded Pascal replay implement it.
pub trait PascalRandomSource {
    type Error;

    fn draw_longint(&mut self, range: i32) -> Result<i32, Self::Error>;
    fn draw_int64(&mut self, range: i64) -> Result<i64, Self::Error>;
}

impl PascalRandomSource for PascalDrawReplay {
    type Error = PascalReplayError;

    fn draw_longint(&mut self, range: i32) -> Result<i32, Self::Error> {
        self.random_longint(range)
    }

    fn draw_int64(&mut self, range: i64) -> Result<i64, Self::Error> {
        self.random_int64(range)
    }
}

impl PascalRandomSource for crate::Fpc322Rng {
    type Error = std::convert::Infallible;

    fn draw_longint(&mut self, range: i32) -> Result<i32, Self::Error> {
        Ok(self.random_longint(range).value as i32)
    }

    fn draw_int64(&mut self, range: i64) -> Result<i64, Self::Error> {
        Ok(self.random_int64(range).value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const REAL_TICK_316: &str =
        include_str!("../tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl");

    fn replay() -> PascalDrawReplay {
        PascalDrawReplay::from_reader(Cursor::new(REAL_TICK_316)).unwrap()
    }

    #[test]
    fn replays_real_pascal_draw_with_exact_width_and_range() {
        let mut replay = replay();
        assert_eq!(replay.draw(PascalReplayWidth::LongInt, 13), Ok(12));
        assert_eq!(replay.draws_consumed(), 1);
        assert_eq!(replay.draws_remaining(), 0);
        assert_eq!(replay.finish(), Ok(()));
    }

    #[test]
    fn rejects_wrong_range_without_consuming_draw() {
        let mut replay = replay();
        assert!(matches!(
            replay.draw(PascalReplayWidth::LongInt, 12),
            Err(PascalReplayError::RequestMismatch {
                api_call_index: 0,
                ..
            })
        ));
        assert_eq!(replay.draws_consumed(), 0);
        assert_eq!(replay.draw(PascalReplayWidth::LongInt, 13), Ok(12));
    }

    #[test]
    fn rejects_wrong_overload_width_without_consuming_draw() {
        let mut replay = replay();
        assert!(matches!(
            replay.draw(PascalReplayWidth::Int64, 13),
            Err(PascalReplayError::RequestMismatch { .. })
        ));
        assert_eq!(replay.draws_consumed(), 0);
    }

    #[test]
    fn detects_unconsumed_real_draws() {
        let replay = replay();
        assert!(matches!(
            replay.finish(),
            Err(PascalReplayError::Unconsumed {
                remaining: 1,
                next_api_call_index: 0
            })
        ));
    }

    #[test]
    fn detects_exhaustion_after_last_real_draw() {
        let mut replay = replay();
        assert_eq!(replay.draw(PascalReplayWidth::LongInt, 13), Ok(12));
        assert!(matches!(
            replay.draw(PascalReplayWidth::LongInt, 13),
            Err(PascalReplayError::Exhausted { .. })
        ));
    }

    #[test]
    #[rustfmt::skip]
    fn width_specific_longint_entrypoint_is_simulation_friendly() {
        let mut replay = replay();
        assert_eq!(replay.random_longint(13), Ok(12));
        assert_eq!(replay.draws_consumed(), 1);
    }

    #[test]
    #[rustfmt::skip]
    fn width_specific_int64_entrypoint_replays_two_word_request() {
        let trace = concat!(
            "{\"record_type\":\"session\",\"schema_version\":2,\"session_id\":\"int64-test\"}\n",
            "{\"record_type\":\"draw\",\"api_call_index\":0,\"width\":\"Int64\",\"range\":108,\"result\":87,\"raw_mt_words\":2,\"source_kind\":\"rot_expression\"}\n"
        );
        let mut replay = PascalDrawReplay::from_reader(Cursor::new(trace)).unwrap();
        assert_eq!(replay.random_int64(108), Ok(87));
        assert_eq!(replay.finish(), Ok(()));
    }
}
