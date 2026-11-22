# First Rust runtime slice: Pascal tick 316

## Scope

This is the first executable vertical slice in the Rust reconstruction. It is deliberately not a general Pascal `.rot` interpreter or a complete combat tick.

The slice executes one concrete path:
1. Enter `Comum.rot [IA]` for fighter 2.
2. Request `Random(13)` through the shared `PascalRandomSource` interface.
3. Replay the captured tick-316 result (LongInt range 13, value 12).
4. Take the nonzero branch at source line 293.
5. Evaluate the source-ordered distance/facing predicates.
6. Invoke the common left/right movement action.
7. Apply `Empurra` movement, body-overlap separation, and live-fighter screen clamps.
8. Apply `Comum.rot [Acao]` movement state writes and `[AcaoFim]` walk-frame increment.
9. Emit independent Rust `tick_start`/`tick_end` checkpoint JSONL.
10. Compare those checkpoints against the Pascal tick-316 sample using the existing differential parser.

## Pascal source path

Pinned reference: OpenStreetChaves branch `research/sortearvalor-jsonl-v1`.

- `Dados/Lutadores/Comum.rot`, section `[IA]`, line 293: `if Random(13) = 0 then`.
- Same section: distance-target adjustment and four direction/facing predicates.
- `Dados/Lutadores/Comum.rot`, section `[Acao]`: movement commands are accepted only from `acParado` or `acAndando`; `Empurra` runs before the action becomes `acAndando`; duration becomes 99.
- Same file, section `[AcaoFim]`: walking frame increments and wraps after 16.
- `source/game/ChavesGolpes.pas`, `AcaoJogador`: common [Acao], fighter-specific [Acao], then common [AcaoFim].
- Same unit, `Empurra`: world movement, body-overlap separation, and live-fighter screen bounds.
- `source/game/Estado7_Luta.inc`, `Estado7_Luta`: CPU/IA runs before fighter drawing and later physics. `L7_Fisica` does not increment a duration-99 action frame.

## Real capture evidence

Fixture: `tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl`.

The sample records:
- Tick 316, game state 7.
- Fighter 2 / character 1 begins action 1, frame 0, duration 99.
- `Comum.rot [IA]` requests LongInt `Random(13)`; captured result is 12.
- The branch at source line 293 is false.
- The script end records action 2, frame 1.
- The tick-end checkpoint independently records fighter 2 action 2, frame 1.

The fixture is a selected event sample, not the complete tick trace. It does not include all IA inputs such as `Dist`, `P.DistanciaIA`, facing, or body geometry.

## Explicit test assumptions

To exercise the observed nonzero branch and reproduce the recorded action/frame checkpoint, the test supplies a bounded set of admissible values not present in the sample:
- `Dist = 150`, `P.DistanciaIA = 250` (difference exceeds 50, so no target overwrite).
- Fighter 2 faces right; it is not defending and has no projectile threat.
- `Dist > 80`, so the close-range attack probability chain is not entered.
- The fighters' body boxes do not overlap; the walk displacement is 3 pixels and remains inside the screen bounds.

These values select the source-defined left-walk branch and match the observed end checkpoint. They are not claimed to be the hidden original tick-316 inputs. The differential match validates this bounded path against the available checkpoint fields, not complete original-state equivalence.

## Runtime boundary

Implemented:
- Width-aware replay call through `PascalRandomSource`.
- Exact request validation and full draw consumption via `PascalDrawReplay::finish`.
- The captured `Random(13) != 0` branch.
- Source-ordered distance/facing decision for walking.
- The common walking command's action/duration/frame changes.
- Pascal `Empurra` overlap and screen-clamp behavior.
- Rust-generated checkpoints and first-divergence comparison.

Fail-closed branches:
- `Random(13) = 0`, which enters the subsequent `Random(6)` distance-target selection.
- Close-range attack selection at `Dist <= 80`, including its multiple probability draws and nested commands.
- Character-specific IA and any unimplemented script operations.

No synthetic draws are consumed to force alignment. Unsupported control flow returns an explicit error.

## Reproduction

From the repository root:

```sh
cargo test --all-targets

cargo run --bin pascal-replay-tick -- tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl \
  > /tmp/rust-tick316.jsonl

cargo run --bin pascal-diff -- tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl \
  /tmp/rust-tick316.jsonl
```

Expected differential result: `MATCH: 2 Pascal/Rust tick checkpoints compared`.

## Next expansion

1. Capture a full Pascal tick with all AI inputs and state fields needed to remove the bounded-input assumptions.
2. Implement the `Random(13) = 0` / `Random(6)` branch with its exact source semantics.
3. Implement the close-range AI probability chain while retaining one shared RNG cursor through nested `AcaoJogador` calls.
4. Expand from the AI action path into player input, frame scripts, hit resolution, effects, and physics in the documented Pascal phase order.
5. Widen checkpoint coverage only when the Pascal exporter and Rust runtime both produce the corresponding fields.