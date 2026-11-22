# Project roadmap

## Vision

Build an independent Rust implementation of **Street Chaves — O Lutador da Vila**, with a deterministic, testable game simulation and an SDL3-based presentation/runtime layer.

This roadmap describes the whole project: research, architecture, implementation, validation, packaging, and release. It is not a claim that any listed feature is already implemented. The repository is currently in the research and architecture stage.

There are no calendar estimates. Each phase advances when its exit criteria are met. Delivery is deliberately split into two stages: (1) build a complete, independently playable Rust baseline from the best documented evidence available; (2) after that baseline exists, run the final compatibility stage by pairing it with the original executable. Original-executable parity is not a prerequisite for the first playable milestone. The original game executable and its data are treated as evidence and compatibility references, not as permission to redistribute copyrighted material.

## Guiding constraints

- Independent implementation: do not mechanically translate either SDL project.
- Evidence-led behavior: distinguish original-game observations from port-specific behavior and inference.
- Normative-source precedence: follow OpenStreetChaves Pascal/FPC 3.2.2 for Rust behavior; use C17/SDL3 for comparison only. Record disagreements and leave them open for original-executable verification.
- Simulation/platform separation: the game core must not depend on SDL, window handles, device APIs, or wall-clock time.
- Deterministic tests: identical initial state, RNG state, and logical inputs should reproduce the same simulation results.
- Compatibility before embellishment: preserve observed behavior first; improvements and optional enhancements must be explicit.
- Asset provenance: no code or asset reuse before source, license, and redistribution review.
- Incremental delivery: every milestone should leave the repository buildable and documented once implementation begins.

## Roadmap at a glance

| Phase | Focus | Status |
|---|---|---|
| 0 | Research baseline and project definition | In progress |
| 1 | Original-game evidence and source/data inventory | In progress / needs validation (Pascal .rot pins and symbolic normalizer added) |
| 2 | Technical architecture and workspace foundation | In progress (minimal Cargo crate created) |
| 3 | Core simulation primitives and deterministic harness | In progress (FPC 3.2.2 RNG, timeline contracts and static .rot normalization; no playable simulation yet) |
| 4 | Fighter data, animation, and state machine | Planned |
| 5 | Combat, effects, and physics | Planned |
| 6 | SDL3 runtime, input, rendering, and audio adapters | Planned |
| 7 | Complete arcade flow and content integration | Planned |
| 8 | Final original-executable compatibility, regression, and platform hardening | Deferred until first playable baseline |
| 9 | Packaging, documentation, and first release | Planned |

## Phase 0 — Research baseline and project definition

**Status: In progress**

Goals:
- Compare the independent Pascal/SDL2 and C17/SDL3 projects.
- Use the SMS hack as a supporting reference for reverse-engineering and asset-pipeline techniques only.
- Record architecture, fight-loop order, frame selection, collision, effects, and physics.
- Maintain an evidence register and explicit open questions.
- Define the Rust project's independent goals and non-goals.

Deliverables:
- Comparative analysis.
- Fight-loop deep dive.
- Rust architecture proposal.
- Evidence register.
- Project and documentation roadmaps.

Exit criteria:
- Contributors can distinguish source-observed facts, original-game evidence, port-specific behavior, inference, and unknowns.
- Major architectural decisions have written rationale.
- No implementation requirement is based solely on an unverified assumption.

## Phase 1 — Original-game evidence and source/data inventory

**Status: In progress / needs validation**

Goals:
- Pin the exact commits of reference projects used for each investigation.
- Follow Pascal/FPC 3.2.2 as the normative Rust reconstruction source; keep C-port divergences separately documented for original-executable verification.
- Inventory and hash the supplied original executable as a preliminary static-research artifact; deeper runtime pairing and differential validation remain reserved for the final compatibility stage and do not block the first playable Rust baseline.
- Inventory fighter, move, stage, effect, sprite, sound, configuration, and save data.
- Determine which values are static, decompiled, runtime-observed, cross-project, inferred, or unknown.
- Audit source and asset licenses and redistribution constraints.

Deliverables:
- Source inventory and pinned revision manifest.
- Verbatim `Comum.rot` and `C01.rot`..`C15.rot` inputs plus a SHA-pinned manifest.
- Static symbolic timelines for all 15 fighters and a Pascal RNG source inventory.
- Data-format and resource catalog.
- Preliminary original executable identity and PE inspection report; repeatable runtime capture procedure for the later compatibility stage.
- Provenance/license notes.
- Initial differential-test matrix.

Exit criteria:
- Every planned input dataset has a known origin and format.
- The reference executable and capture environment are reproducible, or the limitation is explicitly documented.
- Unclear asset rights are treated as a blocker for redistribution, not guessed away.

## Phase 2 — Technical architecture and workspace foundation

**Status: In progress**

Initial work: a dependency-free Cargo library now exists with typed fighter/action/tick identifiers and source-preserving combat data structures. The full workspace layout and SDL3 application crate remain future work.

Goals:
- Convert the architecture proposal into a minimal Rust workspace.
- Establish dependency direction and crate boundaries only where they provide real value.
- Define error handling, logging, configuration, feature flags, and test conventions.
- Select and pin SDL3 Rust bindings for the platform adapter, without making the core depend on SDL.

Candidate initial structure:
- `chaves-core`: game orchestration and logical world state.
- `chaves-data`: validated content definitions and loaders.
- `chaves-replay`: deterministic input recording and playback.
- `chaves-sdl3`: window, device input, renderer, and audio integration.

Fighter, physics, rendering, and audio crates can be split out when their interfaces stabilize. Do not create a large crate forest before the code needs it.

Exit criteria:
- A minimal workspace builds and runs automated tests.
- Dependency rules prevent SDL types from entering core APIs.
- CI can compile and test the headless core without a display or audio device.

## Phase 3 — Core simulation primitives and deterministic harness

**Status: In progress**

Initial contracts cover ordered per-tick events, FPC 3.2.2 MTwist width-sensitive RNG costs, explicit Pascal box lifecycle operations, strength-conditioned row ordering, and strict AABB overlap. The static normalizer covers all 15 fighter scripts. The Pascal schema-v2 exporter compiles and has two byte-identical 3,000-step captures with matching `logica.txt` checkpoints. The Rust CLI reconciles all 8,841 wrapper draws against the symbolic index: 8,415 unique .rot matches, 0 ambiguous, 0 unmatched, and 426 non-.rot source contexts (422 direct wrapper calls and 4 helper calls). Rust simulation/replay integration, complete branch-predicate semantics, global RNG coverage, and original-executable parity remain open.

Goals:
- Implement logical coordinates, ticks, player identifiers, input actions, timers, geometry, and explicit game state types.
- Implement a simulation-owned RNG abstraction; add a compatibility algorithm only when evidence requires it.
- Define stable state snapshots and a versioned replay format.
- Build a headless runner that advances fixed logical ticks from scripted inputs.
- Add structured logs/traces for state transitions and debugging.

Exit criteria:
- Unit tests cover primitive types and boundary behavior.
- A replay can be recorded and played back deterministically.
- State hashes or snapshots can identify the first divergent tick.
- Headless tests do not initialize SDL.

## Phase 4 — Fighter data, animation, and state machine

**Status: Planned**

Goals:
- Define validated fighter and move data schemas from evidence, not from one port's file format.
- Map the complete roster and character identifiers.
- Implement input command recognition, move selection, animation clocks, frame selection, and state transitions.
- Represent frame-associated hitboxes, hurtboxes, motion, sound, and effect triggers as simulation data.
- Keep character-specific exceptions explicit and traceable.

Exit criteria:
- All documented fighter data passes schema validation.
- Representative and character-specific transitions have deterministic tests.
- Frame selection produces simulation-side state changes; rendering only consumes snapshots.
- Unknown or conflicting fighter behavior remains marked as unresolved.

## Phase 5 — Combat, effects, and physics

**Status: Planned**

Goals:
- Implement AABB collision with verified edge semantics.
- Implement attack qualification, guarding, damage, chip, meter, combo, knockback, hit-stop, and reaction behavior.
- Implement effects/projectiles as a separate collision and lifecycle system from melee.
- Preserve stable effect-slot ordering where evidence shows it affects behavior.
- Implement movement, gravity, push, bounds, landing, camera interaction, and stage objects.
- Separate simulation events from render and audio consequences.

Exit criteria:
- Combat and physics rules have unit and tick-level tests.
- Effect lifecycle, slot reuse, collision, and current-tick rendering behavior are covered.
- Differential fixtures exist for representative attacks, blocks, hits, and effects.
- Port-specific quirks are either intentionally rejected or documented as compatibility decisions.

## Phase 6 — SDL3 runtime, input, rendering, and audio adapters

**Status: Planned**

Goals:
- Create the SDL3 application shell and fixed-tick scheduler.
- Map keyboard and controller events into logical actions.
- Implement renderer-neutral draw snapshots/commands and an SDL3 renderer.
- Implement audio event consumption and asset playback outside the core.
- Add scaling, letterboxing, fullscreen, focus handling, and debug overlays.
- Keep presentation frame rate independent from simulation tick rate.

Exit criteria:
- The game can open a window and render a diagnostic scene.
- Input adapters produce logical actions without leaking SDL types into the core.
- Renderer and audio adapters can be replaced or disabled independently.
- Rendering does not mutate simulation state.

## Phase 7 — Complete arcade flow and content integration

**Status: Planned**

Goals:
- Implement title/menu, character selection, fight setup, rounds, results, endings, and other verified screens.
- Integrate fighter, stage, effect, sprite, sound, music, font, and UI resources.
- Implement configuration, controls, and save/unlock behavior where supported by evidence.
- Ensure all required content has provenance and a documented loading path.
- Reach a complete playable loop before adding optional enhancements.

Exit criteria:
- A player can progress through the complete intended arcade flow.
- Required content loads with validation and useful diagnostics.
- Missing or malformed data fails clearly rather than crashing unpredictably.
- The game is playable with keyboard and at least one supported controller path, if controller support is validated for the chosen SDL target.

## Phase 8 — Final original-executable compatibility, regression, and platform hardening

**Status: Deferred until the first playable Rust baseline exists**

The preliminary static-inspection plan and its investigation sequence are tracked in [original executable reverse engineering](original-executable-reverse-engineering.md). Static PE/resource inspection can proceed during research, but pairing the original executable with Rust for behavioral compatibility remains deferred.

Goals:
- Begin systematic differential testing against the original executable only after the first playable Rust baseline exists.
- Run deterministic replay and logical-state regression suites.
- Maintain golden rendering tests separately from gameplay tests.
- Test timing, RNG, focus loss, pause, resize, audio-device loss, and save/config edge cases.
- Validate packaging and runtime dependencies on selected desktop platforms.
- Profile performance and memory before optimizing.

Exit criteria:
- Critical gameplay scenarios have reproducible expected results.
- Regressions identify the first divergent tick or rendering stage.
- Supported platform claims are backed by actual build/run testing.
- Known incompatibilities and limitations are published.

## Phase 9 — Packaging, documentation, and first release

**Status: Planned**

Goals:
- Provide reproducible developer builds and user-facing packages.
- Document build prerequisites, asset/data requirements, controls, configuration, troubleshooting, and known limitations.
- Add versioning, changelog, release notes, and issue templates.
- Publish only assets and binaries that can legally be redistributed.
- Define a support policy and a process for compatibility regressions.

Exit criteria:
- A clean environment can follow the documented build or installation path.
- Release artifacts are traceable to source commits and build instructions.
- Licensing and third-party notices are included.
- The README accurately describes tested platforms and the actual feature set.

## Cross-cutting quality gates

These apply throughout the roadmap:

- **Correctness:** no gameplay rule is accepted without evidence or an explicitly labeled design decision.
- **Determinism:** core behavior is testable without wall-clock scheduling or SDL.
- **Observability:** logs and replay diagnostics make failures debuggable.
- **Traceability:** generated data, source revisions, and asset origins are recorded.
- **Maintainability:** avoid premature abstraction and unnecessary crate splitting.
- **Accessibility/usability:** controls, scaling, audio settings, and input remapping are considered before release.
- **Scope control:** optional features must not delay a verified, complete core gameplay loop without an explicit decision.

## What is deliberately not promised yet

- A playable build or a release date; the first playable milestone and the later original-executable parity milestone are separate deliverables.
- Exact compatibility for mechanics that have not been validated against the original executable.
- Redistribution of original game assets or executable data.
- Browser/mobile support before desktop architecture and packaging are proven.
- Reproducing every bug or implementation quirk from either reference project.

## Relationship to the documentation roadmap

The [documentation roadmap](documentation-roadmap.md) tracks the research documents and evidence needed to support these phases. The two roadmaps should evolve together: implementation milestones must not outrun the evidence and design decisions they depend on.
