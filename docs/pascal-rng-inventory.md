# 🌈 Pascal RNG call-site inventory

## 🌈 Scope

This inventory is generated from the pinned Pascal source and the 15 fighter timelines. It is a static source inventory, not an execution trace.

## 🌈 Results from the first complete normalization

| Metric | Count |
|---|---:|
| Unique `.rot` Random sites | 255 |
| Occurrences across 15 composed fighter timelines | 465 |
| LongInt-width occurrences | 244 |
| Int64-width occurrences | 221 |
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

Next work should use that harness to capture a reproducible stream with:
- phase/system identifier;
- fighter and section;
- source line and call site;
- LongInt vs Int64 overload;
- raw MT word start index and number of words consumed;
- requested range and returned value;
- branch and loop context.

The current source normalizer does not assign these dynamic values. It preserves only the static call graph and symbolic execution guards.

## 🌈 Precedence rule

Pascal/FPC 3.2.2 is the normative source for the Rust implementation. C-port call sites and RNG differences remain comparison annotations. The original `Chaves.exe` may later provide evidence for a documented behavior change, but the C port does not silently override Pascal.

See [RNG compatibility](rng-compatibility.md), [ROT normalizer](rot-normalizer.md), and [C call-site comparison](rng-callsite-inventory.md).
