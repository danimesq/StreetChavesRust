# StreetChavesRust documentation

This directory records the research and engineering decisions for an independent Rust reimplementation of **Street Chaves — O Lutador da Vila**.

## Roadmaps

- [Project roadmap](project-roadmap.md): the whole project, from research and architecture through implementation, validation, packaging, and release.
- [Documentation and research roadmap](documentation-roadmap.md): the research and specifications needed to support those project phases.

## Research documents

- [Original executable reverse engineering](original-executable-reverse-engineering.md): preliminary PE/toolchain findings and the investigation plan for embedded resources, external assets, gameplay cross-references, and final compatibility fixtures.
- [Delphi source reconstruction and reproducible binary](delphi-source-and-reproducible-binary.md): two separate long-term research tracks, from source-level recovery to a strict byte-identical rebuild criterion.

- [Comparative analysis of the SDL implementations](comparative-analysis.md): the Pascal/SDL2 and C17/SDL3 projects, their architectures, strengths, differences, and limitations.
- [Fight-loop deep dive](fight-loop-deep-dive.md): source-level comparison of the fight tick, fighter frame selection, hit resolution, effects, and physics.
- [Source and data inventory](source-inventory.md): pinned reference revisions, repository structure, runtime entry points, data roots, generated tables, and subsystem ownership.
- [Fighter data deep dive](fighter-data-deep-dive.md): INI/ROT semantics, the C specification-to-table pipeline, roster boundaries, and fighter-data differences.
- [Fighter roster matrix](fighter-roster-matrix.md): character IDs, Pascal/C coverage, optional sections, and the Chapolin extension.
- [Frame, AI and hitbox comparison](frame-ai-hitbox-comparison.md): all 15 Pascal [Quadro]/[IA] blocks, C CPU comparison, exact 379/379 hitbox geometry+metadata and 305/305 hurtbox matches, and exceptional move-ID findings.
- [Ordered frame event ledger](ordered-frame-event-ledger.md): source-ordered frame/timer, effect, sound, RNG, and simulation-mutation statements from all 15 Pascal [Quadro] blocks.
- [Combat tick contracts](combat-tick-contract.md): Rust contracts for ordered events, normative FPC 3.2.2 MTwist RNG, collision geometry, and box records.
- [Box lifecycle deep dive](box-lifecycle-deep-dive.md): exact C selector and Pascal AplicarCaixas behavior, including retain-on-miss, inactive rows, ordered strength variants, and a port divergence.
- [RNG compatibility](rng-compatibility.md): normative FPC 3.2.2 MTwist implementation, one-word LongInt and two-word Int64 behavior; experimental Delphi LCG candidate remains separate.
- [Pascal reference trace harness](pascal-reference-trace-harness.md): deterministic --gravar regression, schema-v2 tick/script/control/RNG JSONL, repeatability evidence, and Rust reconciliation CLI.
- [Pascal RNG inventory](pascal-rng-inventory.md): 255 unique .rot RNG sites, 465 composed occurrences, and 71 .pas/.inc references.
- [ROT normalizer](rot-normalizer.md): parser for all 15 fighter scripts, nested guards/loops and symbolic RNG sites.
- [C RNG call-site inventory](rng-callsite-inventory.md): comparative C shared RNG call sites and fight-tick phase order.
- [Original executable reverse engineering](original-executable-reverse-engineering.md): PE/toolchain, resources, assets, gameplay cross-references, and final compatibility-fixture plan.
- [Chapolin bonus plan](chapolin-bonus-2026.md): keep Chapolin recoverable as an optional 2026 bonus, separate from the base roster.
- [Data formats and resource pipelines](data-formats.md): INI/ROT data, fighter/stage/effect resources, generated C tables, asset loading and unresolved parser semantics.
- [Provenance and licensing review](provenance-and-licenses.md): declared project licenses, third-party material classes, asset rights risks, and current reuse decisions.
- [Rust architecture proposal](rust-architecture.md): a proposed separation of deterministic game logic from SDL3 presentation and platform services.
- [Evidence register and open questions](evidence-and-open-questions.md): facts, provenance, confidence levels, unresolved questions, and validation priorities.

## Tooling

- [Reverse-engineering tools](../tools/README.md): REA preflight collector, evidence bundle format, and the handoff workflow into REA/Ghidra.

## Source projects

| Project | Role in this research |
|---|---|
| [OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves) | Pascal/Lazarus reconstruction migrated to SDL2. **Normative project source** for fighter scripts, interpreter behavior, update order and FPC 3.2.2 RNG semantics. |
| [open-street-chaves](https://github.com/danimesq/open-street-chaves) | Independent C17 + SDL3 reimplementation. Comparative source for rules, hitboxes and testability; differences are annotations and do not override Pascal. |
| [street-chaves-hack-sms](https://github.com/danimesq/street-chaves-hack-sms) | Supporting reference only. Useful for ROM tracing, graphics extraction, binary formats, and hardware constraints. It is not a reference implementation of the original PC game's mechanics. |

These repositories are independent projects, not a parent/child chain. For the Rust reconstruction, Pascal is the normative source by explicit project policy. C is compared and disagreements are documented for later original-executable reverse engineering.

## Research rules

1. Follow the Pascal source/runtime first; keep C as a comparative implementation and record divergences for original-executable verification.
2. Prefer evidence from the original executable and reproducible observations over assumptions.
3. Distinguish original-game behavior from port-specific behavior.
4. Mark facts, inferences, and unknowns explicitly.
5. Do not copy source code or assets until provenance and license compatibility have been reviewed.
6. Keep the Rust core independent from SDL and the operating system.
7. Every behavior that can be tested should eventually have a deterministic test or replay fixture.
