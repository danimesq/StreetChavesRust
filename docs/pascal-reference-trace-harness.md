# 🌈 Pascal reference trace harness

## 🌈 Why this matters

The .rot normalizer now preserves symbolic action/timer guards, loops, case reachability, RNG sites and the measured FPC 3.2.2 argument-evaluation profile. It does not evaluate concrete fighter state or assign dynamic ticks and raw MT-word indices.

The pinned Pascal project already contains two valuable regression mechanisms. Reuse them to produce dynamic reference traces rather than creating an unrelated random recorder.

## 🌈 Existing deterministic whole-game harness: --gravar

In source/ChavesSDL.pas, TAppSDL.GravarQuadros implements a deterministic logic/render regression run:

- Explicitly sets RandSeed to 20261026.
- Replays a fixed input sequence that repeats every 60 logic steps.
- Advances TJogo.PassoSDL without wall-clock waiting.
- Saves quadro-NNNNN.bmp at the chosen interval.
- Writes logica.txt with the step number, game state, and FNV-1a summaries of the complete Lutador and Efeitos structures.

This is a useful whole-game checkpoint oracle. It proves whether two runs with the same scripted inputs and seed produce the same captured state hashes. It does **not** currently emit each Random call or its raw MT-word position.

The harness is invoked by the Pascal executable as:

    StreetChaves.exe --gravar <output-directory> [steps] [interval]

Default values in the source are 3,000 steps and 50-step capture intervals. The recorded keyboard script is fixed in the Pascal source, so it is a broad smoke/regression scenario, not a targeted per-move fixture.

## 🌈 Existing deterministic draw recorder: ModoDeSorteio

ChavesRoteiro.pas centralizes script Random requests in SortearValor(N, Largo):

- msGravar calls the selected FPC System.Random overload and stores the tuple (N, Largo, R).
- msRepetir checks that the next request has the same N and Largo, then returns the stored R.
- A mismatch sets SorteioDivergiu and returns to normal RNG operation.
- SorteiosRestantes reports any recorded values that the replay did not consume.

The width bit matters: LongInt and Int64 requests are different even when their numeric range is equal. On the pinned FPC 3.2.2 runtime, their raw MT-word costs differ (one versus two).

This recorder already supports deterministic comparison of random-dependent paths. The experimental exporter described below observes the same shared wrapper; it does not replace the in-memory record/replay mechanism.

## 🌈 Existing old-code versus script differential harness: ChavesConferencia

ChavesConferencia.pas documents and implements the following procedure:

1. Take a snapshot of both fighters, effects, and selected game globals.
2. Set ModoDeSorteio(msGravar).
3. Run the old Pascal routine.
4. Return to normal RNG mode and snapshot the old result.
5. Restore the original game snapshot.
6. Set ModoDeSorteio(msRepetir).
7. Run the new routine/script against the exact random values consumed by the old routine.
8. Detect unconsumed or mismatched draws, compare fighter/effect/global fields, and append differences to conferencia.log.

This is an excellent oracle for the **Pascal migration**: it tests the old Pascal routine against its newer .rot-script counterpart under the same initial state and same random results.

It is not an original-executable oracle. The comparison is between two implementations inside the Pascal project, not between Pascal and the original Chaves.exe.

## 🌈 Experimental JSONL exporter (schema v1 history; superseded by v2 below)

The schema-v1 exporter description in this section is historical. The current
schema-v2 implementation and its validation results are documented below. The
Pascal research branch remains separate from upstream `main`; this Rust research
branch does not change the pinned normative source revision:
`e1b025c35ecbc2b14d8635564aafd23c2107d3d1`.

Run the existing deterministic harness with the added option:

```sh
StreetChaves.exe --gravar saida 3000 50 --rng-jsonl saida/rng.jsonl
```

Tracing is disabled unless `--rng-jsonl` is supplied. The new code is attached to
`SortearValor`, writes a session header followed by one JSON object per wrapper request,
and records:

| Field | Meaning |
|---|---|
| `session_id` | Deterministic capture/session identifier |
| `api_call_index` | Sequential request index within this trace |
| `first_mt_word_index` | First word index counted within the `SortearValor` wrapper scope |
| `raw_mt_words` | 1 for LongInt, 2 for Int64; 0 for an in-memory replay result |
| `width` | LongInt or Int64 overload selected by the interpreter |
| `range` / `result` | Exact request argument and returned value |
| `recorder_mode` | `normal`, `record`, `replay`, or replay-mismatch fallback |
| `draw_source` | `System.Random` or `recorder_replay` |
| `rand_seed_before` / `rand_seed_after` | RandSeed snapshots around the request |

A successful `msRepetir` result is logged as `recorder_replay`, with
`first_mt_word_index: null` and `raw_mt_words: 0`; replay does not consume the
generator. A replay mismatch retains the existing fallback to `System.Random`.

**Scope limitation:** the word index is local to calls through `SortearValor`.
The Pascal program also has direct `System.Random` call sites outside this wrapper,
so the v1 index is not a global MTwist stream position. The trace also does not yet
attach fighter, phase, source line, action/timer, branch guards or loop values.
The header says so explicitly rather than inventing those values.

The exporter writes after the random result is obtained and leaves the existing
LongInt/Int64 overload choice intact. It does not draw extra random values, reset the
RNG, split streams, or change record/replay order. File I/O errors disable the trace
and are reported through a `.error.txt` sidecar; they are not allowed to replace a
game result.

The schema-v1 Python validator and regression tests remain developer tooling
in this Rust branch. A Rust library joiner is also provided in
`src/trace_join.rs`; the Python validator has not yet been ported to Rust.

The schema validator and regression tests live in this Rust branch:

```sh
python3 tools/validate-pascal-rng-jsonl.py saida/rng.jsonl
python3 -m unittest discover -s tools -p 'test_*.py' -v
```

Do not promote a trace to a reference fixture until the Pascal branch builds with
Lazarus 4.8/FPC 3.2.2 and two identical fixed-seed runs produce byte-identical
JSONL plus matching `logica.txt` checkpoints.

## 🌈 Joining symbolic and dynamic traces

A reconciliation tool can join a dynamic JSONL draw with the static normalizer using:

- pinned source commit and file path;
- source line/column;
- script section and fighter;
- range expression and runtime range value;
- LongInt/Int64 width;
- nested guards and enclosing loops;
- symbolic P.Acao/P.QuadroAcao predicates.

Static normalizer event sequence is not a runtime draw index. The runtime log's api_call_index and first_mt_word_index are authoritative for a concrete run.

For repeated loops, the dynamic trace should attach each iteration's loop variable values. For nested branches, preserve the actual guards that evaluated true. This produces an executable per-action/per-tick trace rather than an ordered list of source statements.

## 🌈 Validation sequence

1. Build the pinned Pascal project with Lazarus 4.8/FPC 3.2.2 and enable the optional trace.
2. Run --gravar with RandSeed 20261026 and preserve logica.txt, screenshots, and JSONL RNG records.
3. Repeat the same run and require byte-identical RNG JSONL and logic checkpoints.
4. Use ModoDeSorteio(msGravar/msRepetir) to compare selected old-code/script routines, retaining conferencia.log.
5. Build Rust replay fixtures from captured values; verify draw index, width, range, returned value, and final state fields.
6. Only later compare these traces with Chaves.exe using the separate REA/Ghidra compatibility stage.

## 🌈 Provenance

- [ChavesSDL.pas deterministic --gravar harness](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/ChavesSDL.pas)
- [ChavesRoteiro.pas Random recorder/replayer](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/game/ChavesRoteiro.pas)
- [ChavesConferencia.pas old-code/script differential harness](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/game/ChavesConferencia.pas)
- [Pinned FPC 3.2.2 MTwist implementation](https://github.com/fpc/FPCSource/blob/release_3_2_2/rtl/inc/system.inc)

 
## 🌈 Schema v2: concrete runtime context (verified Pascal capture)

The schema-v1 description above is historical. The Pascal research branch
`research/sortearvalor-jsonl-v1` implements schema v2. It compiles and runs
against the pinned Pascal project with FPC 3.2.2 and SDL2; it remains an
unmerged experiment, not a change to upstream `main`.

The v2 stream contains:

| Record | Runtime meaning |
|---|---|
| `session` | Seed, session identity, pinned Pascal revision, and explicit RNG scope |
| `tick_start` / `tick_end` | Zero-based `TJogo.PassoSDL` step, whether logic ran, game state, selected fighter/action/frame snapshots |
| `script_start` / `script_end` | One concrete `.rot` invocation, unique ID, parent ID for nested calls, origin/section, fighter, character, command, action/frame snapshots, draw count |
| `control` | Actual `if` branch result, selected `case` arm, and concrete `for`/`while`/`repeat` iteration values and counts |
| `draw` | Existing RNG request fields plus active tick/script, fighter/action/frame, source origin/line, and per-script draw index |

Three sequence counters are deliberately separate:
`tick_index` counts logical game steps, `api_call_index` orders calls through
`SortearValor`, and `script_draw_index` counts draws in one particular script
invocation. Nested `.rot` invocations have their own IDs and local draw counters.

The runtime source line joins a concrete draw/control event to a static
normalizer event's file, section and line. The Rust joiner is implemented and
preserves `unique`, `ambiguous`, `unmatched` and `non-ROT source` outcomes.
The Pascal interpreter does not yet emit source columns, so same-line events
may be ambiguous. Control
records expose branch decisions and loop values at interpreter control nodes;
they do not expose every short-circuited boolean subexpression. Tick snapshots
cover selected action/timer fields, not every byte of game memory.

The wrapper-scoped MT-word counter remains unchanged: direct `System.Random`
calls outside `SortearValor` are still excluded. It is not a global MTwist
position.

**Verified capture:** GitHub Actions run
[Pascal runtime trace #45040091460](https://github.com/danimesq/StreetChavesRust/actions/runs/45040091460)
compiled the Pascal branch with FPC 3.2.2 and SDL2, ran the fixed-seed
`--gravar` scenario twice (3,000 steps each), compared both JSONL files and
both `logica.txt` checkpoints byte for byte, validated both schema-v2 traces,
and uploaded artifact `pascal-runtime-trace-schema-v2`.

Each run contains 3,000 ticks, 20,437 script invocations, 394,252 control
records, 8,841 `SortearValor` calls (8,841 `System.Random`, 0 replay), and
14,895 wrapper-scoped MT words. The overload distribution is 2,787 LongInt
draws and 6,054 Int64 draws, matching the observed 14,895-word total. Both
trace-enabled runs are identical. Their `logica.txt` files also match the
third fixed-seed `--gravar` baseline run with JSONL tracing disabled. This
establishes that enabling the logger did not alter recorded logic checkpoints
in this scenario; it is not a full memory-state comparison or original-game
parity proof. The full trace artifact is retained by Actions until
2026-12-22. A compact, selected real-event excerpt is committed at
`tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl`; it is marked as an
event sample, not a complete tick trace.

`src/trace_join.rs` exposes the reconciliation API, and
`src/bin/trace-join.rs` provides a full-capture CLI:

```sh
cargo run --release --bin trace-join -- trace/rng.jsonl symbolic-action-tick-index.json reconciliation.jsonl
```

The end-to-end pipeline in
[Pascal runtime trace #45040091460](https://github.com/danimesq/StreetChavesRust/actions/runs/45040091460)
recompiled Pascal, repeated both fixed-seed captures, regenerated the 15
fighter symbolic index from the pinned Pascal source, reconciled every draw
in both traces, and confirmed byte-identical reports. The full-capture result:

| Classification | Count |
|---|---:|
| Unique static site match | 8,415 |
| Ambiguous | 0 |
| Unmatched | 0 |
| Non-`.rot` source context | 426 |

The 426 non-`.rot` calls are further identified as 422 direct calls through
`SortearValor` with no active script context and 4 calls from Pascal helpers
inside a `.rot` execution. They are real wrapper draws, not failed static
matches. All 8,415 `.rot` expression draws join uniquely under the current
key (fighter/character, file/section, line, width, and literal range where
available). The index contains 465 fighter-timeline RNG occurrences from
255 unique static source sites.

The captured first draw still joins to `Common.IA.L293.C4`. The tests also
prove that an artificially duplicated same-line candidate remains ambiguous
and that helper draws are not mislabelled as static `.rot` sites.

Rust CI run
[contract tests #45038903679](https://github.com/danimesq/StreetChavesRust/actions/runs/45038903679)
passes formatting and all 22 library tests plus both integration tests.
The full capture/reconciliation evidence is in
[artifact #14761022549](https://github.com/danimesq/StreetChavesRust/actions/runs/45040091460/artifacts/14761022549);
the integrated pipeline passed.


## 🌈 Rust replay adapter: first real-draw consumer

The Rust crate now exposes `pascal_replay::PascalDrawReplay`. It consumes
schema-v2 `draw` records in their original `api_call_index` order and returns
the captured result only when the caller supplies the exact Pascal overload
(`LongInt` or `Int64`) and range. A mismatch does not advance the cursor.
The adapter rejects non-sequential API indexes, unknown widths, invalid
result ranges, and incorrect raw MT-word costs. `finish()` fails when the
simulator exits while recorded draws remain, making early/extra branch
divergence observable.

The first regression test uses the committed, verbatim real event sample
`tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl`: its captured
`LongInt` request for range 13 must replay as 12. Additional tests verify
width/range mismatch, cursor preservation, exhaustion, and unconsumed draws.

This is deliberately a **recorded-result replay adapter**, not yet a complete
Rust game simulation and not a second MTwist implementation. It proves that a
future simulation can consume real Pascal RNG decisions without inventing
fixture values. The adapter currently consumes the wrapper-scoped
`SortearValor` stream; direct `System.Random` calls outside that wrapper are
still outside the trace contract. Full tick/state transition comparison and
multi-fighter/action fixtures remain the next integration work.


## 🌈 Executable tick differential: first-divergence comparator

The Rust crate now exposes `pascal_differential::load_tick_checkpoints` and
`first_divergence`. They parse the Pascal harness's concrete `tick_start`
and `tick_end` records and compare the selected fighter/action/frame fields
in ascending tick order. A missing checkpoint is itself a divergence. The
comparator reports the first differing tick, phase, field, and both values.

A CLI is available for a Rust simulator that emits the same checkpoint JSONL
shape:

```sh
cargo run --release --bin pascal-diff -- trace/rng.jsonl rust-checkpoints.jsonl
```

The first version compares only fields actually present in the Pascal
schema-v2 snapshots. In particular, `tick_end` does not currently carry
action-duration fields, and neither phase is a full memory dump. A matching
report therefore means equality of these emitted checkpoint fields, not
complete game-state parity.

The replay adapter also exposes width-specific
`random_longint(i32)` and `random_int64(i64)` entrypoints. A simulator can
call these at the corresponding RNG sites and gets a strict error on the
first unexpected width/range, exhausted stream, or leftover draw at
`finish()`. This is the execution seam; the repository does not yet contain
a complete action runner that drives the existing combat state model through
all 3,000 captured ticks.

The recovered Actions artifact contains the complete 3,000-tick trace:
8,841 wrapper draws (2,787 LongInt and 6,054 Int64), 394,252 control
records, and 20,437 script invocations. The compact committed tick-316 sample
remains the only checked-in raw draw fixture at this stage. The full artifact
is retained at Actions artifact 14761022549 until 2026-12-22; do not treat
the sample as if it contained the full run.


The Pascal capture workflow now runs an additional Rust regression against the
same freshly generated full JSONL file. It feeds every captured request into
the independent `Fpc322Rng` implementation and checks the returned value,
overload width, request range, API call index, first MT-word index, and
one-versus-two-word cost. The expected totals are 8,841 calls and 14,895 raw
MT words. This test is stronger than recorded-result replay: it independently
recalculates the MTwist sequence from the fixed seed. It still validates the
RNG stream, not the complete combat/action simulation.
