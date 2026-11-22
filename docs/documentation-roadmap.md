# Documentation and research roadmap

## Purpose

This roadmap tracks the research and documentation needed to support the full [project roadmap](project-roadmap.md). It is not a substitute for the implementation plan: it defines what must be understood, documented, and validated before corresponding engineering decisions become commitments.

There are no calendar estimates. Each item advances according to evidence and explicit exit criteria.

## Status vocabulary

- **Planned**: identified, but not yet researched sufficiently.
- **In progress**: active source inspection or evidence collection.
- **Needs validation**: source evidence exists, but original-game behavior has not been confirmed.
- **Verified**: the stated acceptance criteria have been met and supporting evidence is linked.
- **Blocked**: progress depends on unavailable material, executable access, licensing clarification, or another explicit dependency.

## Current baseline

| Document | Role | Next improvement |
|---|---|---|
| [Comparative analysis](comparative-analysis.md) | Compares the independent Pascal/SDL2 and C17/SDL3 implementations | Expand subsystem and data-format comparisons |
| [Fight-loop deep dive](fight-loop-deep-dive.md) | Records source-level tick order, frame selection, combat, effects, and physics | Expand fighter-, move-, and effect-specific evidence |
| [Rust architecture proposal](rust-architecture.md) | Proposes simulation/platform boundaries and a candidate workspace | Revisit decisions as behavior and data formats become clearer |
| [Evidence register](evidence-and-open-questions.md) | Defines evidence labels and tracks facts and unknowns | Turn broad questions into traceable investigations and validation cases |
| [Source and data inventory](source-inventory.md) | Pins the three reference revisions and maps code/data ownership | Complete schema, parser and file-level asset inventory |
| [Data formats](data-formats.md) | Records observed INI/ROT, generated-table and resource pipelines | Test parser edge cases and finish format specifications |
| [Fighter data deep dive](fighter-data-deep-dive.md) | Compares Pascal INI/ROT execution with the C spec/generated-table model | Expand into per-character move and field matrices |
| [Fighter roster matrix](fighter-roster-matrix.md) | Records IDs, per-character file presence and optional script sections | Validate all move differences against original executable captures |
| [Frame, AI and hitbox comparison](frame-ai-hitbox-comparison.md) | Indexes all 15 Pascal frame/AI scripts and compares all 379 hitbox rectangles with C | Complete event timelines, resource mapping, hit metadata, and hurtboxes |
| [Provenance and licenses](provenance-and-licenses.md) | Records declared licenses and blocks unverified asset reuse | Complete per-file rights and dependency audit |

The baseline is useful, but it is not a complete specification. Agreement between two ports does not by itself prove original-game behavior.

## Research rules

For every meaningful claim:

1. Identify its origin: original executable, runtime observation, Pascal source, C source, SMS project, or inference.
2. Link the exact source file, symbol, data file, commit, binary hash, trace, or fixture where possible.
3. Apply the evidence labels in [the evidence register](evidence-and-open-questions.md).
4. Record disagreements instead of silently choosing a preferred port.
5. State what observation or test would resolve an uncertainty.
6. Update the evidence register when a finding changes a project-level decision.

Pin analyzed repository revisions. A moving `main` branch must not silently change the evidence base. Do not promote port-specific behavior into an original-game requirement without validation.

## Phase 0 — Foundations

**Status: In progress**

- [x] Documentation index.
- [x] Roles of the three independent reference projects.
- [x] Evidence vocabulary and initial open questions.
- [x] High-level SDL architecture comparison.
- [x] Fight-loop source-level analysis.
- [ ] Add a lightweight review checklist for new research documents.
- [x] Record exact reference-project commits used by each major investigation.

**Deliverable:** navigable documentation with consistent provenance and confidence language.

**Exit criteria:** a contributor can identify what is known, where it came from, and what remains unverified.

## Phase 1 — Source and data inventory

**Status: In progress**

Documents:
- `source-inventory.md`: relevant repositories, pinned revisions, directories, entry points, generated files, and subsystem ownership.
- `data-formats.md`: fighter data, scripts, C tables, resources, configuration, saves, and generated-data pipelines.
- `provenance-and-licenses.md`: source and asset origins, licenses, redistribution limits, and external dependencies.

Tasks:
- [x] Trace the documented paths by which fighter, stage, sound, sprite, and effect data enter each SDL implementation.
- [x] Identify checked-in, generated, extracted, and externally supplied data at repository level.
- [x] Keep SMS-specific findings in a clearly marked supporting section.
- [ ] Document parser behavior and format edge cases before designing Rust equivalents.
- [ ] Complete per-file asset provenance and third-party dependency license review.
- [ ] Confirm original executable/data archive identities and hashes used by each SDL reference.

**Current result:** repository structure, pinned revisions, major entry points, data roots, generated-table dimensions, and initial licensing boundaries are documented in the three Phase 1 documents. This is a partial inventory, not a completed format or rights audit.

**Exit criteria:** every relevant data source has a known origin, format, consumer, and reuse status. Unknown rights remain explicitly blocked.

## Phase 2 — Fighter, move, and animation specification

**Status: In progress**

Documents:
- `fighter-roster-matrix.md`: roster, IDs, names as represented in source data, availability, and optional script-section coverage.
- `fighter-data-deep-dive.md`: Pascal INI/ROT semantics compared with the C specification and generated tables.
- `fighter-data-schema.md`: common/per-fighter fields, defaults, ranges, and validation.
- `animation-and-frame-selection.md`: frame selection, animation clocks, transitions, frame-associated boxes, and side effects.
- `moves-and-command-recognition.md`: commands, button rules, priorities, buffering, and cancellation.
- `fighter-state-machine.md`: state inventory, transitions, timers, and exceptions.

Tasks:
- [x] Inventory all 16 Pascal INIs and all 16 character ROT section headers.
- [x] Establish the 15-character base roster and isolate Pascal C16 Chapolin as an extension.
- [x] Compare the Pascal data/script representation with the C spec-to-generated-table pipeline.
- [x] Inventory the five named special-move slots and input notation for all 15 fighters from both references.
- [x] Compare Pascal special-action duration assignments with the C move-setup duration table for all 15 fighters, recording exceptions and mismatches.
- [x] Complete the first cross-roster [Quadro] event index and compare all 379 Pascal/C hitbox rectangles by action/timer key and coordinates.
- [x] Compare all 15 character-specific Pascal [IA] blocks with the C CPU table; retain difficulty-scale and shared-AI caveats.
- [ ] Complete ordered per-character event timelines, resource-ID mapping, hit metadata, and hurtbox comparison.
- [ ] Compare every fighter's move behavior beyond the initial inventory; do not extrapolate from duration/name agreement.
- [x] Start the move-by-move matrix with Chaves, including Pascal command recognition, special-action behavior, hitbox rows, CPU rules, and validation backlog.
- [x] Complete the first comparative source pass for all 15 fighters; keep detailed event ordering and original-executable checks as separate remaining work.
- Trace input recognition through move selection, frame selection, hitbox activation, and reaction.
- Separate shared behavior from character-specific rules.
- Preserve source numeric IDs as evidence without assuming IDs match between implementations.

**Exit criteria:** each rule has a source reference and confidence label; a typed Rust schema can represent observed differences without hiding them in magic numbers.

## Phase 3 — Combat, effects, and physics

**Status: In progress**

Documents:
- `combat-and-damage.md`: hit qualification, guard, chip, meter, combo, knockback, hit-stop, and exceptions.
- `effects-and-projectiles.md`: effect inventory, lifecycle, spawn rules, slot ordering, collision, reflection, and snapshots.
- `physics-and-stage.md`: movement, gravity, push, bounds, camera, stage objects, and landing.
- `round-flow-and-timers.md`: round transitions, time-over, pause, win conditions, and game-clock behavior.

Tasks:
- Inventory effect types in both SDL projects; map equivalents only when evidence supports it.
- Investigate slot reuse, update order, current-tick drawing, and lifetime boundaries.
- Keep melee and effect collision as separate systems.
- Trace random-number calls and whether cosmetic behavior shares gameplay RNG state.
- Turn discrepancies into concrete test cases.

**Exit criteria:** combat and effects are described as ordered, independently testable systems; unresolved differences have explicit validation procedures.

## Phase 4 — Full game flow and platform-facing behavior

**Status: Planned**

Documents:
- `screen-flow.md`: boot, title/menu, selection, fight, results, endings, and other original screens.
- `input-and-controls.md`: keyboard, joystick/gamepad, logical actions, held/edge-triggered input, and remapping.
- `rendering-and-camera.md`: logical resolution, draw order, layers, scaling, animation presentation, and camera.
- `audio-and-music.md`: sound triggers, channels/voices, music transitions, timing, and dependencies.
- `save-config-and-options.md`: configuration, saves, unlocks, and persistent settings.

Tasks:
- Trace the original flow instead of treating either port's screen coverage as authoritative.
- Separate simulation events from visual/audio consequences.
- Document physical input behavior separately from logical game actions.
- Identify which behaviors can be tested headlessly and which need visual/audio comparison.

**Exit criteria:** the observable game flow is mapped and platform/presentation responsibilities are clearly separated from simulation.

## Phase 5 — Original-executable validation (final compatibility stage)

**Status: Deferred until the first playable Rust baseline exists**

Artifacts:
- `original-build-and-capture.md`: executable identity, hash, provenance, environment, and repeatable capture method.
- `runtime-observations/`: observations, traces, screenshots, and reproducibility metadata.
- `differential-test-matrix.md`: scenarios comparing original behavior, both SDL projects, and future Rust behavior.

Priority scenarios:
- Idle, movement, crouching, jumping, landing, and boundaries.
- Representative normal/special attacks and character-specific actions.
- Hit, block, chip damage, knockdown, throws, hit-stop, and round transitions.
- Projectile spawn, clash, reflection, fighter collision, and expiration.
- Pause/focus behavior, timer boundaries, and RNG sequences.

This phase begins after the first playable Rust baseline. It is not a gate for reaching the first playable build. Each observation should record executable hash, environment, initial conditions, input sequence, tick/frame reference, and expected result. Screenshots alone cannot prove hidden logical state.

**Exit criteria:** high-impact mechanics have reproducible original-game observations or remain explicitly unverified; Rust divergences are recorded as compatibility decisions rather than silently patched.

## Phase 6 — Asset and resource pipeline

**Status: Planned**

Documents:
- `asset-catalog.md`: resource inventory, dimensions, formats, hashes, references, and provenance.
- `asset-pipeline.md`: extraction/inspection tools, conversion steps, naming, validation, and reproducibility.
- `asset-licensing.md`: redistribution policy and separation of original assets from source code.

Tasks:
- Identify minimum external data needed for a playable build.
- Document sprite-sheet slicing and animation-frame mapping.
- Identify redistributable fonts/audio/tools and their licenses.
- Make inspection tools emit hashes and provenance metadata.
- Keep the SMS pipeline distinct from PC-game resource assumptions.

**Exit criteria:** assets used by the project have traceable origins, reproducible processing steps, and a clear redistribution policy.

## Phase 7 — Implementation-facing specifications

**Status: Planned**

Documents:
- `core-specification.md`: tick contract, state ownership, update phases, and deterministic guarantees.
- `replay-format.md`: versioned input events, RNG initialization, checkpoints, and compatibility policy.
- `test-fixtures.md`: fixture format, expected state snapshots, and golden-image separation.
- `error-and-logging-conventions.md`: diagnostics, trace levels, and failure reporting.

Tasks:
- Turn verified behavior into testable contracts.
- Link each implementation requirement to evidence and a validation case.
- Record deliberate deviations and compatibility trade-offs.
- Keep architecture docs synchronized with actual workspace decisions.

**Exit criteria:** implementation tasks can be written without relying on undocumented assumptions.

## Document review checklist

Before marking a research document complete:

- [ ] Scope and non-goals are explicit.
- [ ] Source repositories and commits are identified.
- [ ] Claims are labeled as evidence, inference, unknown, or port-specific.
- [ ] Important claims link to exact files/symbols or reproducible observations.
- [ ] Disagreements between projects are visible.
- [ ] Open questions include a proposed way to resolve them.
- [ ] Rust implications are separated from observations about the existing projects.
- [ ] Licensing/provenance concerns are recorded where relevant.
- [ ] The documentation index and evidence register are updated if needed.

## Relationship to the project roadmap

The [project roadmap](project-roadmap.md) governs the full engineering effort. This document supports it by tracking the research and specifications needed to make each implementation phase evidence-led. If implementation uncovers contradictory evidence, update the research documents and revise the project plan rather than preserving an outdated assumption.
