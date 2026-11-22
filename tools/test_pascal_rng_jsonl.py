#!/usr/bin/env python3
"""Unit tests for the schema-v2 Pascal runtime trace validator."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


VALIDATOR = Path(__file__).with_name("validate-pascal-rng-jsonl.py")


def session_record() -> dict[str, object]:
    return {
        "record_type": "session",
        "schema_version": 2,
        "session_id": "gravar-seed-20261026",
        "rng_scope": "SortearValor",
        "rand_seed_initial": 20261026,
        "raw_word_index_scope": (
            "SortearValor-only; direct System.Random calls outside this wrapper are excluded"
        ),
        "context_status": "logical_tick_and_nested_script_context_attached",
    }


def tick_start() -> dict[str, object]:
    return {
        "record_type": "tick_start",
        "schema_version": 2,
        "session_id": "gravar-seed-20261026",
        "tick_index": 0,
        "executed": True,
        "game_state": 7,
    }


def tick_end() -> dict[str, object]:
    return {
        "record_type": "tick_end",
        "schema_version": 2,
        "session_id": "gravar-seed-20261026",
        "tick_index": 0,
        "game_state": 7,
    }


def context(tick: int | None, script_id: int | None, parent: int | None = None) -> dict[str, object]:
    return {
        "tick_index": tick,
        "script_execution_id": script_id,
        "parent_script_execution_id": parent,
        "script_origin": "C01.rot [Acao]" if script_id is not None else None,
        "fighter_index": 1 if script_id is not None else None,
        "character_id": 1 if script_id is not None else None,
        "action_id": 3 if script_id is not None else None,
        "action_frame": 4 if script_id is not None else None,
        "command_code": 3 if script_id is not None else None,
    }


def script_start(script_id: int = 0, parent: int | None = None) -> dict[str, object]:
    return {
        "record_type": "script_start",
        "schema_version": 2,
        "session_id": "gravar-seed-20261026",
        **context(0, script_id, parent),
    }


def script_end(script_id: int = 0, draws: int = 1, parent: int | None = None) -> dict[str, object]:
    return {
        "record_type": "script_end",
        "schema_version": 2,
        "session_id": "gravar-seed-20261026",
        **context(0, script_id, parent),
        "script_draw_count": draws,
    }


def draw_record(
    api_index: int,
    first_word: int | None,
    raw_words: int,
    width: str,
    source: str = "System.Random",
    mode: str = "normal",
    script_id: int | None = 0,
    script_draw_index: int | None = 0,
) -> dict[str, object]:
    is_script = script_id is not None
    return {
        "record_type": "draw",
        "schema_version": 2,
        "session_id": "gravar-seed-20261026",
        "rng_scope": "SortearValor",
        "api_call_index": api_index,
        "first_mt_word_index": first_word,
        "raw_mt_words": raw_words,
        "width": width,
        "range": 8,
        "result": 2,
        "recorder_mode": mode,
        "draw_source": source,
        "rand_seed_before": 20261026,
        "rand_seed_after": 20261026,
        "source_kind": "rot_expression" if is_script else "pascal_direct",
        "source_origin": "C01.rot [Acao]" if is_script else None,
        "source_line": 25 if is_script else None,
        "script_draw_index": script_draw_index if is_script else None,
        **context(0, script_id),
    }


def control_record() -> dict[str, object]:
    return {
        "record_type": "control",
        "schema_version": 2,
        "session_id": "gravar-seed-20261026",
        "control_event_index": 0,
        "control_type": "if_branch",
        "source_line": 20,
        **context(0, 0),
        "branch_taken": True,
    }


class PascalRngJsonlTests(unittest.TestCase):
    def run_validator(self, records: list[dict[str, object]]) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as directory:
            trace = Path(directory) / "trace.jsonl"
            trace.write_text(
                "".join(json.dumps(record) + "\n" for record in records),
                encoding="utf-8",
            )
            return subprocess.run(
                [sys.executable, str(VALIDATOR), str(trace)],
                check=False,
                capture_output=True,
                text=True,
            )

    def test_accepts_tick_script_control_and_rng_records(self) -> None:
        records = [
            session_record(), tick_start(), script_start(), control_record(),
            draw_record(0, 0, 1, "LongInt"), script_end(), tick_end(),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("1 ticks, 1 script executions, 1 branch/loop records", result.stdout)
        self.assertIn("1 instrumented game RNG calls", result.stdout)

    def test_accepts_nested_script_frames(self) -> None:
        records = [
            session_record(), tick_start(), script_start(0),
            script_start(1, 0),
            draw_record(0, 0, 1, "LongInt", script_id=1, script_draw_index=0),
            script_end(1, draws=1, parent=0),
            script_end(0, draws=0),
            tick_end(),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("2 script executions", result.stdout)

    def test_accepts_replay_without_advancing_word_cursor(self) -> None:
        records = [
            session_record(), tick_start(),
            draw_record(
                0, None, 0, "LongInt", source="recorder_replay",
                mode="replay", script_id=None,
            ),
            tick_end(),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("1 replay", result.stdout)
        self.assertIn("0 instrumented MT word(s)", result.stdout)

    def test_accepts_pascal_helper_draw_inside_script_context(self) -> None:
        helper_draw = draw_record(0, 0, 1, "LongInt")
        helper_draw["source_kind"] = "pascal_helper"
        helper_draw["source_line"] = None
        records = [
            session_record(), tick_start(), script_start(),
            helper_draw, script_end(draws=1), tick_end(),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_rejects_fabricated_rot_line_for_pascal_helper_draw(self) -> None:
        helper_draw = draw_record(0, 0, 1, "LongInt")
        helper_draw["source_kind"] = "pascal_helper"
        helper_draw["source_line"] = 25
        records = [
            session_record(), tick_start(), script_start(),
            helper_draw, script_end(draws=1), tick_end(),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 1)
        self.assertIn("pascal_helper draw must not claim", result.stderr)

    def test_rejects_wrong_width_consumption(self) -> None:
        records = [
            session_record(), tick_start(), script_start(),
            draw_record(0, 0, 2, "LongInt"), script_end(), tick_end(),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 1)
        self.assertIn("LongInt must consume 1", result.stderr)

    def test_rejects_noncontiguous_api_call_index(self) -> None:
        records = [
            session_record(), tick_start(), script_start(),
            draw_record(1, 0, 1, "LongInt"), script_end(), tick_end(),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 1)
        self.assertIn("api_call_index", result.stderr)

    def test_rejects_replay_that_claims_raw_words(self) -> None:
        records = [
            session_record(), tick_start(),
            draw_record(
                0, 0, 1, "LongInt", source="recorder_replay",
                mode="replay", script_id=None,
            ),
            tick_end(),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 1)
        self.assertIn("replay must not claim MT-word consumption", result.stderr)

    def test_rejects_unclosed_tick(self) -> None:
        result = self.run_validator([session_record(), tick_start()])
        self.assertEqual(result.returncode, 1)
        self.assertIn("tick 0 still open", result.stderr)

    def test_rejects_script_end_out_of_nesting_order(self) -> None:
        records = [
            session_record(), tick_start(), script_start(0),
            script_start(1, 0), script_end(0, draws=0), tick_end(),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 1)
        self.assertIn("does not close the active script", result.stderr)

    def test_rejects_draw_with_wrong_script_local_index(self) -> None:
        records = [
            session_record(), tick_start(), script_start(),
            draw_record(0, 0, 1, "LongInt", script_draw_index=2),
            script_end(), tick_end(),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 1)
        self.assertIn("script_draw_index", result.stderr)

    def test_accepts_captured_pascal_tick316_fixture(self) -> None:
        fixture = Path(__file__).resolve().parents[1] / "tests" / "fixtures" / "pascal-runtime-schema-v2-tick316.jsonl"
        result = subprocess.run(
            [sys.executable, str(VALIDATOR), str(fixture)],
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("1 ticks, 1 script executions, 1 branch/loop records", result.stdout)
        self.assertIn("1 SortearValor calls", result.stdout)
        self.assertIn("1 instrumented MT word(s)", result.stdout)

    def test_rejects_missing_session_header(self) -> None:
        result = self.run_validator([tick_start()])
        self.assertEqual(result.returncode, 1)
        self.assertIn("event appeared before session header", result.stderr)


if __name__ == "__main__":
    unittest.main()
