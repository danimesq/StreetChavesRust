# Executable Pascal-derived AI runtime slice

## Scope

The Rust runtime now executes a bounded portion of `Dados/Lutadores/Comum.rot [IA]`, including the target-selection branch, movement decision and close-range probability chain. It is still not a general `.rot` interpreter or a complete combat tick.

The implemented source order is:

1. Request `Random(13)` as Pascal `LongInt`.
2. When its result is zero, request `Random(6)` as `LongInt` and select one of the six `P.DistanciaIA` targets (100, 150, 200, 250, 300 or 350).
3. Apply `Abs(Dist - P.DistanciaIA) <= 50` and update the target distance when the condition holds.
4. Evaluate the source-ordered direction/facing decisions and invoke the corresponding movement command.
5. When `not P.Defendendo`, `Ameaca = 0` and `Dist <= 80`, execute the four independent attack probabilities followed by the conditional crouch probability and its four nested attack probabilities.
6. Execute the common `[Acao]` command path, an injected character-specific action-script callback, and common `[AcaoFim]` in order.
7. Emit Rust-owned checkpoints from the calculated state.

The same mutable `PascalRandomSource` is passed into the child-script callback. A recording adapter collects requests from both parent and child without making a new cursor.

## Pascal source references

Reference source is the merged OpenStreetChaves Pascal implementation:

- `Dados/Lutadores/Comum.rot`, section `[IA]`, lines 293-323: target selection, distance update, movement, and close-range attack probabilities.
- Same file, section `[Acao]`: guards and state writes for movement, basic punches/kicks, crouch, and airborne attacks.
- Same file, section `[AcaoFim]`: walking animation frame increment and wrap after 16.
- `source/game/ChavesGolpes.pas`, `AcaoJogador`: common `[Acao]`, character-specific `[Acao]`, then common `[AcaoFim]`.
- Same unit, `Empurra`: movement, body overlap separation, and live-fighter screen bounds.
- `source/game/Estado7_Luta.inc`: game loop phase ordering and physics integration.

## Real Pascal RNG evidence

The captured traces are real draw segments from the deterministic 3,000-tick Pascal run, not generated test values:

### Tick 316

Fixture: `tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl`.

- One `LongInt Random(13)`, result 12.
- The nonzero branch is taken.
- The selected-event fixture also contains the Pascal tick-start and tick-end action/frame checkpoints.

This is the only fixture currently used for a complete two-checkpoint differential match.

### Tick 317

Fixture: `tests/fixtures/pascal-runtime-ai-target-tick317.jsonl`.

- Global API calls 94-95.
- `LongInt Random(13) = 0`, followed by `LongInt Random(6) = 2`.
- The observed selector result maps to target distance 200.

The fixture preserves the absolute global API indices and only includes this contiguous two-draw segment. It does not export the numeric `Dist` or the preexisting `P.DistanciaIA`, so the runtime scenario's remaining inputs are explicit assumptions.

### Tick 381

Fixture: `tests/fixtures/pascal-runtime-ai-close-range-tick381.jsonl`.

The real Common IA draw segment is global API calls 423-432:

| Source line | Width | Range | Result |
|---:|---|---:|---:|
| 293 | LongInt | 13 | 7 |
| 312 | Int64 | 29 | 25 |
| 313 | Int64 | 29 | 0 |
| 314 | Int64 | 32 | 1 |
| 315 | Int64 | 32 | 8 |
| 316 | Int64 | 14 | 0 |
| 319 | Int64 | 7 | 3 |
| 320 | Int64 | 7 | 3 |
| 321 | Int64 | 11 | 6 |
| 322 | Int64 | 11 | 5 |

The Rust regression asserts exact width, request range, result, and ordering for all ten requests. It also asserts the commands selected by this scenario.

The probability ranges are compatible with a difficulty factor of 3.6 because Pascal `Round` gives 29, 32, 14, 7 and 11 for the corresponding multipliers. The trace records the rounded ranges, not the source floating-point factor itself. Therefore 3.6 is a convenient scenario input, not a captured or uniquely proven value.

## RNG cursor across nested character actions

`CharacterActionScript` is the explicit child-execution seam corresponding to the character-specific `[Acao]` called by Pascal `AcaoJogador`. The child receives the same mutable RNG source used by the parent IA runtime.

A deterministic Rust integration test has a fake child consume one additional draw after the parent `Random(13)` request. The test asserts that:
- the child receives the next request from the same cursor;
- the parent and child draws appear in one ordered runtime draw list;
- the cursor advances twice, with no reset.

Evidence boundary: the existing full Pascal 3,000-tick capture includes nested script invocations, but none of its recorded RNG draws is emitted from a nested child-script context. The new test validates the Rust cursor-sharing architecture; it does not claim that the original capture contains an observed child-script RNG call. Real character-specific child scripts remain to be ported and captured.

## Missing historical inputs

The selected event samples and current full Pascal trace do not contain the numeric values needed to reconstruct all state at ticks 316, 317 or 381. In particular, the current trace does not export:

- IA inputs `Dist`, `DistProj`, `OppY`, `Ameaca` and `Fator` alongside the script start;
- `P.DistanciaIA`, `P.Lado`, `P.Defendendo`, `P.VelY`, `P.NoAr`;
- fighter positions and body collision boxes for both fighters at IA entry.

The runtime tests use explicitly documented scenario inputs to exercise the recorded control-flow and draw sequences. They do not claim those hidden inputs match the historical ticks. A trace-only Pascal instrumentation follow-up is needed to export them; no gameplay logic needs to be changed.

## Runtime boundaries

Implemented:
- Both `Random(13)` branches, including the `Random(6)` target selector.
- The common AI movement predicates.
- All ten possible close-range probability requests in source order, with the four crouch follow-up draws conditional on the fifth result.
- Pascal `LongInt` and `Int64` request widths and ranges.
- Ties-to-even rounding for probability limits.
- Common action state transitions for the basic movement, punch/kick, crouch, and airborne commands used by this slice.
- A child-script injection seam with one shared RNG cursor.
- Strict captured-draw replay, unconsumed-draw detection, and Rust-generated checkpoints.

Still outside scope:
- A general Pascal `.rot` interpreter.
- Porting the actual character-specific `CNN.rot [Acao]` and `CNN.rot [IA]` scripts.
- Full state snapshots and historical hidden-input validation.
- All secondary effects of `AcaoJogador` such as every charge/sound/effect/global write.
- Player input, full frame scripts, hit resolution, effects, and complete physics.
- A complete tick-level equivalence claim against the original executable.

Unsupported semantics must fail explicitly. Never insert synthetic draws or overwrite calculated Rust state with captured Pascal state merely to force a differential match.

## Reproduction

From the repository root:

```sh
cargo test --all-targets

cargo run --bin pascal-replay-tick -- tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl \
  > /tmp/rust-tick316.jsonl

cargo run --bin pascal-diff -- tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl \
  /tmp/rust-tick316.jsonl
```

The tick-316 comparison is expected to report `MATCH: 2 Pascal/Rust tick checkpoints compared`. Tick 317 and 381 are RNG-path regressions, not complete state differential matches because their fixtures deliberately contain draw segments only.

## Next work

1. Add trace-only Pascal IA input/state snapshots so historical hidden inputs are actually captured.
2. Port real character-specific `[Acao]` scripts that can run under the shared RNG source.
3. Find or create a Pascal capture where a child script itself consumes RNG, then compare that nested draw sequence with Rust.
4. Expand state checkpoints and action semantics incrementally; keep each new equivalence claim tied to an actual capture.
