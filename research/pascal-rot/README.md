# 🌈 Pinned Pascal .rot inputs

These files are verbatim research inputs from [OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves), pinned at commit `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`.

- `Comum.rot` is the shared script executed before each fighter's corresponding section.
- `C01.rot` through `C15.rot` are the 15-fighter roster inputs.
- `C16.rot` is intentionally excluded because this reconstruction's active roster is 15 fighters.
- Source file blob SHAs are recorded in `source-manifest.json`.
- Upstream project license: GPL-3.0. These copies are research inputs; preserve their provenance and license when redistributing.

## Normative-source rule

The Pascal interpreter and scripts define the primary semantics for Rust normalization and implementation. The C17/SDL3 port is a comparative implementation. When the ports disagree, record the C difference and keep the Pascal rule as the Rust default until the original Chaves.exe is reverse-engineered and provides contrary evidence.

The normalizer does not execute game state. It emits symbolic structure, source line references, branch guards, loop regions, and static RNG call sites. Runtime tick numbers and dynamic RNG indices are assigned only by a concrete execution/replay harness.
