# 🌘 Rust Gaming Eclipse: StreetChavesRust integration plan

**Status:** Architecture proposal, 2026-11-24.  
**Scope:** Define how StreetChavesRust will consume RustMultiConsoleFoundation, evaluate Bevy/Rapier without losing the deterministic game core, and later serve as an integration reference for ImpossibleEngine.  
**Implementation status:** This document does not add Bevy, Rapier, or Foundation dependencies and does not claim that a playable runtime exists.

## 🌘 1. StreetChavesRust remains an independent game

StreetChavesRust owns the Street Chaves domain:

- Fighter, move, stage, effect, and game-flow rules.
- Source-derived data and provenance.
- The fixed logical simulation tick and deterministic replay.
- Collision and hit/hurt-box semantics required for compatibility.
- Game-specific asset meaning and audio events.

The existing architecture remains the starting point: a headless Rust game core with renderer-neutral output and replaceable host/presentation adapters. The current repository has foundational Rust contracts and research, but no playable runtime or SDL3/Canvas frontend yet.

StreetChavesRust must not become a dependency of RustMultiConsoleFoundation or ImpossibleEngine. It is a real consumer, a migration/reference project, and a demanding integration fixture. Its implementation may inform reusable abstractions, but game-specific rules must not leak into the Foundation or compiler.

## 🌘 2. Target dependency direction

The intended dependency direction is:

```text
StreetChavesRust game domain
  simulation · combat · data · replay · semantic events
                    |
                    v
RustMultiConsoleFoundation public contracts
  time · input · display · audio · storage · assets · diagnostics
                    |
                    v
selected platform adapter / target profile
  SDL · browser host · Orbital · console SDK · custom pipeline
```

The game declares capabilities it needs. The Foundation resolves the supported implementation for the selected target. StreetChavesRust must not import SDL, Orbital, browser bindings, or console SDK types from its game-domain crates.

The Foundation currently contains the `foundation-packaging` workspace crate, not a published general-purpose runtime/API crate. Therefore this project must not add a guessed `rust_multi_console_foundation` dependency yet. The consumer integration begins only after the Foundation publishes or tags a documented API crate with a compiling example and target-specific feature contract.

## 🌘 3. Bevy is an optional host/runtime integration, not the Foundation

Bevy and the Foundation solve different problems:

- **Bevy** is a game engine: ECS, application scheduling, asset handling, rendering, windowing/input integrations, and related engine facilities.
- **RustMultiConsoleFoundation** is intended to provide stable cross-target platform contracts, capability reporting, adapters, asset adaptation, build tooling, and packaging.
- **Rapier 2D** is a physics engine. `bevy_rapier2d` connects Rapier to Bevy through Bevy plugins and systems. It is not a replacement for SDL windowing, input, audio, or presentation.

There are two valid integration modes, but a build must not have two owners for the same host responsibility:

### A. Foundation-hosted mode

The Foundation selects and owns the target adapter and host loop. StreetChavesRust supplies simulation updates and renderer-neutral presentation data. This is the desired low-friction route for desktop, browser, and supported console targets.

### B. Bevy-hosted mode

A Bevy `App` owns its application schedule and the Bevy-managed window/input/rendering path. A future Foundation-Bevy bridge may expose compatible Foundation capabilities as Bevy resources/plugins, while avoiding a second window, event loop, renderer, or input owner.

The Bevy bridge is optional and target-limited. It must not become a prerequisite of the game core or of the Foundation's non-Bevy adapters.

## 🌘 4. Current Bevy/Rapier research snapshot

As of 2026-11-24:

- The latest stable Bevy release found is **0.20.1** (2026-10-28).
- The current `bevy_rapier2d` release found is **0.37.0**, which depends on Bevy `^0.20`. Select matching release lines; do not combine an arbitrary Bevy and Rapier version.
- Bevy's published platform overview lists Windows, macOS, Linux, Web, iOS, and Android. It does not establish PSP or classic-console support.
- Bevy's own 0.20.1 crate documentation warns that it is still early-stage and that breaking API changes are released about every three months. Its MSRV tracks the latest stable Rust rather than StreetChavesRust's current `rust-version = "1.70"`.
- Bevy 0.20 highlights Solari and DLSS work. That does not make advanced GPU rendering a baseline requirement for a 2D retro game or imply those features exist on every target.

References:

- https://github.com/bevyengine/bevy/releases
- https://bevy.org/news/bevy-0-20/
- https://bevy.org/
- https://docs.rs/crate/bevy/0.20.1
- https://docs.rs/crate/bevy_rapier2d/0.37.0
- https://www.rapier.rs/docs/user_guides/bevy_plugin/getting_started_bevy/

Because Bevy's MSRV and API cadence are materially different from the current StreetChavesRust baseline, Bevy must remain opt-in until a separate compatibility/build experiment justifies changing the project's Rust version policy.

## 🌘 5. Preserve the deterministic simulation before experimenting

The game core remains authoritative. Bevy systems must not silently redefine its semantics.

Preserve, in particular:

- The source-derived 28 ms logical tick and documented update ordering.
- The explicit RNG state, algorithm, draw width, and draw order.
- Pascal/FPC 3.2.2 as the normative project source for the reconstruction; C remains comparative evidence.
- The strict AABB and source-specific hitbox/hurtbox lifecycle contracts.
- Immutable renderer-neutral snapshots or ordered draw commands.
- Headless tests and deterministic replay independent of the host loop.

The initial Bevy integration should call the existing core at explicit fixed-tick boundaries and consume its presentation snapshot. It should not recreate the combat simulation as a second, independently authoritative ECS implementation.

## 🌘 6. Rapier evaluation gates

Rapier is a candidate for general-purpose rigid-body, contact, and collision workloads. It must not be adopted merely because the project is written in Rust.

StreetChaves has compatibility-sensitive rules: hand-authored boxes, strict boundary behavior, specific update ordering, stale-box retention, and source-specific hit/hurt-box selection. A general-purpose physics solver may express different contact, integration, and floating-point semantics.

Therefore:

1. Keep current explicit AABB and game-specific collision contracts as the compatibility baseline.
2. Prototype Rapier in an isolated experiment or optional adapter, not as a replacement in the authoritative core.
3. Define a bounded workload where a physics engine offers measurable value (for example, non-authoritative stage objects or optional effects).
4. Compare fixed-seed traces, contact outcomes, boundary cases, update ordering, and target builds against the baseline.
5. Promote Rapier only if the behavior difference is understood, covered by tests, and accepted as either compatible or an explicitly non-compatible enhancement.

SDL and Rapier are not competitors: SDL is a platform/media library; Rapier supplies physics simulation. Bevy can host Rapier, while SDL or another Foundation adapter can host a game that uses standalone Rapier.

## 🌘 7. Relationship with ImpossibleEngine

ImpossibleEngine should eventually be able to open, model, and build a StreetChaves-derived project, but it must not depend on the StreetChavesRust crate or copy its game-specific implementation into the compiler.

The intended distinction is:

- **StreetChavesRust** is the independent reference implementation and source of game-specific behavior.
- **ImpossibleEngine** owns project import, event-sheet semantics, typed IR, compilation/lowering, and authoring workflows.
- **RustMultiConsoleFoundation** owns the reusable platform/runtime contracts and target adapters.

A future Street Chaves import should preserve source provenance and distinguish data that can be represented in Event Sheets from behavior that still requires a native/custom runtime module. Importing a project must not be described as a complete or lossless conversion until every required semantic has an explicit mapping and regression fixture.

## 🌘 8. Integration sequence and acceptance criteria

1. **Keep the core stable:** continue the current evidence-led, headless game-core work. Do not block combat correctness on Bevy or Foundation work.
2. **Foundation API first:** wait for a real, versioned runtime contract and a minimal reference consumer in RustMultiConsoleFoundation.
3. **Minimal Foundation consumer:** add one StreetChaves host adapter for a single validated desktop target; map normalized input, time, display submission, diagnostics, and audio only as their APIs exist.
4. **Bevy spike:** separately test Bevy 0.20.x with the matching Rapier plugin. Measure build/MSRV impact, fixed-tick integration, renderer/input ownership, and browser behavior.
5. **Choose per target:** Foundation-native mode may serve targets where its adapters are stronger; Bevy-hosted mode may serve targets where Bevy is a good fit. Do not promise a Bevy route for PSP or other unvalidated consoles.
6. **ImpossibleEngine fixture:** use the game as an end-to-end migration/reference project only after the compiler can represent its behavior or explicitly call a supported native module.

Acceptance requires a compiling minimal consumer, explicit capability reporting, tests for deterministic simulation independence, reproducible target/build instructions, and no duplicate host ownership.

## 🌘 9. Decisions still open

- The exact public Foundation runtime crate name and semver policy.
- Whether a Foundation-Bevy bridge belongs in the Foundation repository or a separate integration crate.
- Which StreetChaves subsystems, if any, benefit from ECS representation instead of the current domain model.
- Whether standalone Rapier has a justified bounded use outside authoritative combat.
- Which first Foundation target should replace the planned direct SDL3 host path.
- How ImpossibleEngine imports projects that combine Event Sheets with native Rust modules.

These questions are intentionally not resolved by guessing APIs or creating placeholder dependencies.
