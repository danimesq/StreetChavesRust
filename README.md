# StreetChavesRust

**Olha o churRust! Olha o churrust! Olha o churrus!** 🌶️

An independent Rust reimplementation project inspired by **Street Chaves — O Lutador da Vila**. The goal is to understand the game’s behavior, compare existing reconstructions, and build a clean Rust implementation with a testable game core and a separate SDL3 presentation layer.

> **Project status: first runnable native SDL3 loop through RustMultiConsoleFoundation, source-derived simulation slices, and deterministic Pascal evidence.**
> The repository contains Rust contracts for ordered combat timelines, Pascal/FPC 3.2.2 MTwist RNG compatibility, strict AABB geometry, and source-specific hitbox/hurtbox lifecycle behavior. The static .rot normalizer covers all 15 fighters: 5,091 symbolic events, 1,749 guard groups, 255 unique RNG sites (465 fighter-timeline occurrences), plus 71 Pascal .pas/.inc RNG references. Separately, an experimental Pascal schema-v2 runtime exporter now captures concrete ticks, nested script invocations, actual branch/loop outcomes and SortearValor draws. Two fixed-seed 3,000-step runs compiled with FPC 3.2.2 and produced byte-identical traces and logica.txt checkpoints. The Rust joiner reconciled 8,841 draws: 8,415 unique .rot matches, 0 ambiguous, 0 unmatched, and 426 non-.rot contexts (422 direct wrapper calls, 4 Pascal helpers). The Pascal exporter remains on a research branch; Rust simulation/replay and original-executable parity remain incomplete. The game is not yet playable.

## Project goals

- Build an independent Rust implementation rather than mechanically translating either existing SDL codebase.
- Keep game simulation independent from SDL, rendering, audio, and operating-system services.
- Make game behavior observable and testable through deterministic ticks, tests, and replay fixtures.
- Preserve meaningful update order, collision boundaries, fighter state transitions, and effect-slot behavior.
- Separate verified original-game behavior from port-specific behavior, inference, and unresolved questions.
- Review source and asset provenance and license compatibility before reusing code or assets.

## Research references

The project studies three independent repositories, but they do not have equal authority for this reconstruction. **OpenStreetChaves (Pascal/Lazarus) is the normative project source** for Rust behavior, including its FPC 3.2.2 interpreter/runtime semantics. The C17/SDL3 project is a comparative implementation: disagreements are documented and reserved for original-executable verification, but C behavior does not silently override Pascal.

| Project | Technology | Role here |
|---|---|---|
| [OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves) | Pascal / Lazarus / SDL2 | **Normative project source** for fighter scripts, interpreter semantics, game-state order, and FPC 3.2.2 RNG behavior. |
| [open-street-chaves](https://github.com/danimesq/open-street-chaves) | C17 / SDL3 | Comparative implementation for cross-checking state logic, hitboxes, combat and effects. Its differences remain annotations, not normative overrides. |
| [street-chaves-hack-sms](https://github.com/danimesq/street-chaves-hack-sms) | SMS ROM hack | Supporting reference for ROM investigation, graphics extraction, binary formats, and hardware constraints. It is **not** treated as an authoritative source for the PC game's mechanics. |

## Documentation

The research and design notes live in [the documentation directory](docs/README.md).

- [Comparative analysis](docs/comparative-analysis.md) — architecture, strengths, differences, and limitations of the SDL implementations.
- [Source and data inventory](docs/source-inventory.md) — pinned source revisions, code/data layout, entry points, and subsystem ownership.
- [Fighter data deep dive](docs/fighter-data-deep-dive.md) — INI/ROT semantics, generated C tables, command recognition, and character-specific behavior.
- [Fighter roster matrix](docs/fighter-roster-matrix.md) — original 15-character roster, Pascal C16 Chapolin extension, and script-section coverage.
- [Frame, AI and hitbox comparison](docs/frame-ai-hitbox-comparison.md) — all 15 Pascal frame/AI indexes, 379/379 exact hitbox geometry+metadata matches, and 305/305 hurtbox matches.
- [Ordered frame event ledger](docs/ordered-frame-event-ledger.md) — source-ordered frame, timer, effect, sound, RNG, and simulation-mutation statements for all 15 Pascal [Quadro] blocks.
- [Combat tick contracts](docs/combat-tick-contract.md) — Rust event ordering, Delphi 5 RNG, hitbox/hurtbox records, and strict collision geometry.
- [Box lifecycle deep dive](docs/box-lifecycle-deep-dive.md) — C selector and Pascal AplicarCaixas update order, stale-box retention, strength variants, and the fighter 11 discrepancy.
- [Pascal RNG compatibility](docs/rng-compatibility.md) — FPC 3.2.2 MTwist semantics, LongInt/Int64 raw-word costs, deterministic seed and the optional Delphi candidate.
- [Pascal reference trace harness](docs/pascal-reference-trace-harness.md) — schema-v2 tick/script/control/RNG JSONL capture, deterministic replay evidence, and Rust-to-symbolic reconciliation.
- [Pascal RNG inventory](docs/pascal-rng-inventory.md) — 255 unique `.rot` sites, 465 composed occurrences and 71 Pascal source references.
- [ROT normalizer](docs/rot-normalizer.md) — parser, source pins, symbolic timeline output and CI artifact.
- [C RNG call-site comparison](docs/rng-callsite-inventory.md) — shared C RNG call sites, tick-phase order and variable draw-count traps.
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

The project has started its first implementation slice: a dependency-free Rust contract crate with ordered tick events, Delphi 5 RNG compatibility for an injected seed, strict AABB geometry, source-specific C/Pascal box-lifecycle adapters, and explicit attack deactivation that preserves stored metadata. The Rust format check and 16 contract tests pass in GitHub Actions. It is not yet a playable game. The current phase combines source research with foundational simulation contracts. The current research pass indexes all 15 Pascal frame scripts and AI blocks, compares all 379 hitbox geometry and metadata rows plus all 305 hurtboxes with the C tables, records exceptional move IDs, and publishes source-ordered frame-event ledgers. The first playable Rust baseline comes before the final original-executable compatibility stage:


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


## Native host integration

The opt-in native-foundation feature consumes the shared SDL3 host from RustMultiConsoleFoundation. It centralizes the native window, keyboard events, textures, WAV decoding and PCM mixer while StreetChavesRust retains its two-player 28 ms simulation clock and Pascal sound-bank indexing. See docs/foundation-native-host-pr37.md. Sprite-sheet source rectangles and pivots must be registered from audited local metadata; the runtime does not guess that P.Quadro is an atlas cell.


## Native executable

A first runnable native loop is available through the optional Foundation host:

    cargo run --features native-foundation --bin street-chaves-native

Run with local OpenStreetChaves assets for original stage/fighter sheets/audio:

    cargo run --features native-foundation --bin street-chaves-native -- --game-root "/path/to/OpenStreetChaves" --stage 1 --p1 1 --p2 4

Without a game folder it opens a procedural training stage with animated vector fighters. See [the native runnable loop guide](docs/native-playable-loop-pr38.md). Original game BMP/WAV assets are not redistributed.
