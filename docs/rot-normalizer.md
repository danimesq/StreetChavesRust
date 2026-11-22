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

Each extracted event retains its branch guards, enclosing loop regions, source location, symbolic action predicates, and symbolic `P.QuadroAcao` predicates. A second output groups the events into symbolic action/timer guard lanes, so a reader can navigate events gated by `P.Acao = acEspecial2` and `P.QuadroAcao = 10` without evaluating those conditions.

A `Random(...)` call is represented as a static RNG site. Each event records the Pascal integer width and expected raw MT word cost: LongInt consumes one FPC 3.2.2 MT word; Int64 consumes two and reduces the assembled 63-bit value modulo the range. The output deliberately leaves runtime tick and dynamic raw-word index unset. Those values cannot be known without an execution with concrete fighter state and branch outcomes.

Boolean `and/or` follows the Pascal interpreter rule: if the left operand is Boolean it can short-circuit; if it is integer bitwise logic, both operands are evaluated. The normalizer records the actual RHS execution condition. `case` arms retain ordered reachability guards: an arm is reached only if no earlier arm matched, and `else` requires that none of the listed arms matched.

Function arguments with more than one argument preserve an explicit argument-evaluation profile. The Pascal interpreter determines `OrdemArgsInversa` with a runtime compiler probe. The CI now installs the pinned FPC 3.2.2 compiler, builds `tools/fpc-argument-order-probe.pas` (the same probe as the interpreter initialization), and passes the measured direction to the normalizer. On the current x86_64 Linux runner it reported `right-to-left` with evaluation trace `5431`, so the last argument is evaluated first. This is a measured target/compiler profile, not a universal guarantee for every FPC target; rerun the probe for another build environment.

## 🌈 Pascal RNG inventory

The optional `--pascal-source-root` scan finds `Randomize`, `RandSeed`, `Random`, `Sortear` and `Sortear64` in Pascal `.pas` and `.inc` files, excluding comments and string literals. It records path, line, column, call kind and source excerpt.

The scan is static. It does not assign execution order across camera, fighter, stage, effects, HUD or round-state systems. The unique `.rot` RNG inventory is kept separately from the `.pas/.inc` call-site inventory, and shared `Comum.rot` sites are deduplicated even though they occur in each fighter's composed timeline.

An important existing mechanism is already present in `ChavesRoteiro.pas`: `ModoDeSorteio(msGravar/msRepetir)` can record and replay requests, checking both range and 32/64-bit mode. The experimental [upstream branch](https://github.com/danimesq/OpenStreetChaves/tree/research/sortearvalor-jsonl-v1) adds schema-v2 JSONL for ticks, nested script executions, control decisions/loop values, and RNG requests. It compiled under FPC 3.2.2 and produced two byte-identical fixed-seed 3,000-step captures, including matching `logica.txt` checkpoints. The MT-word index is local to `SortearValor`; direct `System.Random` calls elsewhere remain outside it. The branch remains unmerged and the normalizer's source pin unchanged. See [Pascal reference trace harness](pascal-reference-trace-harness.md) for the artifact and validation evidence.

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
- `symbolic-action-tick-index.json`: compact groups keyed by symbolic `P.Acao` and `P.QuadroAcao` guard contexts, with full guards/loop/source order retained
- `timeline-summary.md`: roster/section/event counts and interpretation notes
- `pascal-rng-call-sites.json`: Pascal `.pas/.inc` RNG call-site inventory
- `rot-rng-call-sites.json`: unique `.rot` RNG sites, including width and raw MT word cost
- `source-manifest.json`: source hashes for the generated bundle
- `argument-order.txt`: compiler-order result and evaluation trace from the FPC probe (included in the CI artifact)

## 🌈 CI

The `Pascal .rot normalization` workflow installs and verifies FPC 3.2.2, runs the compiler argument-order probe, tests the grammar, checks the pinned Pascal source, normalizes the full 15-fighter roster with the measured profile, verifies the output boundaries and uploads the bundle as the `pascal-rot-symbolic-timelines` artifact. The completed full run #42 (9 parser tests passed) installed and verified FPC 3.2.2, measured right-to-left argument evaluation with trace `5431`, then produced 5,091 symbolic events grouped into 1,749 symbolic action/timer lanes, 255 unique `.rot` RNG sites represented 465 times across the composed fighter timelines, and 71 RNG-related `.pas/.inc` references. The 255 unique sites split into 160 LongInt (one raw MT word) and 95 Int64 (two raw MT words); composed occurrences split into 244 and 221 respectively.

## 🌈 Deliberate non-goals

- No concrete tick execution or guessed fighter starting state. The action/timer index is a symbolic guard index, not a dynamic tick log.
- No flattening of nested guards, loops, repeat-until conditions or case dispatch.
- No fabricated global RNG draw indices.
- No assumption that source-order AST order alone determines runtime FPC argument evaluation order.
- No C-port rule promotion when it conflicts with Pascal.
- No original-executable compatibility claim before the later REA/Ghidra stage.

 
## 🌈 Concrete Pascal trace integration status

The schema-v2 Pascal capture is now compiled, repeatable and represented by a
real tick-316 fixture. Rust's `src/trace_join.rs` loads the concrete JSONL and
the static symbolic action/tick index, then joins by fighter/character,
script file and section, source line, RNG width, and literal range where
available. The first captured `Comum.rot [IA]` draw uniquely matches
`Common.IA.L293.C4`; tests also preserve an intentionally ambiguous
same-line candidate and reject mapping Pascal-helper draws to `.rot` sites.

The end-to-end pipeline now reconciles all 8,841 calls from each deterministic
capture against the regenerated 15-fighter index. The observed result is
8,415 unique .rot source matches, zero ambiguous, zero unmatched, and 426
non-.rot contexts (422 direct wrapper calls plus 4 Pascal-helper calls).
The report also preserves the matched site's symbolic action/tick guards,
source location, RNG width/range and the IDs of control observations from
the same script invocation.

This proves the draw-to-static-site join for this 3,000-step Pascal scenario.
It does not prove that each runtime branch observation is the evaluation of
one particular symbolic boolean subexpression. Runtime source columns are
still absent, and direct System.Random calls outside SortearValor plus
full game-state snapshots remain outside schema v2. Static normalization
remains a source model; the runtime JSONL is the execution evidence.
