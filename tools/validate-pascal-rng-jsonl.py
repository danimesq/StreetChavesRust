#!/usr/bin/env python3
"""Validate schema-v2 JSONL emitted by OpenStreetChaves runtime tracing.

The MT-word cursor spans instrumented game-source System.Random call sites
routed through SortearValor. External RTL/library calls are not intercepted.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


class TraceValidationError(ValueError):
    """Raised when a Pascal runtime trace is malformed or internally inconsistent."""


def _object(value: Any, line_number: int) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TraceValidationError(f"line {line_number}: each JSONL record must be an object")
    return value


def _integer(record: dict[str, Any], key: str, line_number: int) -> int:
    value = record.get(key)
    if isinstance(value, bool) or not isinstance(value, int):
        raise TraceValidationError(f"line {line_number}: {key!r} must be an integer")
    return value


def _context_tick(record: dict[str, Any], active_tick: int | None, line_number: int) -> None:
    tick = record.get("tick_index")
    if active_tick is None:
        if tick is not None:
            raise TraceValidationError(f"line {line_number}: event claims a tick while no tick is open")
    elif tick != active_tick:
        raise TraceValidationError(
            f"line {line_number}: event tick_index {tick!r} != active tick {active_tick}"
        )


def validate_trace(path: Path) -> dict[str, Any]:
    """Validate ordering, nested script contexts, controls and RNG cursor semantics."""
    session: dict[str, Any] | None = None
    next_api_call = next_mt_word = next_tick = next_script_id = next_control_id = 0
    system_random_calls = replay_calls = control_records = script_records = 0
    active_tick: int | None = None
    script_stack: list[int] = []
    script_draw_counts: dict[int, int] = {}
    tick_count = 0

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
                if record.get("schema_version") != 2:
                    raise TraceValidationError(f"line {line_number}: schema_version must be 2")
                if record.get("rng_scope") != "SortearValor":
                    raise TraceValidationError(f"line {line_number}: unexpected rng_scope")
                session_id = record.get("session_id")
                if not isinstance(session_id, str) or not session_id:
                    raise TraceValidationError(f"line {line_number}: session_id must be non-empty text")
                _integer(record, "rand_seed_initial", line_number)
                # Window fixtures preserve original event records and declare the
                # cursors at which that consecutive excerpt begins.
                for offset_key, counter_name in (
                    ("tick_index_start", "tick"),
                    ("script_execution_id_start", "script"),
                    ("control_event_index_start", "control"),
                    ("api_call_index_start", "api"),
                    ("first_mt_word_index_start", "mt"),
                ):
                    offset = record.get(offset_key, 0)
                    if isinstance(offset, bool) or not isinstance(offset, int) or offset < 0:
                        raise TraceValidationError(
                            f"line {line_number}: {offset_key} must be a non-negative integer"
                        )
                    if counter_name == "tick":
                        next_tick = offset
                    elif counter_name == "script":
                        next_script_id = offset
                    elif counter_name == "control":
                        next_control_id = offset
                    elif counter_name == "api":
                        next_api_call = offset
                    elif counter_name == "mt":
                        next_mt_word = offset
                if record.get("record_scope", "full_trace") not in (
                    "full_trace", "consecutive_tick_window", "event_sample"
                ):
                    raise TraceValidationError(f"line {line_number}: unsupported record_scope")
                if record.get("record_scope") in ("consecutive_tick_window", "event_sample"):
                    required_offsets = (
                        "tick_index_start", "script_execution_id_start",
                        "control_event_index_start",
                    )
                    if any(key not in record for key in required_offsets):
                        raise TraceValidationError(
                            f"line {line_number}: trace excerpt is missing cursor offsets"
                        )
                    sample_description = record.get("sample_description")
                    if record.get("record_scope") == "event_sample" and not isinstance(
                        sample_description, str
                    ):
                        raise TraceValidationError(
                            f"line {line_number}: event_sample requires a sample_description"
                        )
                scope = record.get("raw_word_index_scope")
                if not isinstance(scope, str) or not (
                    "SortearValor-only" in scope or "all instrumented game-source" in scope
                ):
                    raise TraceValidationError(
                        f"line {line_number}: raw_word_index_scope must disclose instrumented RNG scope"
                    )
                source_column_scope = record.get("source_column_scope", "legacy")
                if source_column_scope not in (
                    "legacy",
                    "direct_pascal_inc",
                    "direct_pascal_inc+rot_ast",
                ):
                    raise TraceValidationError(
                        f"line {line_number}: unsupported source_column_scope"
                    )
                session = record
                continue

            if session is None:
                raise TraceValidationError(f"line {line_number}: event appeared before session header")
            if record.get("schema_version") != 2:
                raise TraceValidationError(f"line {line_number}: schema_version must be 2")
            if record.get("session_id") != session["session_id"]:
                raise TraceValidationError(f"line {line_number}: session_id differs from header")

            if record_type == "tick_start":
                if active_tick is not None:
                    raise TraceValidationError(f"line {line_number}: nested tick_start")
                tick = _integer(record, "tick_index", line_number)
                if tick != next_tick:
                    raise TraceValidationError(
                        f"line {line_number}: tick_index {tick} != expected {next_tick}"
                    )
                if not isinstance(record.get("executed"), bool):
                    raise TraceValidationError(f"line {line_number}: tick_start.executed must be boolean")
                _integer(record, "game_state", line_number)
                active_tick = tick
                next_tick += 1
                tick_count += 1
                continue

            if record_type == "tick_end":
                tick = _integer(record, "tick_index", line_number)
                if active_tick is None or tick != active_tick:
                    raise TraceValidationError(f"line {line_number}: tick_end does not close active tick")
                if script_stack:
                    raise TraceValidationError(f"line {line_number}: tick ended with an open script context")
                _integer(record, "game_state", line_number)
                active_tick = None
                continue

            if record_type in ("script_start", "script_end", "control", "draw"):
                _context_tick(record, active_tick, line_number)

            if record_type == "script_start":
                script_id = _integer(record, "script_execution_id", line_number)
                if script_id != next_script_id:
                    raise TraceValidationError(
                        f"line {line_number}: script_execution_id {script_id} != expected {next_script_id}"
                    )
                next_script_id += 1
                parent_id = record.get("parent_script_execution_id")
                expected_parent = script_stack[-1] if script_stack else None
                if parent_id != expected_parent:
                    raise TraceValidationError(
                        f"line {line_number}: parent_script_execution_id {parent_id!r} "
                        f"!= active parent {expected_parent!r}"
                    )
                if not isinstance(record.get("script_origin"), str):
                    raise TraceValidationError(f"line {line_number}: script_origin must be text")
                _integer(record, "fighter_index", line_number)
                _integer(record, "character_id", line_number)
                _integer(record, "command_code", line_number)
                script_stack.append(script_id)
                script_draw_counts[script_id] = 0
                script_records += 1
                continue

            if record_type == "script_end":
                script_id = _integer(record, "script_execution_id", line_number)
                if not script_stack or script_stack[-1] != script_id:
                    raise TraceValidationError(
                        f"line {line_number}: script_end does not close the active script"
                    )
                if _integer(record, "script_draw_count", line_number) != script_draw_counts[script_id]:
                    raise TraceValidationError(f"line {line_number}: script_draw_count mismatch")
                script_stack.pop()
                continue

            if record_type == "control":
                if not script_stack:
                    raise TraceValidationError(f"line {line_number}: control event outside a script")
                control_id = _integer(record, "control_event_index", line_number)
                if control_id != next_control_id:
                    raise TraceValidationError(
                        f"line {line_number}: control_event_index {control_id} != expected {next_control_id}"
                    )
                next_control_id += 1
                _integer(record, "source_line", line_number)
                if not isinstance(record.get("control_type"), str):
                    raise TraceValidationError(f"line {line_number}: control_type must be text")
                if record.get("script_execution_id") != script_stack[-1]:
                    raise TraceValidationError(f"line {line_number}: control event script context mismatch")
                control_records += 1
                continue

            if record_type != "draw":
                raise TraceValidationError(
                    f"line {line_number}: unknown record_type {record_type!r}"
                )
            if record.get("rng_scope") != "SortearValor":
                raise TraceValidationError(f"line {line_number}: unexpected rng_scope")
            if record.get("tick_index") is not None and active_tick is None:
                raise TraceValidationError(f"line {line_number}: draw has a tick but no tick is open")

            script_id = record.get("script_execution_id")
            if script_stack:
                if script_id != script_stack[-1]:
                    raise TraceValidationError(f"line {line_number}: draw script context mismatch")
            elif script_id is not None:
                raise TraceValidationError(f"line {line_number}: draw references an inactive script")

            api_call_index = _integer(record, "api_call_index", line_number)
            if api_call_index != next_api_call:
                raise TraceValidationError(
                    f"line {line_number}: api_call_index {api_call_index} != expected {next_api_call}"
                )
            next_api_call += 1

            width = record.get("width")
            if width not in ("LongInt", "Int64"):
                raise TraceValidationError(f"line {line_number}: width must be LongInt or Int64")
            range_value = _integer(record, "range", line_number)
            result = _integer(record, "result", line_number)
            _integer(record, "rand_seed_before", line_number)
            _integer(record, "rand_seed_after", line_number)

            recorder_mode = record.get("recorder_mode")
            if recorder_mode not in (
                "normal", "record", "replay", "replay_mismatch_fallback"
            ):
                raise TraceValidationError(f"line {line_number}: unknown recorder_mode")

            draw_source = record.get("draw_source")
            raw_mt_words = _integer(record, "raw_mt_words", line_number)
            first_word = record.get("first_mt_word_index")
            script_draw_index = record.get("script_draw_index")

            source_kind = record.get("source_kind")
            if script_id is None:
                if script_draw_index is not None:
                    raise TraceValidationError(f"line {line_number}: draw without script has script_draw_index")
                if source_kind != "pascal_direct":
                    raise TraceValidationError(f"line {line_number}: draw without script must be pascal_direct")
                source_origin = record.get("source_origin")
                source_line = record.get("source_line")
                source_column = record.get("source_column")
                if source_origin is not None and not isinstance(source_origin, str):
                    raise TraceValidationError(f"line {line_number}: direct source_origin must be text or null")
                if source_line is not None:
                    _integer(record, "source_line", line_number)
                if source_column is not None:
                    _integer(record, "source_column", line_number)
                    if source_column <= 0:
                        raise TraceValidationError(f"line {line_number}: source_column must be positive")
                if (source_origin is None) != (source_line is None):
                    raise TraceValidationError(
                        f"line {line_number}: direct source_origin and source_line must be present together"
                    )
                if source_origin is not None and source_origin.lower().endswith(".inc") and source_column is None:
                    raise TraceValidationError(
                        f"line {line_number}: direct .inc draw must include source_column"
                    )
            else:
                expected_draw_index = script_draw_counts[script_id]
                if script_draw_index != expected_draw_index:
                    raise TraceValidationError(
                        f"line {line_number}: script_draw_index {script_draw_index!r} "
                        f"!= expected {expected_draw_index}"
                    )
                if not isinstance(record.get("source_origin"), str):
                    raise TraceValidationError(f"line {line_number}: script-context draw must include source_origin")
                if source_kind == "rot_expression":
                    _integer(record, "source_line", line_number)
                    if session.get("source_column_scope") == "direct_pascal_inc+rot_ast":
                        if record.get("source_column") is None:
                            raise TraceValidationError(
                                f"line {line_number}: ROT expression draw requires source_column"
                            )
                        _integer(record, "source_column", line_number)
                    elif record.get("source_column") is not None:
                        _integer(record, "source_column", line_number)
                elif source_kind == "pascal_helper":
                    if record.get("source_line") is not None:
                        _integer(record, "source_line", line_number)
                        if ".rot" in record["source_origin"].lower():
                            raise TraceValidationError(
                                f"line {line_number}: pascal_helper draw must not claim a .rot source line"
                            )
                        if record["source_origin"].lower().endswith(".inc"):
                            if record.get("source_column") is None:
                                raise TraceValidationError(
                                    f"line {line_number}: direct helper call must include source_column"
                                )
                            _integer(record, "source_column", line_number)
                    elif record.get("source_column") is not None:
                        raise TraceValidationError(
                            f"line {line_number}: helper source_column requires source_line"
                        )
                else:
                    raise TraceValidationError(
                        f"line {line_number}: script-context draw source_kind must be "
                        "'rot_expression' or 'pascal_helper'"
                    )
                script_draw_counts[script_id] += 1

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
                    f"line {line_number}: replay request cannot claim a System.Random draw"
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
            if range_value > 0 and not 0 <= result < range_value:
                raise TraceValidationError(
                    f"line {line_number}: result is outside the positive requested range"
                )
            next_mt_word += expected_words
            system_random_calls += 1

    if session is None:
        raise TraceValidationError("trace is empty: missing session header")
    if active_tick is not None:
        raise TraceValidationError(f"trace ended with tick {active_tick} still open")
    if script_stack:
        raise TraceValidationError(f"trace ended with open script context(s): {script_stack}")

    return {
        "session_id": session["session_id"],
        "rand_seed_initial": session["rand_seed_initial"],
        "ticks": tick_count,
        "script_executions": script_records,
        "control_records": control_records,
        "api_calls": next_api_call,
        "system_random_calls": system_random_calls,
        "replay_calls": replay_calls,
        "raw_mt_words": next_mt_word,
        "rng_scope": "SortearValor",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", type=Path, help="Pascal runtime JSONL file")
    args = parser.parse_args()

    try:
        summary = validate_trace(args.trace)
    except TraceValidationError as exc:
        print(f"invalid trace: {exc}", file=sys.stderr)
        return 1

    print(
        f"valid trace {summary['session_id']}: "
        f"{summary['ticks']} ticks, {summary['script_executions']} script executions, "
        f"{summary['control_records']} branch/loop records, "
        f"{summary['api_calls']} instrumented game RNG calls "
        f"({summary['system_random_calls']} System.Random, {summary['replay_calls']} replay), "
        f"{summary['raw_mt_words']} instrumented MT word(s), "
        f"initial RandSeed={summary['rand_seed_initial']}"
    )
    print(
        "scope note: game-source System.Random call sites are routed through SortearValor; "
        "direct Pascal .inc and ROT expression draws include source columns; concrete short-circuit "
        "subexpression values are not emitted"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
