# Rust architecture proposal

## Goals

- Reimplement the original PC game independently in Rust.
- Preserve observable behavior where evidence supports it.
- Keep simulation deterministic and testable without SDL or a window.
- Use SDL3 for platform services without allowing SDL types to leak into the game domain.
- Make unknown behavior explicit instead of inventing mechanics.
- Keep original assets separate from source code and document their provenance.

## Proposed workspace

```text
street-chaves-rust/
├── Cargo.toml
├── README.md
├── docs/
│   ├── README.md
│   ├── comparative-analysis.md
│   ├── rust-architecture.md
│   └── evidence-and-open-questions.md
├── crates/
│   ├── chaves-core/       # world, tick, game state, orchestration
│   ├── chaves-fighter/    # player state, moves, hit/hurt boxes
│   ├── chaves-physics/    # movement, gravity, push, collision
│   ├── chaves-data/       # validated fighter/stage/config data
│   ├── chaves-replay/     # input recording, deterministic playback
│   ├── chaves-render/     # renderer-neutral draw commands
│   ├── chaves-audio/      # renderer-neutral audio events
│   ├── chaves-input/      # logical action mapping
│   └── chaves-sdl3/       # SDL3 window, input, renderer, audio adapters
├── tests/
│   ├── fixtures/
│   ├── deterministic/
│   └── golden/
└── tools/
    ├── asset-inspector/
    └── replay-inspector/
```

This is a proposal, not a claim that every crate should exist from day one. Start with the smallest compiling workspace and split crates when boundaries become useful.

## Dependency direction

```text
                 +-------------------+
                 |    chaves-sdl3    |
                 | window/input/audio|
                 +---------+---------+
                           |
                           v
+-------------------+   +-------------------+
|  chaves-replay    |-->|    chaves-core    |
| record/play input |   | deterministic sim|
+-------------------+   +---------+---------+
                                  |
                    +-------------+-------------+
                    |             |             |
                    v             v             v
             chaves-fighter chaves-physics chaves-data

chaves-core emits renderer-neutral draw data and audio events.
chaves-render consumes draw data; chaves-audio consumes audio events.
SDL3 adapters implement the platform-facing sides.
```

The exact dependency graph can evolve, but the core must not import SDL3, OS window handles, device identifiers, or wall-clock APIs.

## Simulation clock

The original-game research identifies a default interval of 28 ms. The simulation should advance in discrete logical ticks. The SDL loop may run at any presentation rate, but it must not change the meaning of a game tick.

A scheduler can accumulate elapsed wall-clock duration and execute zero or more fixed updates. Rendering may interpolate visual positions later, but interpolation must never feed back into gameplay state.

The displayed game timer is a separate counter: the recovered specification says one displayed second advances every 25 logical ticks. Keep this as an explicit game rule, not a conversion from wall-clock seconds.

Avoid floating-point nondeterminism where practical. Before selecting fixed-point representations, measure the original's numeric behavior and decide whether exact compatibility or stable cross-platform replay is the priority for each field.

## Deterministic random numbers

Do not call a global random API from gameplay code. Introduce an explicit RNG state owned by the simulation and pass it through systems that need randomness.

A replay must record:
- replay format version;
- initial RNG seed/state;
- initial game configuration;
- logical input actions per tick;
- optional checkpoints and state hashes.

The same version, initial state, and input sequence should produce the same logical state sequence.

The original executable's Delphi RNG behavior is documented in the SDL3 reverse-engineering spec. If exact compatibility is required, implement and test that algorithm as a named compatibility RNG rather than silently using Rust's default RNG.

## Domain model

Prefer explicit types over raw integers:

- `PlayerId`, `FighterId`, `StageId`;
- `GameScreen` and `FighterState` enums;
- `LogicalInput` / `PlayerActions`;
- `HitBox`, `HurtBox`, and `RectI`;
- `MoveId`, `EffectId`, and `AnimationFrameId`;
- `GameTick` and `FrameTimer`.

Keep original numeric IDs in data when they are part of compatibility evidence, but wrap them in validated newtypes.

A fighter update should have a documented order of operations. The recovered fight-tick ordering is important: input, AI, stage objects, drawing/frame selection side effects, hit detection, hit-stop, animation clocks, and physics are not interchangeable. Model the order explicitly and add tests for it.

## Data-driven content

Start with typed Rust structs and a versioned data format. Validate:
- required fighter and stage identifiers;
- frame references and animation ranges;
- hit/hurt box dimensions;
- move command syntax;
- effect references;
- numeric bounds and timer values.

Do not reproduce the Pascal `.rot` interpreter or the C project's generated-table format without a concrete need. The target is to preserve behavior, not the source representation.

Generated data must have a reproducible generator and a documented source. Never hand-edit generated tables without recording the change in the generator or source data.

## Rendering boundary

The simulation should emit renderer-neutral commands or a snapshot of renderable state. The renderer owns:
- SDL textures and surfaces;
- scaling and letterboxing;
- texture upload and lifetime;
- font rasterization;
- debug overlays;
- screenshot capture.

Keep a 400×300 logical coordinate system as the initial compatibility target. Do not assume the original's 3× software buffer is a gameplay rule; it is a presentation choice in the Pascal implementation.

For early compatibility work, a software renderer can be useful because it is easy to inspect and hash. A GPU-backed renderer can be added later without changing the simulation API.

## Audio boundary

The core emits semantic audio events such as a move sound, hit sound, menu sound, or music transition. The SDL adapter resolves those events to audio assets and playback.

The core must not open audio devices, mix samples, or depend on SDL audio callback timing.

## Input boundary

SDL keyboard, controller, and joystick events are translated into logical actions before they reach the simulation. Keep physical scancodes and device IDs in the adapter layer.

The logical input representation should preserve edge-triggered presses and held states separately. The original game uses both held directions/buttons and press counters for command recognition, so a single boolean per button may not be sufficient.

## Testing strategy

1. **Unit tests** for geometry, timers, command parsing, and individual transitions.
2. **Tick tests** for exact update ordering and state evolution.
3. **Replay tests** with fixed seeds and scripted inputs.
4. **State snapshots** at known ticks, with stable serialization and hashes.
5. **Golden rendering tests** separate from logical-state tests.
6. **Differential tests** against traces or captures from the original and the independent SDL projects.

When a golden image changes, require an explicit update and a short explanation of why it is expected.

## SDL3 dependency

Use the Rust SDL3 bindings behind `chaves-sdl3`. The current crate documentation describes both high-level `sdl3` bindings and low-level `sdl3-sys`; the high-level crate is the natural first choice unless a required SDL3 API is missing.

The SDL3 Rust bindings expose build options for linking an installed SDL3 or building it from source. Decide packaging and CI policy separately for Linux, Windows, macOS, and browser targets. Do not make the game core's build depend on the presence of SDL3.

## First implementation milestones

- [ ] Workspace skeleton and documented dependency rules.
- [ ] Typed logical coordinates, tick type, player ID, and input actions.
- [ ] Deterministic RNG abstraction with a compatibility test.
- [ ] Headless simulation runner and replay format.
- [ ] Player state representation based on verified fields.
- [ ] A minimal, evidence-backed transition set.
- [ ] Renderer-neutral snapshot and SDL3 window.
- [ ] Input adapter for keyboard and gamepad.
- [ ] First differential fixture against an original-game trace.

No gameplay behavior should be added merely to make the first demo look complete. Unknowns belong in the evidence register.
