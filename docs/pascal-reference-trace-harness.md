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

## 🌈 Experimental JSONL exporter (upstream branch)

A first opt-in exporter has been implemented on the experimental
[OpenStreetChaves branch](https://github.com/danimesq/OpenStreetChaves/tree/research/sortearvalor-jsonl-v1),
based on the exact Pascal source revision pinned above:
`e1b025c35ecbc2b14d8635564aafd23c2107d3d1`. It is not merged into upstream `main`,
and this Rust research branch does not change the normative source pin yet.

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
