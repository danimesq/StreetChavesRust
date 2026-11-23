# 🌈 ROT interpreter and live action-execution contract

## 🌈 Status and scope

The Rust crate contains a source-preserving lexer/parser, typed local execution, and a host-aware execution path for live fight state. M1 parses every section in `Comum.rot` and `C01.rot` through `C15.rot`. The scalar executor handles local assignments, blocks, conditionals, loops and `case`; the host executor additionally binds fighter fields, game globals, RNG and engine helpers through an explicit adapter.

The first end-to-end character path is C09 (Paty) command `cmEsp3Fraco` / `cmEsp3Forte`. `FightSimulation` executes the pinned scripts in source order: common `[Acao]`, C09 `[Acao]`, then common `[AcaoFim]`. The invocation uses the live `CombatState`, maps `P` and `O` to the actual fighters, consumes the fight's shared `Fpc322Rng`, and routes `ExisteEfeito`, `CriaEfeito` and `Empurra` to their existing simulation components.

**This is a first bounded character integration, not yet a roster-wide runtime.** Only the explicitly mapped fighter fields, globals, helpers and C09 command paths are executable. Unsupported field/helper access returns a source-located error instead of being fabricated or ignored.

The normative source is [OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves), pinned at commit `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`. The checked-in files under `research/pascal-rot/` are the reproducible input set.

## 🌈 Rust API

- `RotDocument::from_str(source, text)` extracts INI-style sections in source order while preserving original line locations.
- `RotDocument::section(name)` finds a section case-insensitively.
- `RotDocument::parse_optional(name)` returns `None` when a section is absent. It never invents a script for a missing section.
- `RotSection::parse()` returns a `RotProgram` or a `RotError` carrying source name, line, column and message.
- `RotProgram` contains local declarations and a source-ordered statement tree. `SourceSpan` records line/column ranges and byte offsets.
- `RotExecutionFrame::new(program, jog, codigo)` creates fresh locals and initializes `Jog`, byte-narrowed `Codigo` and `Desloc` (0 for player 1, 50 for player 2).
- `RotExecutionFrame::set_symbol(name, value)` supplies host-owned constants or readable global symbols. Local names take precedence.
- `RotExecutionFrame::set_local(name, value)` applies the declared Pascal assignment conversion.
- `RotExecutionFrame::evaluate(expression)` evaluates one supported expression.
- `RotExecutionFrame::execute(program)` runs the local subset with a default one-million-step budget. `execute_with_step_limit` accepts an explicit host budget.
- `RotExecutionFrame::execute_with_host(program, host)` runs the same AST through `RotExecutionHost`, which supplies live fighter/global reads and writes, helper calls and the shared RNG connection.

## 🌈 Parsed syntax

The lexer handles identifiers, decimal and hexadecimal integers, decimal/exponent real literals, quoted strings with doubled-quote escaping, ROT operators and punctuation, and `//`, `{...}` and `(*...*)` comments.

The parser handles:

- `var` declarations for Byte, ShortInt, SmallInt, Word, Integer/LongInt, Int64, Double and Boolean. Unknown type names are retained in the AST and rejected when a frame needs to instantiate them.
- assignment and procedure/function calls;
- `begin/end`, `if/then/else`, `for/to/downto/do`, `while/do` and `repeat/until`;
- `case/of/else`, set literals, ranges and `in`;
- Pascal expression precedence for comparisons, arithmetic, integer division/modulo, shifts, bitwise operators and unary operators.

Whole-line semicolon comments are removed as the Pascal loader does, while their newline is preserved so later source locations remain stable.

## 🌈 M2 value and execution semantics

The value model distinguishes integer values with a separate Int64-width flag, real values, Booleans, strings and evaluated integer sets. Integer assignment narrows/wraps to Byte, ShortInt, SmallInt, Word or Integer. Int64 remains wide. Integer-to-Double and real-to-Double assignment are accepted; real-to-integer and Boolean/numeric conversions are rejected.

Boolean `and/or` short-circuit. Integer `and/or/xor` are bitwise operations and evaluate both operands. Comparisons reject mixing strings, Booleans and numbers. Set membership supports scalar values and inclusive integer ranges.

The pure functions `Round`, `Trunc`, `Abs`, `Cos`, `Sin` and `ShortInt` are implemented. The pinned FPC 3.2.2 profile uses ties-to-even rounding for `Round`; the evaluator implements that profile.

Statement execution preserves source order. `for` bounds are evaluated once, lower bound first. `while` checks before its body; `repeat/until` checks after its body. `case` evaluates its selector once, scans arms in source order and enters the first matching arm. Each arm's labels are evaluated in order before its match decision.

The step budget is a host safety guard to stop non-terminating scripts. It is not claimed to be a Pascal language rule.

## 🌈 Deliberate runtime boundary

The following limitations remain:

- Only explicitly mapped `P.*` / `O.*` fields are exposed by the fight host. Unknown fields fail with a source-located error.
- Only the globals used by the first action path and currently mapped constants are bound. Other globals fail explicitly.
- `Random` currently supports the LongInt call path used by C09's action script. Int64 RNG calls are not yet exposed by the fight host.
- `ExisteEfeito`, `CriaEfeito` and `Empurra` are connected to the live effect arena and combat state. `AcaoJogador` and `AplicarCaixas` are not yet connected.
- C09 command codes 14, 15, 20 and 21 are routed through ROT. Other characters and C09's remaining command paths retain their existing bounded ports/unhandled reporting.
- Nested character action calls do not yet recursively invoke the complete Pascal `AcaoJogador` lifecycle.

The host adapter uses the authoritative `CombatState`, effect arena and same `Fpc322Rng` owned by `FightSimulation`; it does not create a parallel script-only fighter state or random generator.

Rust currently represents real values as `f64`, while the Pascal interpreter uses `Extended` internally. High-precision differential validation remains open. No floating-point equivalence is claimed.

## 🌈 Validation

The test suite parses every section in all 16 pinned ROT files. It also covers literal parsing, source spans/errors, typed local defaults and assignment conversions, arithmetic/operators, sets, Boolean short-circuiting, pure functions, local statement execution, loop/control-flow order, unsupported host-bound operations and the step limit.

Fight integration tests execute C09's real `[Acao]` section through the normal command dispatcher and assert live action state, charge, sound, five effect allocations, ten shared RNG draws and the common prelude/epilogue behavior.

Run the checks from the repository root:

```sh
cargo fmt --all -- --check
cargo test --all-targets
```
