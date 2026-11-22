#!/usr/bin/env python3
"""Unit tests for validate-pascal-rng-jsonl.py."""

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
        "schema_version": 1,
        "session_id": "gravar-seed-20261026",
        "rng_scope": "SortearValor",
        "rand_seed_initial": 20261026,
        "raw_word_index_scope": (
            "SortearValor-only; direct System.Random calls outside this wrapper are excluded"
        ),
        "context_status": "action/tick context not attached yet",
    }


def draw_record(
    api_index: int,
    first_word: int | None,
    raw_words: int,
    width: str,
    source: str = "System.Random",
    mode: str = "normal",
) -> dict[str, object]:
    return {
        "record_type": "draw",
        "schema_version": 1,
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

    def test_accepts_longint_int64_and_replay_without_advancing_word_cursor(self) -> None:
        records = [
            session_record(),
            draw_record(0, 0, 1, "LongInt", mode="record"),
            draw_record(1, 1, 2, "Int64"),
            draw_record(2, None, 0, "LongInt", source="recorder_replay", mode="replay"),
            draw_record(3, 3, 1, "LongInt"),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("4 SortearValor calls", result.stdout)
        self.assertIn("3 wrapper-scoped MT word(s)", result.stdout)

    def test_rejects_wrong_width_consumption(self) -> None:
        records = [
            session_record(),
            draw_record(0, 0, 2, "LongInt"),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 1)
        self.assertIn("LongInt must consume 1", result.stderr)

    def test_rejects_noncontiguous_api_call_index(self) -> None:
        records = [
            session_record(),
            draw_record(1, 0, 1, "LongInt"),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 1)
        self.assertIn("api_call_index", result.stderr)

    def test_rejects_replay_that_claims_raw_words(self) -> None:
        records = [
            session_record(),
            draw_record(0, 0, 1, "LongInt", source="recorder_replay", mode="replay"),
        ]
        result = self.run_validator(records)
        self.assertEqual(result.returncode, 1)
        self.assertIn("replay must not claim MT-word consumption", result.stderr)

    def test_rejects_missing_session_header(self) -> None:
        result = self.run_validator([draw_record(0, 0, 1, "LongInt")])
        self.assertEqual(result.returncode, 1)
        self.assertIn("draw appeared before session header", result.stderr)


if __name__ == "__main__":
    unittest.main()
