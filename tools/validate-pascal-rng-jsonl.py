#!/usr/bin/env python3
"""Validate JSONL emitted by OpenStreetChaves SortearValor tracing.

The MT-word counter is intentionally scoped to calls that pass through
SortearValor. It is not a global FPC MTwist position.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


class TraceValidationError(ValueError):
    """Raised when a Pascal RNG trace is malformed or internally inconsistent."""


def _object(value: Any, line_number: int) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TraceValidationError(f"line {line_number}: each JSONL record must be an object")
    return value


def _integer(record: dict[str, Any], key: str, line_number: int) -> int:
    value = record.get(key)
    if isinstance(value, bool) or not isinstance(value, int):
        raise TraceValidationError(f"line {line_number}: {key!r} must be an integer")
    return value


def validate_trace(path: Path) -> dict[str, Any]:
    """Validate one trace and return a compact summary."""
    session: dict[str, Any] | None = None
    next_api_call = 0
    next_mt_word = 0
    system_random_calls = 0
    replay_calls = 0

    try:
        handle = path.open("r", encoding="utf-8")
    except OSError as exc:
        raise TraceValidationError(f"cannot open {path}: {exc}") from exc

    with handle:
        for line_number, raw_line in enumerate(handle, start=1):
            if not raw_line.strip():
                raise TraceValidationError(f"line {line_number}: blank lines are not valid JSONL records")
            try:
                record = _object(json.loads(raw_line), line_number)
            except json.JSONDecodeError as exc:
                raise TraceValidationError(
                    f"line {line_number}: invalid JSON at column {exc.colno}: {exc.msg}"
                ) from exc

            record_type = record.get("record_type")
            if record_type == "session":
                if session is not None or line_number != 1:
                    raise TraceValidationError(
                        f"line {line_number}: exactly one session record must appear first"
                    )
                if record.get("schema_version") != 1:
                    raise TraceValidationError(f"line {line_number}: unsupported schema_version")
                if record.get("rng_scope") != "SortearValor":
                    raise TraceValidationError(f"line {line_number}: unexpected rng_scope")
                session_id = record.get("session_id")
                if not isinstance(session_id, str) or not session_id:
                    raise TraceValidationError(f"line {line_number}: session_id must be non-empty text")
                seed = _integer(record, "rand_seed_initial", line_number)
                scope = record.get("raw_word_index_scope")
                if not isinstance(scope, str) or "SortearValor-only" not in scope:
                    raise TraceValidationError(
                        f"line {line_number}: raw_word_index_scope must disclose the wrapper-only scope"
                    )
                session = record
                continue

            if record_type != "draw":
                raise TraceValidationError(
                    f"line {line_number}: record_type must be 'session' or 'draw'"
                )
            if session is None:
                raise TraceValidationError(f"line {line_number}: draw appeared before session header")
            if record.get("schema_version") != 1:
                raise TraceValidationError(f"line {line_number}: unsupported schema_version")
            if record.get("rng_scope") != "SortearValor":
                raise TraceValidationError(f"line {line_number}: unexpected rng_scope")
            if record.get("session_id") != session["session_id"]:
                raise TraceValidationError(f"line {line_number}: session_id differs from header")

            api_call_index = _integer(record, "api_call_index", line_number)
            if api_call_index != next_api_call:
                raise TraceValidationError(
                    f"line {line_number}: api_call_index {api_call_index} != expected {next_api_call}"
                )
            next_api_call += 1

            width = record.get("width")
            if width not in ("LongInt", "Int64"):
                raise TraceValidationError(f"line {line_number}: width must be LongInt or Int64")
            _integer(record, "range", line_number)
            result = _integer(record, "result", line_number)
            _integer(record, "rand_seed_before", line_number)
            _integer(record, "rand_seed_after", line_number)

            recorder_mode = record.get("recorder_mode")
            if recorder_mode not in (
                "normal",
                "record",
                "replay",
                "replay_mismatch_fallback",
            ):
                raise TraceValidationError(f"line {line_number}: unknown recorder_mode")

            draw_source = record.get("draw_source")
            raw_mt_words = _integer(record, "raw_mt_words", line_number)
            first_word = record.get("first_mt_word_index")

            if draw_source == "recorder_replay":
                if recorder_mode != "replay":
                    raise TraceValidationError(
                        f"line {line_number}: recorder_replay must use recorder_mode='replay'"
                    )
                if first_word is not None or raw_mt_words != 0:
                    raise TraceValidationError(
                        f"line {line_number}: replay must not claim MT-word consumption"
                    )
                replay_calls += 1
                continue

            if draw_source != "System.Random":
                raise TraceValidationError(f"line {line_number}: unknown draw_source")
            if recorder_mode == "replay":
                raise TraceValidationError(
                    f"line {line_number}: a replay request cannot claim a System.Random draw"
                )

            expected_words = 2 if width == "Int64" else 1
            if raw_mt_words != expected_words:
                raise TraceValidationError(
                    f"line {line_number}: {width} must consume {expected_words} raw MT word(s)"
                )
            if isinstance(first_word, bool) or not isinstance(first_word, int):
                raise TraceValidationError(
                    f"line {line_number}: first_mt_word_index must be an integer for System.Random"
                )
            if first_word != next_mt_word:
                raise TraceValidationError(
                    f"line {line_number}: first_mt_word_index {first_word} != expected {next_mt_word}"
                )
            range_value = record["range"]
            if range_value > 0 and not 0 <= result < range_value:
                raise TraceValidationError(
                    f"line {line_number}: result is outside the positive requested range"
                )

            next_mt_word += expected_words
            system_random_calls += 1

    if session is None:
        raise TraceValidationError("trace is empty: missing session header")

    return {
        "session_id": session["session_id"],
        "rand_seed_initial": seed,
        "api_calls": next_api_call,
        "system_random_calls": system_random_calls,
        "replay_calls": replay_calls,
        "raw_mt_words": next_mt_word,
        "rng_scope": "SortearValor",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", type=Path, help="Pascal RNG JSONL file")
    args = parser.parse_args()

    try:
        summary = validate_trace(args.trace)
    except TraceValidationError as exc:
        print(f"invalid trace: {exc}", file=sys.stderr)
        return 1

    print(
        f"valid trace {summary['session_id']}: "
        f"{summary['api_calls']} SortearValor calls, "
        f"{summary['system_random_calls']} System.Random calls, "
        f"{summary['replay_calls']} replay returns, "
        f"{summary['raw_mt_words']} wrapper-scoped MT word(s), "
        f"initial RandSeed={summary['rand_seed_initial']}"
    )
    print(
        "scope note: direct System.Random calls outside SortearValor are excluded; "
        "action/tick context is not attached in schema v1"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
