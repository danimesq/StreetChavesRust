# Headless FightSimulation

## Status

The Rust core has a headless `SimulationTick` implementation: `FightSimulation`. It now runs the common `[Quadro]` section, the character frame path, and source-ordered box selection without creating a window, renderer, SDL dependency, browser API or WASM binding.

## Tick order

For each 28 ms call from `FixedStepRunner`:

1. Receive the immutable `InputFrame` and retain it in the tick report.
2. Dispatch input and queued actions when the fight is not hit-stopped or screen-flashing.
3. For each fighter, run common `[Quadro]` from `Comum.rot`.
4. Run the character-specific `[Quadro]` path. C01's already-tested special/victory/pose branches retain their bounded hand-port; other mapped character frame sections execute through ROT.
5. After both frame sections succeed, apply the current hurtbox and all matching attack rows using Pascal's source-ordered `AplicarCaixas` semantics.
6. Advance the live `EffectArena` exactly once and retain its ordered render snapshots.
7. Increment the fight tick and publish a `FightTickReport`.

The frame scripts and effect helpers use the existing `Fpc322Rng` stream, seeded explicitly by the caller. `FightTickReport` records character frame executions, successful box updates and source-located errors. A frame-script error prevents that fighter's box-application step for the tick.

## Current executable path

```text
host events
  -> InputSampler
  -> FixedStepRunner (28 ms)
  -> FightSimulation::tick(InputFrame)
  -> queued C01 command (explicit API, if supplied)
  -> common [Quadro] + character [Quadro]
  -> Pascal AplicarCaixas (hurtbox + ordered hitbox rows)
  -> live EffectArena::advance
  -> FightTickReport + read-only effect snapshots
```

A host can inspect `last_report()` and consume the effect snapshots later in its renderer. The simulation itself never draws.

## Deliberate boundaries

- `InputFrame` reaches the concrete fight tick unchanged. Direction-to-movement, physical key/controller mappings and the source-derived multi-button command recognizer are not invented by this slice.
- C01 special commands can be queued explicitly with `queue_chaves_special`; this is a simulation entry point, not a claim that keyboard/gamepad gestures are recognized.
- Common [Quadro] and the character-specific frame path now run before box selection. C01 special/victory/pose branches remain on the existing tested hand-port; other character sections use ROT within the mapped host boundary.
- `AplicarCaixas` uses the checked-in C01-C15 `CentroX`, `Corpo` and `Ataque` data. It preserves row order, player-two's 50-frame body offset, mirroring and retain-on-miss.
- The action-frame counter still uses the bounded runtime rule; action-end transitions, full L7 physics and exact global fight-loop parity are not claimed.
- The effect arena advances once after fighter frame work. Effect-vs-fighter/effect-vs-effect collision consequences remain pending source audit and integration.
- Runtime errors are recorded in the report instead of being replaced by silent fallback behavior.

## Tests

`tests/fight_simulation.rs` covers:

- the real 28 ms runner invoking the concrete fight implementation;
- common/character [Quadro] execution for both fighters and player-two's 50-frame hurtbox offset;
- left-facing hurtbox mirroring and source-table-driven box application;
- explicit C01 command entry followed by its existing frame executor;
- C01 frame 10 allocating a live type-7 effect before the effect phase;
- live effect movement/lifetime advancing without a renderer;
- normalized input arriving intact without invented command mapping;
- stopped-fight behavior retaining queued commands and freezing fighter frames.

These are headless integration contracts, not Pascal golden checkpoints or proof of full fight-loop parity.
