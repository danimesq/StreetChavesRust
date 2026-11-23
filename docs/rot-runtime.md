# 🌈 ROT parser and local execution contract

## 🌈 Status and scope

The Rust crate contains a source-preserving lexer/parser for the pinned Pascal ROT dialect and an **M2 scalar/local execution subset**. M1 parses every section in `Comum.rot` and `C01.rot` through `C15.rot`. M2 evaluates typed expressions and executes local assignments, blocks, conditionals, loops and `case` statements.

**This is not yet a character action-script runtime.** The executor deliberately stops before fighter/global state, RNG and game-engine helpers. No real character `[Acao]` script is currently executed end to end.

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
- `RotExecutionFrame::execute(program)` runs the supported local subset with a default one-million-step budget. `execute_with_step_limit` accepts an explicit host budget.

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

The following are not yet connected:

- `P.*` / `O.*` fighter-field reads or writes;
- game-global reads or writes;
- `Random`, including LongInt/Int64 width and the shared nested RNG cursor;
- engine helpers `ExisteEfeito`, `CriaEfeito`, `Empurra`, `AcaoJogador` and `AplicarCaixas`;
- the complete lifecycle `Comum [Acao]` → character `[Acao]` → `Comum [AcaoFim]` inside the interpreter.

These operations return source-located errors rather than fabricated zeros or silent no-ops. The host adapter must use the authoritative `CombatState`, effects and shared RNG, not a parallel script-only state.

Rust currently represents real values as `f64`, while the Pascal interpreter uses `Extended` internally. High-precision differential validation remains open. No floating-point equivalence is claimed.

## 🌈 Validation

The test suite parses every section in all 16 pinned ROT files. It also covers literal parsing, source spans/errors, typed local defaults and assignment conversions, arithmetic/operators, sets, Boolean short-circuiting, pure functions, local statement execution, loop/control-flow order, unsupported host-bound operations and the step limit.

Run the checks from the repository root:

```sh
cargo fmt --all -- --check
cargo test --all-targets
```
