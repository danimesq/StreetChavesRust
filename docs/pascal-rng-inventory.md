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

An experimental exporter now writes JSONL from `SortearValor` on the upstream
[research branch](https://github.com/danimesq/OpenStreetChaves/tree/research/sortearvalor-jsonl-v1).
It captures the request index, LongInt/Int64 width, range/result, record/replay mode,
RandSeed snapshots and wrapper-local raw-word count. It is not yet a global RNG trace:
direct `System.Random` sites outside `SortearValor` are excluded, and action/tick,
source and loop context are not yet attached. The branch is not merged into upstream
`main`; the normative source pin therefore remains unchanged. See
[Pascal reference trace harness](pascal-reference-trace-harness.md) for the schema,
command line and validation gate.

The latest normalizer CI run #30 passes 7 grammar tests and verifies the 15-fighter bundle. The current source normalizer does not assign dynamic tick or raw-word indices. It preserves the static call graph and symbolic execution guards, including a measured function-argument evaluation profile. The interpreter's own OrdemArgsInversa probe is reproduced by tools/fpc-argument-order-probe.pas; in the FPC 3.2.2 x86_64 Linux CI build, run #42 reported right-to-left (evaluation trace 5431). The normalizer now emits multi-argument call events in that measured order for this build. `symbolic-action-tick-index.json` groups all 5,091 events into 1,749 groups by symbolic P.Acao / P.QuadroAcao guard contexts and preserves all other guards and loop contexts. These are symbolic lanes, not actual executed ticks.

## 🌈 Precedence rule

Pascal/FPC 3.2.2 is the normative source for the Rust implementation. C-port call sites and RNG differences remain comparison annotations. The original `Chaves.exe` may later provide evidence for a documented behavior change, but the C port does not silently override Pascal.

The existing runtime capture and differential mechanisms are mapped in [Pascal reference trace harness](pascal-reference-trace-harness.md).

See [RNG compatibility](rng-compatibility.md), [ROT normalizer](rot-normalizer.md), and [C call-site comparison](rng-callsite-inventory.md).
