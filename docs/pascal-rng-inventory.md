# 🌈 Pascal RNG call-site inventory

## 🌈 Scope

This inventory is generated from the pinned Pascal source and the 15 fighter timelines. It is a static source inventory, not an execution trace.

## 🌈 Results from the first complete normalization

| Metric | Count |
|---|---:|
| Unique `.rot` Random sites | 255 |
| ├─ LongInt / 1 raw MT word | 160 |
| └─ Int64 / 2 raw MT words | 95 |
| Occurrences across 15 composed fighter timelines | 465 |
| ├─ LongInt / 1 raw MT word | 244 |
| └─ Int64 / 2 raw MT words | 221 |
| RNG-related references/calls in Pascal `.pas` and `.inc` | 71 |

The 465 timeline occurrences include each fighter's execution of the shared `Comum.rot` section. The 255 unique sites are deduplicated by path, section, line and column.

The 71 Pascal source references cover initialization, the interpreter's replay wrapper, game state, camera, stage events, frame presentation, fight logic, HUD and final/victory presentation. The exact per-file call list is emitted in `pascal-rng-call-sites.json` in the CI artifact.

## 🌈 FPC 3.2.2 consumption model

The project README pins Lazarus 4.8 with Free Pascal 3.2.2. The matching RTL implements MTwist.

| Pascal expression width | API overload | Raw MT words |
|---|---|---:|
| ordinary LongInt | `System.Random(LongInt)` | 1 |
| Int64, e.g. `Random(Round(...))` | `System.Random(Int64)` | 2 |

A static call site therefore carries its width and expected raw-word cost. The normalizer includes `random_width`, `raw_rng_words_consumed`, and `rng_result_reduction` for each `Random` event.

## 🌈 Reproducible source traces

The existing Pascal `ModoDeSorteio(msGravar/msRepetir)` records and replays requests by range and width. The game also has a deterministic `--gravar` harness that sets `RandSeed := 20261026` and emits `logica.txt`.

The Pascal research branch
[research/sortearvalor-jsonl-v1](https://github.com/danimesq/OpenStreetChaves/tree/research/sortearvalor-jsonl-v1)
now emits verified schema-v2 JSONL: ticks, nested `.rot` executions, control
decisions/loop values, source provenance, and `SortearValor` draws. Two
3,000-step fixed-seed runs compiled with FPC 3.2.2 and produced byte-identical
JSONL and `logica.txt` checkpoints. Direct `System.Random` calls outside
`SortearValor` remain excluded, so the word cursor is still wrapper-local.
The Pascal branch remains unmerged and the normative source pin unchanged.
See [Pascal reference trace harness](pascal-reference-trace-harness.md).

The latest normalizer CI run #30 passes 7 grammar tests and verifies the 15-fighter bundle. The current source normalizer does not assign dynamic tick or raw-word indices. It preserves the static call graph and symbolic execution guards, including a measured function-argument evaluation profile. The interpreter's own OrdemArgsInversa probe is reproduced by tools/fpc-argument-order-probe.pas; in the FPC 3.2.2 x86_64 Linux CI build, run #42 reported right-to-left (evaluation trace 5431). The normalizer now emits multi-argument call events in that measured order for this build. `symbolic-action-tick-index.json` groups all 5,091 events into 1,749 groups by symbolic P.Acao / P.QuadroAcao guard contexts and preserves all other guards and loop contexts. These are symbolic lanes, not actual executed ticks.

## 🌈 Precedence rule

Pascal/FPC 3.2.2 is the normative source for the Rust implementation. C-port call sites and RNG differences remain comparison annotations. The original `Chaves.exe` may later provide evidence for a documented behavior change, but the C port does not silently override Pascal.

The existing runtime capture and differential mechanisms are mapped in [Pascal reference trace harness](pascal-reference-trace-harness.md).

See [RNG compatibility](rng-compatibility.md), [ROT normalizer](rot-normalizer.md), and [C call-site comparison](rng-callsite-inventory.md).

 
## 🌈 Dynamic trace status: schema v2

The Pascal exporter compiled with FPC 3.2.2 and passed two deterministic
3,000-step captures. Each contains 3,000 ticks, 20,437 script executions,
394,252 control records, 8,841 `SortearValor` requests and 14,895
wrapper-scoped MT words. Both JSONL files and both `logica.txt` outputs
matched byte for byte; both traces passed the schema-v2 validator.

A real tick-316 excerpt is committed as a regression fixture. The Rust
`trace_join` module joins its first `Comum.rot [IA]` draw (line 293,
LongInt, range 13) to the static site `Common.IA.L293.C4` as a unique match.
The joiner preserves ambiguous and unmatched results rather than inventing a
source column. The full 8,841-draw capture has not yet been reconciled
end-to-end against all 15 symbolic timelines.

Source columns, every short-circuited boolean subexpression, direct
`System.Random` calls outside the wrapper, and full game-state snapshots
remain outside schema v2. The MT-word index is not a global MTwist cursor.
The experimental Pascal branch remains unmerged.
