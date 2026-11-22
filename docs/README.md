# StreetChavesRust documentation

This directory records the research and engineering decisions for an independent Rust reimplementation of **Street Chaves — O Lutador da Vila**.

## Roadmaps

- [Project roadmap](project-roadmap.md): the whole project, from research and architecture through implementation, validation, packaging, and release.
- [Documentation and research roadmap](documentation-roadmap.md): the research and specifications needed to support those project phases.

## Research documents

- [Comparative analysis of the SDL implementations](comparative-analysis.md): the Pascal/SDL2 and C17/SDL3 projects, their architectures, strengths, differences, and limitations.
- [Fight-loop deep dive](fight-loop-deep-dive.md): source-level comparison of the fight tick, fighter frame selection, hit resolution, effects, and physics.
- [Source and data inventory](source-inventory.md): pinned reference revisions, repository structure, runtime entry points, data roots, generated tables, and subsystem ownership.
- [Fighter data deep dive](fighter-data-deep-dive.md): INI/ROT semantics, the C specification-to-table pipeline, roster boundaries, and fighter-data differences.
- [Fighter roster matrix](fighter-roster-matrix.md): character IDs, Pascal/C coverage, optional sections, and the Chapolin extension.\n- [Frame, AI and hitbox comparison](frame-ai-hitbox-comparison.md): all 15 Pascal [Quadro]/[IA] blocks, C CPU comparison, 379 normalized hitbox rows, and exceptional move-ID findings.
- [Chapolin bonus plan](chapolin-bonus-2026.md): keep Chapolin recoverable as an optional 2026 bonus, separate from the base roster.
- [Data formats and resource pipelines](data-formats.md): INI/ROT data, fighter/stage/effect resources, generated C tables, asset loading and unresolved parser semantics.
- [Provenance and licensing review](provenance-and-licenses.md): declared project licenses, third-party material classes, asset rights risks, and current reuse decisions.
- [Rust architecture proposal](rust-architecture.md): a proposed separation of deterministic game logic from SDL3 presentation and platform services.
- [Evidence register and open questions](evidence-and-open-questions.md): facts, provenance, confidence levels, unresolved questions, and validation priorities.

## Source projects

| Project | Role in this research |
|---|---|
| [OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves) | Pascal/Lazarus reconstruction migrated to SDL2. Primary reference for a working SDL-hosted game, data-driven fighter scripts, software rendering, and migration strategy. |
| [open-street-chaves](https://github.com/danimesq/open-street-chaves) | Independent C17 + SDL3 reimplementation. Primary reference for reverse-engineered game rules, state machine, hitboxes, deterministic tests, and SDL3 runtime design. |
| [street-chaves-hack-sms](https://github.com/danimesq/street-chaves-hack-sms) | Supporting reference only. Useful for ROM tracing, graphics extraction, binary formats, and hardware constraints. It is not a reference implementation of the original PC game's mechanics. |

These repositories are independent projects, not a parent/child chain. Their claims must be compared rather than silently merged.

## Research rules

1. Prefer evidence from the original executable and reproducible observations over assumptions.
2. Distinguish original-game behavior from port-specific behavior.
3. Mark facts, inferences, and unknowns explicitly.
4. Do not copy source code or assets until provenance and license compatibility have been reviewed.
5. Keep the Rust core independent from SDL and the operating system.
6. Every behavior that can be tested should eventually have a deterministic test or replay fixture.
