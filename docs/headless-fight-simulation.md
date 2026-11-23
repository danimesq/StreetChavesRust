# Headless FightSimulation

## Status

The Rust core now has its first concrete implementation of `SimulationTick`: `FightSimulation`. It composes existing source-backed components without creating a window, renderer, SDL dependency, browser API or WASM binding.

## Tick order

For each 28 ms call from `FixedStepRunner`:

1. Receive the immutable `InputFrame` and retain it in the tick report.
2. If the fight is not stopped, dispatch explicitly queued C01 special commands through the existing Common [Acao] + C01 [Acao] + Common [AcaoFim] path.
3. Run the existing C01 [Quadro] executor for C01 special, victory and pose actions.
4. Advance an eligible C01 action frame once, unless its frame script reset the frame.
5. Advance the existing live `EffectArena` exactly once and retain its ordered render snapshots.
6. Increment the fight tick and publish a `FightTickReport`.

The runtime uses the existing `Fpc322Rng` stream, seeded explicitly by the caller. No second effect collection or renderer-owned simulation state is introduced.

## Current executable path

```text
host events
  -> InputSampler
  -> FixedStepRunner (28 ms)
  -> FightSimulation::tick(InputFrame)
  -> queued C01 command (explicit API, if supplied)
  -> existing C01 [Acao] / [Quadro]
  -> live EffectArena::advance
  -> FightTickReport + read-only effect snapshots
```

A host can inspect `last_report()` and consume the effect snapshots later in its renderer. The simulation itself never draws.

## Deliberate boundaries

- `InputFrame` reaches the concrete fight tick unchanged. Direction-to-movement, physical key/controller mappings and the source-derived multi-button command recognizer are not invented by this slice.
- C01 special commands can be queued explicitly with `queue_chaves_special`; this is a simulation entry point, not a claim that keyboard/gamepad gestures are recognized.
- The current C01 [Quadro] executor is reused only for its implemented special, victory and pose paths. Other actions do not receive fabricated frame logic.
- Action-frame advancement is bounded by the state's existing `action_duration`. Generic Common [Quadro], action-end transitions, physics and the full fight-loop ordering are not implemented here.
- The effect arena advances once after fighter frame work. Effect-vs-fighter/effect-vs-effect collision consequences remain pending source audit and integration.
- Runtime errors are recorded in the report instead of being replaced by silent fallback behavior.

## Tests

`tests/fight_simulation.rs` covers:

- the real 28 ms runner invoking the concrete fight implementation;
- explicit C01 command entry followed by its existing frame executor;
- C01 frame 10 allocating a live type-7 effect before the effect phase;
- live effect movement/lifetime advancing without a renderer;
- normalized input arriving intact without invented command mapping;
- stopped-fight behavior retaining queued commands and freezing fighter frames.

These are headless integration contracts, not Pascal golden checkpoints or proof of full fight-loop parity.
