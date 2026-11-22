# Executable Pascal-derived AI runtime slice

## Scope

The Rust runtime executes a bounded portion of `Dados/Lutadores/Comum.rot [IA]`, including target selection, movement and the close-range probability chain. It is not a general `.rot` interpreter or a complete combat tick.

The implemented source order is:

1. Request `Random(13)` as Pascal `LongInt`.
2. If the result is zero, request `Random(6)` as `LongInt` and select one of six `P.DistanciaIA` targets (100, 150, 200, 250, 300 or 350).
3. Apply `Abs(Dist - P.DistanciaIA) <= 50`.
4. Evaluate the four source-ordered distance/facing predicates.
5. When `not P.Defendendo`, `Ameaca = 0` and `Dist <= 80`, execute the four independent probability requests followed by the conditional crouch request and its four nested probability requests.
6. Execute common `[Acao]`, the character-specific action-script callback, then common `[AcaoFim]`.
7. Emit Rust-owned checkpoints from the calculated state.

The same mutable `PascalRandomSource` is passed into the child-script callback. A recording adapter collects requests from parent and child without creating a second cursor.

## Pascal source references

Reference source: merged OpenStreetChaves Pascal implementation.

- `Dados/Lutadores/Comum.rot`, section `[IA]`, lines 293-323: target selection, distance update, movement and close-range attack probabilities.
- Same file, section `[Acao]`: movement, basic punches/kicks, crouch and airborne attack state writes.
- Same file, section `[AcaoFim]`: walking animation frame increment and wrap after 16.
- `source/game/ChavesGolpes.pas`, `AcaoJogador`: common `[Acao]`, character-specific `[Acao]`, then common `[AcaoFim]`.
- Same unit, `Empurra`: movement, body-overlap separation and live-fighter screen bounds.
- `source/game/Estado7_Luta.inc`: game-loop phase ordering and physics integration.

## Real Pascal capture inputs

OpenStreetChaves PR #3 adds trace-only `ai_inputs` metadata to the `script_start` record for every `.rot [IA]` execution. The data is captured at actual interpreter entry, not reconstructed from later RNG draws. The Rust fixtures now contain those snapshots and the tests parse them directly to build both fighters and all five IA inputs.

| Tick | `Dist` | `DistProj` | `OppY` | `Ameaca` | `Fator` | IA `P.DistanciaIA` at entry | IA facing |
|---:|---:|---:|---:|---:|---:|---:|---|
| 316 | 132 | 0 | 300 | 0 | 3.6 | 0 | left |
| 317 | 124 | 0 | 300 | 0 | 3.6 | 0 | left |
| 381 | 78 | 0 | 300 | 0 | 3.6 | 300 | left |

The snapshots also include both fighters' character/action/frame/duration, facing, defending state, X/Y, vertical velocity, airborne state, speed, health and body box. Tick 316, for example, records the IA fighter at X=366 with body box `[-19,35,13,164]` and the opponent at X=234 with body box `[-18,30,13,170]`.

The snapshot does not yet include `CameraX` or `RelogioParado`. These selected scenarios do not enter screen-edge or paused-clock overlap resolution; those globals must be captured before expanding equivalence claims to those paths.

### Tick 316

Fixture: `tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl`.

- Captured state and all five IA inputs.
- One `LongInt Random(13)`, result 12, so the zero branch is not taken.
- The Rust runtime moves the IA fighter left by 8 pixels, enters action 2, frame 1.
- Rust emits its own `tick_start`/`tick_end` checkpoints. `pascal-diff` confirms the two available Pascal checkpoints match.

### Tick 317

Fixture: `tests/fixtures/pascal-runtime-ai-target-tick317.jsonl`.

- Global API calls 94-95.
- `LongInt Random(13) = 0`, followed by `LongInt Random(6) = 2`.
- The selector sets target distance to 200. Since `Dist=124` and the difference is greater than 50, the target is retained.
- The captured IA fighter faces left and moves right, from X=358 to X=366.

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

The captured `Fator=3.6` explains the rounded Int64 ranges. The second top-level probability succeeds and selects a weak kick. The fifth succeeds and selects crouch; the four conditional follow-up requests are still consumed, but their results do not select additional attacks. The actual fighter ends the IA script in action 4, frame 0. Later physics advances the full tick frame, so this slice compares the AI-script result, not the complete tick-end state.

## RNG cursor across nested character actions

`CharacterActionScript` is the child-execution seam corresponding to the character-specific `[Acao]` invoked by Pascal `AcaoJogador`. The child receives the same mutable RNG source used by the parent IA runtime.

A deterministic Rust integration test uses the actual strict `PascalDrawReplay` implementation with a two-draw JSONL stream. The first draw is consumed by the parent `Common.rot [IA]` executor; the second is tagged with a child script execution ID and `parent_script_execution_id` pointing to the parent. An injected child executor consumes that second draw. The test asserts that:
- the child receives the next draw from the same replay cursor;
- parent and child draws appear in one ordered runtime draw list;
- `draws_consumed()==2`, no draws remain, and `finish()` succeeds.

Evidence boundary: the 3,000-tick Pascal capture contains nested script invocations but none of its captured RNG draws has a child-script parent context. The Rust regression therefore validates shared-cursor replay behavior with a deterministic parent/child trace fixture; it does not claim that historical gameplay has yet demonstrated RNG consumption inside a real character-specific child script. Actual `CNN.rot [Acao]` scripts remain to be ported and observed.

## Runtime boundaries

Implemented:
- Both `Random(13)` branches and the `Random(6)` selector.
- The source-ordered AI movement predicates.
- All ten potential close-range probability requests, with the four crouch follow-ups conditional on the fifth draw.
- Pascal `LongInt` and `Int64` widths and ranges.
- Ties-to-even rounding for probability limits.
- Common action state transitions for the basic movement, punch/kick, crouch and airborne commands used by this slice.
- Child-script injection with one shared RNG cursor.
- Snapshot-driven scenario reconstruction, strict captured-draw replay, unconsumed-draw detection and Rust-generated checkpoints.

Still outside scope:
- A general Pascal `.rot` interpreter.
- Porting the actual character-specific `CNN.rot [Acao]` and `CNN.rot [IA]` scripts.
- `CameraX`, `RelogioParado` and other global-state capture needed for screen-edge/paused-clock paths.
- Every secondary effect of `AcaoJogador`, such as all charge, sound, effect and global writes.
- Player input, full frame scripts, hit resolution, effects and complete physics.
- Complete tick-level equivalence against the original executable.

Unsupported semantics must fail explicitly. Never insert synthetic draws or overwrite calculated Rust state with captured Pascal state to force a differential match.

## Reproduction

```sh
cargo test --all-targets

cargo run --bin pascal-replay-tick -- tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl \
  > /tmp/rust-tick316.jsonl

cargo run --bin pascal-diff -- tests/fixtures/pascal-runtime-schema-v2-tick316.jsonl \
  /tmp/rust-tick316.jsonl
```

The tick-316 comparison is expected to report `MATCH: 2 Pascal/Rust tick checkpoints compared`. Tick 317 and 381 currently validate real RNG order and their selected IA-script outcomes; only tick 316 is a full two-checkpoint differential regression in the current harness.

## Next work

1. Port real character-specific `[Acao]` scripts and verify their state mutations.
2. Find or create a Pascal capture where a child script itself consumes RNG, then compare that nested draw sequence with Rust.
3. Add camera/clock and remaining global inputs needed for collision and edge cases.
4. Expand state checkpoints and action semantics incrementally; tie each equivalence claim to a real capture.
