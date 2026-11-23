# 🌈 ROT parser contract

## 🌈 Status and scope

The Rust crate contains a source-preserving lexer/parser for the pinned Pascal ROT dialect and an initial **M2 expression-evaluation core**. M1 parses every pinned section. The evaluator supports typed call-local storage, scalar values, operators, sets and the pure math/conversion functions. It does not execute statement trees, bind fighter/global state, invoke engine helpers or consume random numbers.

The normative input is the Pascal reconstruction in [OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves), pinned at commit \`e1b025c35ecbc2b14d8635564aafd23c2107d3d1\`. The checked-in files under \`research/pascal-rot/\` remain the reproducible input set.

## 🌈 Rust API

- \`RotDocument::from_str(source, text)\` extracts INI-style sections while preserving their source order and original line locations.
- \`RotDocument::section(name)\` finds a section case-insensitively.
- \`RotDocument::parse_optional(name)\` returns \`None\` when an optional section does not exist. It never synthesizes an empty script for a missing section.
- \`RotSection::parse()\` returns a \`RotProgram\` or a \`RotError\` carrying the source name, line, column, and message.
- \`RotProgram\` contains local declarations and a source-ordered statement tree. \`SourceSpan\` records start/end line and column plus byte offsets within the extracted section text.

The AST retains typed literal values, declared Pascal types, P/O fighter-field references, function calls, unary/binary expressions, sets/ranges, assignments, blocks, conditionals, loops, and ordered case arms.

## 🌈 Supported syntax

The lexer handles identifiers, decimal and hexadecimal integers, decimal/exponent real literals, quoted strings with doubled-quote escaping, the ROT operators and punctuation, and the source comment forms \`//\`, \`{...}\`, and \`(*...*)\`.

The parser handles:

- \`var\` declarations for Byte, ShortInt, SmallInt, Word, Integer/LongInt, Int64, Double, Boolean, and explicitly retained unknown type names;
- assignment and procedure/function calls;
- \`begin/end\`, \`if/then/else\`, \`for/to/downto/do\`, \`while/do\`, and \`repeat/until\`;
- \`case/of/else\`, set literals, ranges, and \`in\`;
- Pascal expression precedence for comparisons, arithmetic, integer division/modulo, shifts, bitwise operators, and unary operators.

Whole-line semicolon comments are removed as the Pascal loader does, while preserving the corresponding newline so later source locations remain stable.

## 🌈 Deliberate execution boundary

Parsing does not establish runtime compatibility. The following remain separate evaluator/integration work:

- Pascal integer widths, overflow and assignment conversions;
- Boolean short-circuit versus integer bitwise evaluation;
- the measured FPC 3.2.2 right-to-left function-argument evaluation profile;
- local-frame initialization and nested \`AcaoJogador\` calls;
- the shared RNG cursor and LongInt/Int64 \`Random\` behavior;
- typed P/O/global state access and engine helpers such as \`CriaEfeito\`, \`ExisteEfeito\`, \`Empurra\`, and \`AplicarCaixas\`;
- the action lifecycle \`Comum [Acao] → character [Acao] → Comum [AcaoFim]\`.

No field is returned as a fabricated zero merely because the current simulation model does not yet expose it. No unsupported statement/helper is silently treated as a no-op. The temporary research plan tracks those later milestones.

## 🌈 Validation

The parser unit tests compile every section in the pinned \`Comum.rot\` and \`C01.rot\` through \`C15.rot\` files. They also cover nested control flow, sets/ranges, calls, string escaping, hexadecimal literals, typed declarations, optional-section absence, source offsets, and readable source-located errors.

Run the repository checks from the root:

\`\`\`sh
cargo fmt --all -- --check
cargo test --all-targets
\`\`\`


## 🌈 Expression-evaluation boundary

The M2 expression core preserves the integer-versus-Int64 width flag used by the pinned interpreter, wraps assignments to Byte/ShortInt/SmallInt/Word/Integer, keeps Int64 values wide, and converts integer/real values to Double only on assignment. Boolean values remain distinct from numbers. Boolean `and/or` short-circuit; integer `and/or/xor` are bitwise. Comparisons enforce the source's string/Boolean/numeric type rules. Set membership evaluates scalar elements and inclusive integer ranges.

The pure functions `Round`, `Trunc`, `Abs`, `Cos`, `Sin` and `ShortInt` are supported. The pinned FPC 3.2.2 profile uses ties-to-even rounding for `Round`; the evaluator implements that behavior. Rust's current real-number representation is `f64`, whereas the Pascal source uses `Extended` internally, so high-precision differential validation remains a required gate before execution compatibility is claimed.

Explicitly unsupported at this stage: statement execution, assignments in the AST, loops/branches/case execution, P/O field access, global reads/writes, RNG, effect/movement helpers and nested `AcaoJogador`. These operations return source-located errors rather than fabricated values or no-op results.
