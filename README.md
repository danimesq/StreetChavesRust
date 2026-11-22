# StreetChavesRust

**Olha o churRust! Olha o churrust! Olha o churrus!** 🌶️

An independent Rust reimplementation project inspired by **Street Chaves — O Lutador da Vila**. The goal is to understand the game’s behavior, compare existing reconstructions, and build a clean Rust implementation with a testable game core and a separate SDL3 presentation layer.

> **Project status: research plus foundational Rust contracts.**
> The repository now contains a dependency-free Rust library for ordered combat timelines, sequential deterministic fixture RNG, and explicit hitbox/hurtbox update operations. It is not yet a playable game. The crate can be tested with `cargo test`; gameplay behavior and original-executable parity remain incomplete.

## Project goals

- Build an independent Rust implementation rather than mechanically translating either existing SDL codebase.
- Keep game simulation independent from SDL, rendering, audio, and operating-system services.
- Make game behavior observable and testable through deterministic ticks, tests, and replay fixtures.
- Preserve meaningful update order, collision boundaries, fighter state transitions, and effect-slot behavior.
- Separate verified original-game behavior from port-specific behavior, inference, and unresolved questions.
- Review source and asset provenance and license compatibility before reusing code or assets.

## Research references

The project studies three independent repositories. They are evidence sources, not a parent/child chain, and none is treated as automatically canonical.

| Project | Technology | Role here |
|---|---|---|
| [OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves) | Pascal / Lazarus / SDL2 | Reference for a working SDL-hosted reconstruction, fighter scripts, software rendering, and its migration history. |
| [open-street-chaves](https://github.com/danimesq/open-street-chaves) | C17 / SDL3 | Independent implementation and a useful reference for game-state logic, hitboxes, combat, effects, and testability. |
| [street-chaves-hack-sms](https://github.com/danimesq/street-chaves-hack-sms) | SMS ROM hack | Supporting reference for ROM investigation, graphics extraction, binary formats, and hardware constraints. It is **not** treated as an authoritative source for the PC game's mechanics. |

## Documentation

The research and design notes live in [the documentation directory](docs/README.md).

- [Comparative analysis](docs/comparative-analysis.md) — architecture, strengths, differences, and limitations of the SDL implementations.
- [Source and data inventory](docs/source-inventory.md) — pinned source revisions, code/data layout, entry points, and subsystem ownership.
- [Fighter data deep dive](docs/fighter-data-deep-dive.md) — INI/ROT semantics, generated C tables, command recognition, and character-specific behavior.
- [Fighter roster matrix](docs/fighter-roster-matrix.md) — original 15-character roster, Pascal C16 Chapolin extension, and script-section coverage.
- [Frame, AI and hitbox comparison](docs/frame-ai-hitbox-comparison.md) — all 15 Pascal frame/AI indexes, 379/379 exact hitbox geometry+metadata matches, and 305/305 hurtbox matches.
- [Ordered frame event ledger](docs/ordered-frame-event-ledger.md) — source-ordered frame, timer, effect, sound, RNG, and simulation-mutation statements for all 15 Pascal [Quadro] blocks.
- [Combat tick contracts](docs/combat-tick-contract.md) — Rust event ordering, deterministic RNG injection, hitbox/hurtbox records, and explicitly unresolved box-lifecycle semantics.
- [Original executable reverse engineering](docs/original-executable-reverse-engineering.md) — PE/resource/asset investigation and gameplay cross-reference plan.
- [REA tooling](tools/README.md) — reproducible preflight evidence collection and REA/Ghidra handoff.
- [Original executable reverse engineering](docs/original-executable-reverse-engineering.md) — preliminary PE inspection, DelphiX/DirectX clues, resource and asset inventory plan, gameplay cross-reference targets, and the later golden-fixture strategy.
- [Move matrix for the 15 fighters](docs/move-matrix-15-fighters.md) — move-by-move research; first detailed entry is Chaves, with evidence and original-executable validation gaps.
- [Chapolin bonus plan](docs/chapolin-bonus-2026.md) — preservation/recovery plan for Chapolin as a future optional bonus.
- [Data formats and resource pipelines](docs/data-formats.md) — fighter scripts, INI resources, generated C tables, and unresolved format semantics.
- [Provenance and licensing review](docs/provenance-and-licenses.md) — code licenses, asset origins, redistribution risks, and reuse decisions.
- [Fight-loop deep dive](docs/fight-loop-deep-dive.md) — source-level analysis of tick order, frame selection, hit resolution, effects, and physics.
- [Rust architecture proposal](docs/rust-architecture.md) — proposed boundaries between the simulation core and SDL3/platform services.
- [Evidence register and open questions](docs/evidence-and-open-questions.md) — evidence, confidence levels, provenance, unknowns, and validation priorities.
- [Project roadmap](docs/project-roadmap.md) — the full path from research through implementation, compatibility, packaging, and release.
- [Documentation and research roadmap](docs/documentation-roadmap.md) — the research documents and evidence needed to support each project phase.

Start with the [documentation index](docs/README.md) for the full research map.

## Engineering principles

1. **Evidence before assumptions.** Prefer original-executable observations and reproducible tests over guesses.
2. **Keep the sources distinct.** Record where Pascal and C implementations agree, differ, or leave behavior unknown.
3. **Simulation first.** The game tick should not depend on rendering side effects.
4. **Deterministic where practical.** Make important rules testable without launching the SDL frontend.
5. **Preserve ordering deliberately.** Update phases, collision checks, and fixed effect-slot iteration can affect gameplay.
6. **Do not fossilize bugs blindly.** A quirk in a reconstruction is not automatically original-game behavior.
7. **Keep provenance visible.** Do not copy code or assets before reviewing their origin and licensing.

## Current focus

The project has started its first implementation slice: a dependency-free Rust contract crate and contract-level tests. It is not yet a playable game. The current phase combines source research with foundational simulation contracts. The current research pass indexes all 15 Pascal frame scripts and AI blocks, compares all 379 hitbox geometry and metadata rows plus all 305 hurtboxes with the C tables, records exceptional move IDs, and publishes source-ordered frame-event ledgers. The first playable Rust baseline comes before the final original-executable compatibility stage:


- Compare the two SDL implementations at the source level.
- Map fighter frame selection and the combat update pipeline.
- Investigate effect types, collision paths, and timing-sensitive behavior.
- Perform preliminary static inspection of the original executable and identify code/resource targets.
- Identify discrepancies that need validation against the original executable.
- Convert confirmed behavior into Rust-facing specifications and deterministic test cases.

Unverified behavior stays documented as an open question rather than being filled in with invented mechanics. Static executable inspection is research groundwork; systematic original-executable pairing and golden differential fixtures remain part of the final compatibility stage after the first playable Rust baseline.

## Contributing and development

At this stage, the most useful contributions are reproducible observations, source references, test cases, and corrections to the research documents. Please distinguish direct source observations from interpretations and original-game claims.

Run the current contract suite with `cargo test`. There is not yet a playable runtime or SDL3 frontend, so no game launch command is available.

---

*An independent implementation, informed by comparison rather than blind translation.*
