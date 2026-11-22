# 🌈 Pascal .rot normalizer

## 🌈 Purpose

`tools/rot-normalize.py` parses the actual scripting dialect implemented by the pinned `source/game/ChavesRoteiro.pas`. It is a static normalizer, not a second game interpreter and not a dynamic execution engine.

The Rust reconstruction follows Pascal as its normative source. The C17/SDL3 port is a comparison source only; divergences are logged and do not silently override Pascal behavior. Disputed behavior still gets a separate original-executable reverse-engineering check.

## 🌈 Pinned inputs

The source inputs are the verbatim `Comum.rot` and `C01.rot`..`C15.rot` files in `research/pascal-rot/`, pinned by `source-manifest.json` to OpenStreetChaves commit `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`. The normalizer verifies Git blob SHA-1s before parsing.

## 🌈 Supported grammar

The parser covers the constructs implemented by `ChavesRoteiro.pas`:

- declarations `var Nome: Tipo;`
- assignments and procedure/function calls
- `if / then / else`, `begin / end`
- `for / to / downto / do`
- `while / do`, `repeat / until`
- `case / of / else`
- expression precedence, unary operators, comparisons, sets, ranges and function calls
- comments, quoted strings, hexadecimal integers and numeric literals

The parser preserves nested structure and source line/column spans. It does not evaluate expressions or simulate fighter state.

## 🌈 Symbolic timeline output

For each fighter it combines the common section before the fighter-specific section, matching the Pascal orchestration order:

- `[Acao]`: action-entry conditions and state mutations
- `[Quadro]`: frame/tick guards, frame changes, effects, sounds, and state mutations
- `[IA]`: AI branch conditions and decisions
- `[Acerto]`: hit-response logic when present

Each extracted event retains its branch guards, enclosing loop regions, source location, symbolic action predicates, and symbolic `P.QuadroAcao` predicates.

A `Random(...)` call is represented as a static RNG site. The output deliberately leaves runtime tick and dynamic draw index unset. Those values cannot be known without an execution with concrete fighter state and branch outcomes.

Boolean `and/or` follows the Pascal interpreter rule: if the left operand is Boolean it can short-circuit; if it is integer bitwise logic, both operands are evaluated. The normalizer records that distinction rather than assuming every textual `and/or` short-circuits.

Function arguments with more than one argument preserve an explicit argument-evaluation profile. The Pascal interpreter determines `OrdemArgsInversa` with a runtime compiler probe, so the default is `unknown`. Do not set a direction until the relevant interpreter build has been measured.

## 🌈 Pascal RNG inventory

The optional `--pascal-source-root` scan finds `Randomize`, `RandSeed`, `Random`, `Sortear` and `Sortear64` in Pascal `.pas` and `.inc` files, excluding comments and string literals. It records path, line, column, call kind and source excerpt.

The scan is static. It does not assign execution order across camera, fighter, stage, effects, HUD or round-state systems. The call-site inventory must be combined with the fight-loop phase graph and conditional guards before it becomes a dynamic RNG trace.

An important existing mechanism is already present in `ChavesRoteiro.pas`: `ModoDeSorteio(msGravar/msRepetir)` can record and replay requests, checking both range and 32/64-bit mode. That should be reused as a source for deterministic reference traces, rather than inventing a parallel Pascal recorder.

## 🌈 Run locally

From the repository root, with the pinned OpenStreetChaves checkout available:

```sh
python3 -m unittest discover -s tools -p 'test_*.py' -v

python3 tools/rot-normalize.py \
  --input-dir research/pascal-rot \
  --output-dir build/rot-normalized \
  --pascal-source-root /path/to/OpenStreetChaves/source \
  --argument-order unknown
```

Generated files:

- `normalized-timelines.json`: full ASTs and symbolic events for all 15 fighters
- `timeline-summary.md`: roster/section/event counts and interpretation notes
- `pascal-rng-call-sites.json`: Pascal RNG call-site inventory
- `source-manifest.json`: source hashes for the generated bundle

## 🌈 CI

The `Pascal .rot normalization` workflow runs grammar unit tests, checks the pinned Pascal source, normalizes the full 15-fighter roster, verifies the output boundaries and uploads the bundle as the `pascal-rot-symbolic-timelines` artifact.

## 🌈 Deliberate non-goals

- No concrete tick execution or guessed fighter starting state.
- No flattening of nested guards, loops, repeat-until conditions or case dispatch.
- No fabricated global RNG draw indices.
- No assumption that source-order AST order alone determines runtime FPC argument evaluation order.
- No C-port rule promotion when it conflicts with Pascal.
- No original-executable compatibility claim before the later REA/Ghidra stage.
