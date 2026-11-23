# Temporary ROT Runtime Research and Delivery Plan

> **TEMPORARY PR DOCUMENT — remove progressively as work is delivered.**
>
> This file is a research ledger and implementation plan for PR #20, not a permanent language specification and not evidence that the runtime exists. When a milestone is delivered, move only the durable contract/evidence into the relevant permanent module documentation, mark the corresponding section below as delivered, and remove that completed planning material. Delete this entire file after all acceptance gates are satisfied and the remaining durable facts have been relocated.

- Target: [PR #20 — runtime: common actions and C13 extra commands](https://github.com/danimesq/StreetChavesRust/pull/20)
- Research date: 2026-11-23
- Status: M0 baseline and M1 parser are delivered; M2 scalar/local execution is in progress. `src/rot_runtime.rs` parses every section in the pinned `Comum.rot` and C01-C15 files and executes a bounded statement subset over typed local frames. Fighter/global bindings, game-helper bridge, RNG and character action-script execution are not claimed.
- Normative Pascal source: [OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves), commit `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`.
- Scope: `Comum.rot [Acao]`, character `C01.rot` through `C15.rot [Acao]`, and `Comum.rot [AcaoFim]`; execute real character action scripts, including C02-C15 specials, through the Rust simulation.
- Out of scope for this delivery: implementing all `[Quadro]`, `[IA]`, or `[Acerto]` sections as part of the action-entry milestone; C16 Chapolin; claiming equivalence with the original Delphi 5 executable.

## 1. Why this plan exists

The current Rust branch has a real fight orchestrator, per-character INI command tables, common basic-action handling, a hand-ported C01 special dispatcher, and hand-ported C13 extra commands 25/26. It does **not** yet execute arbitrary character `[Acao]` sections.

The missing layer is not a generic "special move" switch. The source scripts contain per-character eligibility guards, independent sequential `if` statements, typed assignments, loops, nested engine calls, effects, sounds, movement, power consumption, and RNG. The correct next unit is a reusable Rust interpreter for the actual ROT dialect plus a simulation adapter for the fields and engine helpers that the scripts can access.

This plan intentionally separates three things:

1. **Language execution:** parse and evaluate the real ROT source with Pascal-compatible semantics.
2. **Game integration:** expose the correct fighter/opponent/global state and engine helpers, and invoke scripts in the original order.
3. **Evidence:** compare ordered writes, side effects, and RNG requests against the pinned Pascal implementation and captured executions.

A parser, a symbolic timeline, a hand-coded command switch, or a test that merely checks the final action ID is not by itself proof of runtime compatibility.

## 2. Evidence policy and source pins

### 2.1 Authority

- The pinned Pascal/OpenStreetChaves source is normative for this Rust reconstruction.
- The C17/SDL3 project is comparative evidence only. It must not silently override Pascal.
- The original executable remains a separate later verification target. A Pascal-port match is not automatically an original-executable match.
- The SMS ROM hack is not evidence for PC fighter mechanics.
- C16 Chapolin is a separately documented Pascal-only generated extension and is not part of the C01-C15 acceptance gate.

### 2.2 Pinned inputs

Primary source files:

- `source/game/ChavesRoteiro.pas`: ROT lexer/parser, AST, evaluator, typed memory access, function bridge, RNG wrapper and section loader.
- `source/game/ChavesTipos.pas`: packed `TLutador` record, field types and constants.
- `source/game/ChavesGolpes.pas`: script loading and the authoritative `AcaoJogador` composition.
- `source/game/Estado7_Golpes.inc`: command-rule evaluation, button dispatch and sequence counters.
- `source/game/Estado7_Luta.inc`: human input ordering and fight-loop gates.
- `Dados/Lutadores/Comum.rot`: shared action entry and action-end sections.
- `Dados/Lutadores/C01.rot` through `C15.rot`: character-specific action scripts.

The Rust repository already contains the pinned copies under `research/pascal-rot/` and a source manifest. Keep that manifest as the reproducibility boundary; do not silently refresh the source files while implementing the runtime.

### 2.3 Branch synchronization note

At the time of this audit, GitHub reported the PR head branch as nine commits behind `main`. This research-only change does not rebase, merge, or rewrite the branch. Before implementation begins, reconcile the PR branch with the current base, inspect the intervening changes to the combat/runtime/docs files, and rerun the source audit against the exact branch that will receive the implementation.

## 3. What already exists, and what it does not prove

| Existing component | What it provides | What it does not provide |
|---|---|---|
| `tools/rot-normalize.py` and `docs/rot-normalizer.md` | Static parsing, AST-like structure, source spans, nested guards/loops, symbolic action/tick lanes and RNG-site inventory. | No expression evaluation, mutable fighter state, engine calls or concrete action execution. |
| `src/rot_runtime.rs` and `docs/rot-runtime.md` | Rust lexer, section loader and source-preserving typed AST parser; tests parse every pinned section. | No evaluator, mutable fighter state, engine calls or concrete action execution. |
| `research/pascal-rot/` | Pinned real `Comum.rot` and C01-C15 scripts plus source manifest. | No executable runtime by itself. |
| `src/pascal_replay.rs` | Strict replay of captured Pascal RNG requests, including width/range checking. | Not a ROT interpreter and not a source of guessed initial state. |
| `src/combat_runtime.rs` | A bounded executable slice of common Pascal AI behavior and a child-script seam that shares the parent's RNG cursor. | Not a general ROT parser/evaluator; the child seam is not the actual C02-C15 `[Acao]` implementation. |
| `src/chaves_specials.rs` | Audited hand-port of C01 special entry and C01 AI special dispatch. | Not a substitute for executing C01.rot, and not a general character runtime. |
| `src/fight_simulation.rs` | Headless fight tick, normalized input, ordered command recognition, common basic actions, C01 special dispatch, C13 extra 25/26 dispatch, and explicit unhandled-command reporting. | Does not execute the remaining character action scripts. |
| `src/fighter_command_data.rs` and `data/fighter-command-rules.ini` | Per-character `[Sequencias]`/`[Botoes]` extraction and rule evaluation. | These INI rules choose command codes; they do not implement the ROT action behavior those codes invoke. |

Important distinction: the Python normalizer and the future Rust interpreter may share grammar fixtures and source metadata, but they have different jobs. The normalizer is a research/indexing tool. The Rust runtime must execute the actual pinned script semantics against live simulation state.

## 4. Exact action-call lifecycle

### 4.1 Authoritative Pascal call order

In `source/game/ChavesGolpes.pas`, `AcaoJogador(Jog, Codigo)` performs these operations in order:

1. Execute shared `Comum.rot [Acao]`, if present.
2. Resolve the active fighter's character ID and execute `CNN.rot [Acao]`, if present.
3. Execute shared `Comum.rot [AcaoFim]`, if present.

The source loader loads every shared section from `Comum.rot`. For each character it loads the character sections except `[AcaoFim]`; the action-end section is shared. A missing character section is represented by no script (`nil`), not by synthesized behavior.

At script entry:

- `P` is `Lutador[Jog]`.
- `O` is `Lutador[3 - Jog]`.
- `Jog` is 1 or 2.
- `Codigo` is narrowed to a Pascal `Byte`.
- `Desloc` is 0 for player 1 and 50 for player 2.
- Each invocation gets fresh local variables; nested invocations get their own execution frame.
- The shared game globals and the same underlying RNG stream remain shared.

### 4.2 Shared prelude: `Comum.rot [Acao]`

The first statement is:

```pascal
P.Defendendo := False;
```

The rest of the common section handles movement, basic standing/crouching/air attacks, hit reactions, knockdown/bounce, and guard entry. This means a character-specific special can be rejected by its own guard after the common prelude has already cleared `P.Defendendo`. A Rust dispatcher that checks a character guard first and returns without the common prelude is observably wrong.

The common section also has ordered independent branches. Preserve their source order and do not turn them into a mutually exclusive `match` unless the source proves exclusivity.

### 4.3 Shared epilogue: `Comum.rot [AcaoFim]`

The complete section advances the walking animation only when the fighter is still in `acAndando`:

- Increment `P.QuadroAcao`.
- If the result is greater than 16, wrap it to 0.

This runs after the character script, including when the character script rejects a command and leaves the fighter walking. Tests must cover that rejected-command path, not only successful special entry.

### 4.4 Separate execution phases

Do not accidentally fold these into the action-entry runtime:

- `[Quadro]`: frame/tick behavior, sprite selection, effects, sounds, movement, and `AplicarCaixas`.
- `[IA]`: CPU decisions, called through `ExecutarIA` with five input values.
- `[Acerto]`: character hit reaction, called only from the confirmed hit path and only when that section exists.

The parser should be designed around the full documented ROT dialect, but the first integration gate in this PR is the action lifecycle above. Other sections remain explicitly out of scope until their own callers and phase ordering are audited.

## 5. ROT interpreter contract

### 5.1 Runtime values and declared types

The Pascal interpreter distinguishes four runtime value classes:

- integer, with a separate flag for 64-bit integer values;
- real;
- Boolean;
- string.

The declared local/field/global types include:

- `Byte`, `ShortInt`, `SmallInt`, `Word`, `Integer`/`LongInt`, `Int64`;
- `Double`;
- `Boolean`.

String literals are runtime values and are used by the effect helper's variant argument. They are not a general mutable ROT variable type in the current type-name loader.

Preserve assignment conversion rules:

- integer-to-integer assignment narrows/wraps according to the destination Pascal type;
- integer-to-`Double` and real-to-`Double` are accepted;
- real-to-integer assignment is rejected by the interpreter;
- Boolean fields require Boolean values;
- a type mismatch raises a script error with source origin and line.

Do not normalize every number to Rust `f64` or every integer to `i64`. The integer-width distinction changes RNG behavior and arithmetic.

### 5.2 Statements and expressions

The pinned interpreter supports:

- declarations at the start of a section: `var Name: Type;`;
- assignment and procedure/function calls;
- `if / then / else`;
- `begin / end` blocks;
- `for / to / downto / do`;
- `while / do`;
- `repeat / until`;
- `case / of / else`;
- unary operators, binary operators and Pascal precedence;
- set membership, set elements and ranges, such as `P.Acao in [acParado, acAndando]`;
- integer, real, hexadecimal and quoted string literals;
- semicolon-prefixed whole-line section comments, `//` comments and brace comments.

The operator inventory includes arithmetic, integer division/modulo, shifts, comparisons, `in`, unary signs, `not`, and `and/or/xor`.

The AST in `ChavesRoteiro.pas` distinguishes constants, locals, globals, fighter fields, unary/binary expressions, function calls, sets/ranges, assignment, call, conditional, block, for, repeat, while, case and case-arm nodes. A Rust AST should retain source file, section, line and column so runtime errors and test diffs can point back to the real script.

### 5.3 Evaluation details that must not be simplified away

- **Boolean versus integer `and/or`:** when the left operand is Boolean, `and/or` short-circuits. When it is integer bitwise logic, both operands are evaluated.
- **Argument evaluation order:** `ChavesRoteiro` probes the actual FPC compiler at initialization and stores `OrdemArgsInversa`. The pinned FPC 3.2.2 x86_64 Linux CI probe observed right-to-left evaluation. Treat that as a measured compiler/target profile, not a universal Pascal rule.
- **Function-call side effects:** argument evaluation can consume RNG or invoke nested functions. The evaluation order therefore affects gameplay and draw order.
- **`case`:** evaluate the selector once, test arms in source order, execute the first matching arm, and otherwise execute `else` when present.
- **`for`:** evaluate initial and final bounds once before the loop; use the declared integer local; advance or decrement according to `to/downto`.
- **`repeat`:** execute the body before checking the terminating condition.
- **`while`:** check the condition before each body execution.
- **Locals:** initialize each execution frame to zero/false according to its declared type; do not leak local values between calls.
- **Nested calls:** `AcaoJogador` can be invoked from scripts. A child call gets a new local frame but mutates the same fighters/globals and consumes the same shared RNG cursor.
- **Source order:** sequential `if` statements and loop bodies must execute exactly in order. Do not reorder writes by field, event category or apparent importance.

### 5.4 Built-in functions and engine bridge

The pinned interpreter exposes these twelve function/procedure names:

| Function | Role in the Rust adapter |
|---|---|
| `Random` | Pascal RNG request, preserving LongInt versus Int64 width. |
| `Round`, `Trunc` | Pascal numeric conversion semantics. |
| `Abs`, `Cos`, `Sin` | Numeric functions. |
| `ShortInt` | Explicit signed 8-bit conversion. |
| `ExisteEfeito` | Query the live effect state by type and owner. |
| `CriaEfeito` | Create a real effect with type, variant, owner and coordinates. |
| `Empurra` | Apply the source movement/separation/bounds behavior. |
| `AcaoJogador` | Nested action dispatch using the same lifecycle and shared RNG stream. |
| `AplicarCaixas` | Apply the current frame's collision-box data; required by frame execution, not a substitute for action entry. |

The host bridge must mutate the same `CombatState` used by the fight simulation. Do not maintain a parallel "script state" or an event-only effect list that can disagree with the live effect arena.

Errors must retain script origin and source line. Do not silently convert an unsupported helper or type into a no-op. Do not roll back earlier writes from a script failure unless source evidence establishes transactional behavior; the Pascal interpreter executes mutations directly.

### 5.5 Fighter fields and globals

The Pascal interpreter registers a much wider surface than the current Rust `FighterState`. The field adapter must be audited against `MontarTabelas` in `ChavesRoteiro.pas`, not just the fields used by the first C01 test.

Registered fighter fields include position, character ID, angle/angular velocity, AI distance, body and attack-box coordinates, health and health-bar values, power and charge, active-hit flags, kick flag, airborne/action/frame/duration/facing/jump/force/damage/recoil fields, sprite height, movement speed, score/round/victory/bonus state, draw order, invulnerability/defending/front-walk state, combo counters, sequence counters, directional hold counters, `CargaSoco`, `CargaChute`, and active-player state.

Registered globals include `CameraX`, `CameraY`, `ChaoY`, `ContPisca`, `Flash`, `FlashTela`, `FlashVermelho`, `GanhoCarga`, `NivelDano`, `PausaImpacto`, `SomPendente`, `TremorTela`, `Gravidade`, `NivelJogo`, `Fase`, and `Sangue`.

The current Rust fighter model is intentionally smaller. Before executing real scripts, either add the missing source-visible state to the simulation model or provide a typed adapter that maps every registered ROT field to authoritative state. Missing fields must be reported explicitly; do not return zero for an unimplemented field.

## 6. Character action audit: C02-C15

This table records distinguishing source behavior, not a replacement for the source files. The complete scripts and their source hashes remain in `research/pascal-rot/`. The command groups use the Pascal IDs: Esp1 = 16/17, Esp2 = 18/19, Esp3 = 20/21, Esp4 = 22/23, final = 24, and C13 extras = 25/26.

| Fighter | Source-specific `[Acao]` behavior that must survive execution |
|---|---|
| **C02 — Seu Madruga** | Esp1 is guarded by absence of effect type 9. Esp2 accepts several neutral/defense states and combo windows, including weak/strong punch and a late Special 6 window. Esp3 is limited to jump/air weak-kick states and marks the move as a kick. Esp4 weak is a 45-frame action; Esp4 strong instead enters `acEspecial6`, checks the low-health/power gate, consumes power, sets frame offset +44, flash and final-special sound. |
| **C03 — Chiquinha** | Esp2 launches vertically with strength-dependent `VelY`, marks airborne/kick state and sound. Esp3 duration differs by weak/strong force (14/19). Esp4 is another airborne launch with strength-dependent velocity and +6 charge. Final is 15 frames and explicitly sets sprite offset +48 before flash/sound and power consumption. |
| **C04 — Quico** | Esp1 is guarded by effect type 12. Esp2 is a kick with duration 13/16 by force. Esp4 is an airborne kick with velocity -26/-32. Final forces strong/kick state, consumes power, sets frame offset +12, clears opponent invulnerability, and requests flash/final sound. |
| **C05 — Dona Florinda** | Esp3 has a landing-height guard; it snaps Y to `ChaoY`, clears vertical/air state and `TempoBaixo`, then uses duration 38/40 by force. Esp4 explicitly resets `P.CargaChute := 0`. Final has the shared power gate and frame offset +6. |
| **C06 — Prof. Girafales** | Esp1 and Esp2 are independently guarded by absence of effect type 15. Esp3 clears `NoAr`; Esp4 has its own 30-frame action. Final launches with `VelY := -18`, sets airborne, and uses frame offset +41. |
| **C07 — Sr. Barriga** | Esp1 and Esp2 each reset `P.CargaSoco := 0`; Esp1 duration is 17/20 by force. Esp3/Esp4 write Y, jump direction, `TempoBaixo`, airborne state and strength-dependent vertical velocity. Final is only 12 frames and uses frame offset +38. |
| **C08 — Bruxa do 71** | Esp1 is guarded by effect type 18. Esp2 is accepted only while jumping and only when effect type 19 is absent. Esp3/Esp4 have their own 20/25-frame actions. Final is 20 frames and uses frame offset +33. |
| **C09 — Paty** | Esp1 duration is 14/23 by force. Esp3 creates five type-2 effects in a `for` loop; each spawn evaluates two RNG-bearing coordinate expressions. The exact argument-evaluation profile and draw order must be preserved. Esp4 is restricted to `acPulo` and clears `NoAr`. Final is 40 frames and uses frame offset +19. |
| **C10 — Nhonho** | Esp1 resets `P.CargaSoco := 0`. Esp2 duration is 11/20 by force. Esp3 has a landing-height guard, writes `Y := ChaoY - 50`, sets airborne and uses `VelY := -25/-28`. Final is 40 frames and uses frame offset +38. |
| **C11 — Godines** | Esp1 is guarded by absence of effect type 22. Esp2 adds 6 charge. Esp3 duration is 9/15 by force. Esp4 can chain from `acEspecial5` when its action frame is at least 27. Final is 45 frames and uses frame offset +6. |
| **C12 — Sr. Furtado** | Victory entry also chooses facing from camera-relative X. Esp2 adds 6 charge; Esp3 clears vertical velocity, airborne state and jump direction. The action script contains a `cmEsp4Fraco` final-like branch with power cost/frame offset +38; it does not define the usual `cmEsp4Forte` or `cmEspFinal` branches. Preserve the actual command mapping instead of assuming a uniform roster template. |
| **C13 — Jaiminho** | Esp2 can alter vertical velocity and clears jump direction. Esp4 weak/strong, final and Extra1 all share one branch entering `acEspecial4`; the branch assigns weak, strong, special, or literal force value 4 according to the original command. Extra2 enters `acEspecial5`, clears airborne/jump/vertical state, consumes power, sets frame offset +37, flash and final sound. |
| **C14 — Dona Neves** | Esp3 is accepted only from `acPulo`; Esp3/Esp4 clear airborne and vertical velocity. The Esp2 branch does **not** assign weak/strong `ForcaGolpe`, unlike many neighboring fighters. Final duration is 70. Do not fill in the missing force assignment by convention. |
| **C15 — Glória** | Esp3 can start from a strong kick after action frame 6, in addition to neutral/defense states. Final can start from Special 1 after action frame 24, as well as its neutral/defense states. These are explicit cancel windows, not generic eligibility. |

Additional roster-wide observations:

- The action scripts use independent sequential `if` statements. A command may satisfy more than one branch; preserve the full ordered evaluation rather than choosing one "move handler".
- C02-C12 contain a `cmPose` entry branch; C13-C15 do not. Missing branches remain missing.
- Victory entry is common across the original C01-C15 scripts: idle/walking/crouching eligibility, action = victory, action frame = 0, duration = 250. The separate victory `[Quadro]` behavior is character-specific and outside this action-entry scope.
- C02-C15 have meaningful differences in allowed source actions, durations, force writes, charge increments, effect guards, movement resets, sound IDs and power use. The table is a map for focused tests; the scripts remain the executable truth.

## 7. C01 and C13 regression strategy

### 7.1 C01

C01 currently has a hand-ported special dispatcher in `src/chaves_specials.rs`. Keep it as a compatibility oracle while the script interpreter is introduced.

Required comparisons:

- Execute the pinned C01 `[Acao]` script and the current Rust hand-port from identical pre-state and command inputs.
- Compare every script-visible fighter/global field, not only action ID and duration.
- Cover accepted and rejected paths for each special, effect type-7 ownership, health/power gates, weak/strong force, action-frame reset, charge gain, final-special power consumption, sprite offset, flash and sound.
- Verify common `P.Defendendo := False` and common `[AcaoFim]` order on rejected and accepted commands.
- Keep C01 `[Quadro]` and `[IA]` as separate milestones; this PR's action interpreter gate must not imply they are complete.

### 7.2 C13

The current hand-port implements C13 extra commands 25/26. Preserve it as a regression oracle, but compare it to actual C13.rot execution.

Required comparisons:

- Extra1 (25): exact allowed action set, action/frame/duration, charge gain, and literal force value 4.
- Extra2 (26): exact allowed action set, low-health/power guard, action/frame/duration, air/jump/vertical resets, power and charge behavior, sprite offset +37, flash and final sound.
- Add a case with `Defendendo = true` before command entry. The Pascal common prelude clears it before the C13 branch. The current manual C13 branch should be checked for that common-prelude write; its existing `finish_common_action` call alone represents only the epilogue.
- Test C13 commands 16-24 as well. The existing extra-command hand-port does not mean the rest of C13's `[Acao]` has been implemented.

### 7.3 No fake completion

Do not remove the old hand-port merely because the interpreter can parse C01/C13. Remove or demote it only after source-driven differential tests establish the same ordered writes and side effects for the relevant command surface.

## 8. RNG and deterministic execution evidence

The Pascal script interpreter routes `Random` through `SortearValor`, which supports normal execution, recording (`msGravar`) and replay (`msRepetir`). Replay validates both the requested range and whether the call is 32-bit or 64-bit. A mismatch marks divergence and returns to normal RNG behavior; the Rust strict replay should instead fail the fixture immediately and make the mismatch visible.

Important width distinction:

- `Random(2)` with an ordinary integer literal is a LongInt request.
- `Random(Round(...))` receives the 64-bit integer result of `Round` and follows the Int64 request path.
- The pinned FPC 3.2.2 MT implementation consumes one MT word for LongInt and two words for Int64.
- The FPC RNG is the normative Pascal-port oracle for this project. It is not the same as the original Delphi 5 executable's RNG.

The existing schema-v2 trace work records tick/script nesting, control decisions/loop values and RNG requests. It can prove call-site/draw reconciliation, but the current documentation explicitly says it does not yet provide complete game-state snapshots or prove every dynamic branch evaluation. For action compatibility, extend the Pascal capture with pre/post snapshots of the script-visible state and ordered mutation records. Do not fabricate those snapshots from the static AST.

A single mutable RNG cursor must flow through the caller, ROT evaluation, nested `AcaoJogador` calls and any helper that can consume randomness. A child script must not get a new seed or independent cursor.

## 9. CargaSoco and CargaChute investigation

### 9.1 What the pinned source establishes

- `TLutador.CargaSoco` is a `Byte` documented in `ChavesTipos.pas` as a decreasing counter for buttons 1/3 (punch).
- `TLutador.CargaChute` is a `Byte` documented as a decreasing counter for buttons 2/4 (kick).
- `Estado7_Golpes.inc::RegraVale` reads them only as threshold predicates: `CargaSoco >= N` and `CargaChute >= N`.
- `L7_Entrada` handles directional input, button press edges and calls `golpe`/ `AcaoJogador`; the inspected routine does not update either charge field.
- `sequenciagolpe` updates `TempoBaixo`, `TempoTras`, `SeqEstado` and `SeqTempo`. It does not update either charge field.
- `AcaoJogador` composes common `[Acao]`, character `[Acao]`, and common `[AcaoFim]`; it does not itself implement an extra counter clock outside those scripts.
- The character scripts contain resets:
  - C05: `P.CargaChute := 0` in its Esp4 branch.
  - C07: `P.CargaSoco := 0` in its Esp1 and Esp2 branches.
  - C10: `P.CargaSoco := 0` in its Esp1 branch.
- The interpreter registers both fields as writable `Byte` fields.

### 9.2 What remains unknown

No producer, increment, decrement or repeat-button timing routine was found in the inspected pinned input/fight/interpreter paths. The field comments say "decreasing counter", but that comment does not establish the initial value, who changes it, when it changes, whether it changes per logical tick or input event, or whether the Pascal source port omitted a behavior recovered from the original executable.

This is a bounded negative result for the inspected revision and call paths, not proof that no such routine exists anywhere in the project or original executable.

### 9.3 Next investigation steps

1. Run a repository-wide exact search on the pinned source checkout for every read and write of both fields across all `.pas`, `.inc`, `.rot` and initialization code. Classify each occurrence as declaration, read, reset, initialization, increment or decrement.
2. Trace the complete host-to-game input path, including key-state updates, `L7_Entrada`, button-edge state, the timer/event callback and any fighter initialization/reset path.
3. Inspect the original executable's corresponding field offsets and write sites when that artifact is available; keep this evidence separate from the Pascal reconstruction.
4. Capture held and released punch/kick sequences with field snapshots on every logical tick. Include both players, all four buttons, hit-stop, screen flash, pause, character changes and round resets.
5. Only after a concrete producer and cadence are observed should Rust update these fields. Until then, preserve them as explicit state inputs and do not invent a seed, decay rate, hold threshold, repeat policy or reset rule.

## 10. Test and fixture plan

### 10.1 Parser tests

- Every token/operator/comment/literal form present in the pinned scripts.
- Case-insensitive reserved words and identifiers where the Pascal parser permits them.
- Nested blocks and all supported control-flow statements.
- Sets, ranges and membership.
- Local declarations, initialization and typed assignments.
- Source spans and readable parse errors.
- Parse all pinned `Comum.rot` and C01-C15 sections; absent optional sections must remain absent.
- Golden AST/normalizer cross-checks may help, but they are parser evidence only.

### 10.2 Evaluator semantic tests

- Integer versus real arithmetic and division.
- Byte/ShortInt/SmallInt/Word/Integer/Int64 conversion and overflow behavior.
- Boolean short-circuit versus integer bitwise evaluation, with side-effecting RHS expressions.
- Function argument evaluation order under the measured FPC profile.
- `case` first-match and else behavior.
- `for` bound evaluation, `to` and `downto`; `while`; `repeat/until`.
- Local variables reset per invocation.
- Nested `AcaoJogador` frames retain parent locals and share global state/RNG.
- Error origin, section and line preservation.

### 10.3 Engine bridge tests

- `ExisteEfeito` reads the live effect arena and matches type/owner exactly.
- `CriaEfeito` creates the expected live effect with correct variant, owner, position and allocation order.
- `Empurra` matches the audited movement/separation/bounds contract.
- `AcaoJogador` recursively executes common action, character action and common action-end in order.
- `AplicarCaixas` is wired only to the separately audited frame path.
- Global writes for sound, flash, camera and other registered values mutate the actual simulation state.

### 10.4 Action integration fixtures

Create source-derived Pascal fixtures with:

- script source pin, fighter ID, section, command code and player index;
- full pre-state for P and O plus every relevant global;
- ordered control-flow decisions and helper calls;
- ordered RNG requests (width, range, value, script execution ID and parent execution ID);
- ordered state writes or complete post-state;
- effect arena before/after and sound/flash requests.

Required scenario families:

- every C02-C15 special command that exists in that character's script, accepted and rejected;
- each distinct eligibility boundary (action state, frame cancel, jump/ground/height, effect-exists, health/power);
- weak/strong variants where the script actually distinguishes them;
- power = 0, power = 3, and low-health final-special branches;
- C05/C07/C10 charge-counter resets;
- C09 loop and RNG argument order;
- nested action calls and scripts that consume RNG;
- player 1/player 2 `Desloc` and effect ownership;
- common action-end when walking remains active after a rejected character branch.

A fixture with only a final action ID is insufficient. Compare every registered field touched by the script, global side effects, effect allocations and RNG cursor.

### 10.5 Required regression suite

- Existing common basic attack tests remain unchanged and pass.
- Existing C01 special entry/frame/effect tests remain unchanged and pass.
- Existing C13 Extra1/Extra2 tests remain unchanged and pass.
- Add interpreter-vs-hand-port differential tests for C01 and C13.
- Add per-fighter command-entry tests for all C02-C15 scripts.
- Assert unsupported/missing behavior is reported explicitly; no generic special handler may silently consume an unknown command.
- Run `cargo fmt --all -- --check`, `cargo test --all-targets`, Python normalizer tests, and the Pascal capture/replay/differential checks relevant to the changed scope.

## 11. Delivery sequence and removal policy

| Milestone | Deliverable | Acceptance evidence | Remove from this temporary document when... |
|---|---|---|---|
| M0 — branch/source baseline | **Delivered.** Reconciled the PR branch with current `main`, verified the pinned source manifest, and reran baseline CI. | Initial comparison: 12 ahead / 0 behind; the original Rust workflow passed. The latest verified branch comparison is 20 commits ahead / 0 behind. | Baseline facts are recorded in permanent provenance docs and implementation branch is synchronized. |
| M1 — ROT parser | **Delivered.** Added `src/rot_runtime.rs`: source-preserving lexer, section loader, typed literal/declaration AST, and parser for the pinned ROT statement/expression grammar. | Tests parse every section in `Comum.rot` and C01-C15; nested grammar, literal, type, optional-section, span and error-location tests. `cargo fmt --all -- --check` and `cargo test --all-targets` both passed on head `a363d1f5bfefcebf8fadc0acaf45a68b53a07ad3`. | Parser and current local-execution boundaries are documented in `docs/rot-runtime.md`. |
| M2 — evaluator | **In progress.** Added typed per-call local frames, Pascal assignment narrowing, scalar expressions, sets, pure math/conversion functions, local assignment and execution for blocks, if, for, while, repeat and case. | CI on head `db40658abe385bc4899ccec8cdeae293dcc20c2a` passed `cargo fmt --all -- --check` and `cargo test --all-targets`. The ROT module has 58 passing tests, including control-flow, source-order and loop-budget cases. Fighter/global bindings, engine helpers, RNG, FPC differential probes and full Extended-precision equivalence remain open. | Full evaluator semantics and differential evidence are moved to permanent runtime docs. |
| M3 — execution context and helpers | P/O/local/global bindings, engine helper bridge, error context and one shared RNG cursor. | Helper integration tests and nested-call/RNG tests pass. | Runtime API and bridge ownership are documented in permanent architecture docs. |
| M4 — action lifecycle | Common `[Acao]` → character `[Acao]` → common `[AcaoFim]`. | Common prelude/epilogue ordering tests, including rejected commands. | Lifecycle is implemented and permanently documented beside fight simulation. |
| M5 — roster execution | Execute actual C01-C15 `[Acao]` scripts, including C02-C15 specials. | Per-character fixtures, state diffs, side-effect/RNG comparisons; no generic fallback. | All 15 action scripts pass their gates and the roster matrix is updated. |
| M6 — counter archaeology | Establish whether/where/how `CargaSoco` and `CargaChute` are produced/decremented. | Source-wide write inventory plus captured or original-executable evidence. If still unresolved, preserve explicit unknown status. | The rule is established and implemented/tested, or the open question is transferred to the permanent evidence register. |
| M7 — cleanup | Remove temporary plan and stale pending claims. | All acceptance gates are closed; PR body/docs index updated; no unresolved plan-only claims remain. | Delete this file. |

## 12. Acceptance checklist

- [x] The PR branch is reconciled with the current base and the pinned source manifest is verified.
- [x] Rust parses the actual `Comum.rot` and C01-C15 action sections into ASTs. (M1 parser acceptance; execution remains pending.)
- [ ] Rust evaluator matches the complete pinned Pascal value/type/control-flow semantics. (M2 scalar/local statement subset is implemented; host bindings, helper calls and full differential validation remain open.)
- [ ] P/O/local/global access covers every field/function reachable from the action scripts.
- [ ] `Random` preserves LongInt/Int64 width, range and the single nested RNG cursor.
- [ ] `ExisteEfeito`, `CriaEfeito`, `Empurra` and nested `AcaoJogador` use authoritative simulation state.
- [ ] Action dispatch executes common `[Acao]`, character `[Acao]`, common `[AcaoFim]` in that order.
- [ ] C01 hand-port regression matches actual C01.rot action execution.
- [ ] C13 Extra1/Extra2 hand-port regression matches actual C13.rot action execution.
- [ ] C02-C15 specials execute their real scripts, including all per-character guards and side effects.
- [ ] Pascal-derived fixtures compare state mutations, effects, sounds, globals and RNG, not only final action IDs.
- [ ] `CargaSoco`/`CargaChute` behavior is established from evidence or explicitly remains an unresolved input field; no guessed rule is merged.
- [ ] Existing Rust tests, normalizer tests and relevant Pascal replay/differential checks pass.
- [ ] Durable findings have been moved to permanent docs and this temporary plan is deleted.

## 13. Source references

Pinned Pascal source:

- `source/game/ChavesRoteiro.pas`: interpreter types/AST, parser, evaluator, built-ins, `ExecutarCom`, `CarregarRoteiro`, `MontarTabelas` and compiler argument-order probe.
- `source/game/ChavesTipos.pas`: `TLutador` and counter field declarations.
- `source/game/ChavesGolpes.pas`: script loader, `AcaoJogador`, `AtualizaQuadro`, `ExecutarAcerto` and `ExecutarIA`.
- `source/game/Estado7_Golpes.inc`: `RegraVale`, `golpe`, `sequenciagolpe` and CPU/defense command calls.
- `source/game/Estado7_Luta.inc`: `L7_Entrada` and fight-loop gating/order.
- `Dados/Lutadores/Comum.rot`, `C01.rot`-`C15.rot`: actual script bodies.

Current Rust branch:

- `src/combat_runtime.rs`
- `src/chaves_specials.rs`
- `src/fight_simulation.rs`
- `src/command_recognition.rs`
- `src/fighter_command_data.rs`
- `src/pascal_replay.rs`
- `tests/fight_simulation.rs`
- `docs/mercedes-action-dispatch.md`
- `docs/rot-normalizer.md`
- `docs/first-rust-runtime-slice.md`
- `docs/pascal-reference-trace-harness.md`
- `docs/c01-character-action-slice.md`
- `docs/c01-remaining-integration-gates.md`

