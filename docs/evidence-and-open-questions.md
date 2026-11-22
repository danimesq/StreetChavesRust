# Evidence register and open questions

## Evidence labels

Use these labels consistently in documentation, code comments, tests, and issue reports:

- **[original-static]**: directly observed in the original executable's bytes, resources, or metadata.
- **[original-decomp]**: recovered from decompiler output. Useful evidence, but the pseudocode is not original source.
- **[runtime-observed]**: reproduced by running the original game with controlled input or instrumentation.
- **[cross-project]**: independently described or implemented by more than one project.
- **[inferred]**: a reasoned interpretation that still needs confirmation.
- **[unknown]**: not established yet.
- **[port-specific]**: a behavior or implementation detail belonging to one port, not necessarily the original game.
- **[normative-project-source]**: a deliberate project policy selecting Pascal/OpenStreetChaves as the primary source for Rust reconstruction when ports differ; it is not proof that the port is identical to the original executable.
- **[design-question]**: an unresolved architecture or project-policy choice; it is not yet an implementation requirement.

A fact may carry more than one label. Record the exact file, section, commit, binary hash, or test fixture that supports it.

## Initial evidence ledger

| Topic | Current observation | Evidence | Rust consequence |
|---|---|---|---|
| Original runtime | Delphi 5 + DelphiX Windows executable | [original-static], SDL3 project's `docs/spec/00-code-map.md` | Reconstruct behavior, do not port Delphi UI abstractions |
| Logical display | 400×300 | [original-static], SDL3 code map; also used by Pascal port | Use as initial logical coordinate space; verify scaling independently |
| Default fight tick | 28 ms | [original-decomp], SDL3 `docs/spec/05-fight-engine.md`; Pascal docs also state 28 ms | Fixed logical tick; no assumption of 60 simulation FPS |
| Game clock | One displayed second per 25 ticks | [original-decomp], SDL3 fight-engine spec | Separate game timer counter |
| Fighter count | 15 character IDs in recovered PC-game specification | [original-decomp], SDL3 move/fighter specs | Verify complete roster and ID mapping before defining final content tables |
| Player record | Reconstructed 176-byte record with position, state, frame, hit/hurt boxes, health, meter, etc. | [original-decomp], SDL3 `docs/spec/02-player-record.md` | Use typed Rust fields; preserve raw offsets only in research notes |
| Fight update order | The original tick has a specific order for input, AI, stage, hit detection, timers, and physics | [original-decomp], SDL3 `docs/spec/05-fight-engine.md` | Encode order explicitly and test it |
| Fighter scripts | Pascal project uses INI + `.rot` data and `ChavesRoteiro.pas`; Rust project explicitly follows Pascal first, with C as comparison-only | [normative-project-source] for project precedence; [port-specific] for the interpreter itself | Preserve Pascal grammar, interpreter and execution order; record C differences without silently overriding Pascal |
| Hitbox metadata | 379/379 Pascal/C rows match as a multiset, but character 11 action/state 27 timer 1 has two strength-specific rectangles; Pascal filters `.f1/.f2`, while C matches both and the last row wins. Both row-order chains are source-traced through their loaders/generators. | [cross-project] for row multiset; [original-decomp] for the reviewed frame-updater entry point; [port-specific] for each adapter's selector behavior | Preserve row provenance and strength conditions; the selector discrepancy is confirmed between the pinned ports, but original behavior remains open |
| Hurtboxes | 305/305 Pascal `[Corpo]` and C `hurt frame` rows match after coordinate reordering; both implementations retain old values when no row matches | [cross-project] for rows; [original-decomp] for the reviewed Pascal updater path and [port-specific] for the INI/C adapter details | Preserve per-frame values and explicit retain/deactivate/clear operations; verify the original executable later |
| RNG | OpenStreetChaves README pins Lazarus 4.8/FPC 3.2.2. `ChavesRoteiro.SortearValor` dispatches LongInt vs Int64 to FPC `System.Random`; the pinned FPC RTL is MTwist. LongInt consumes 1 MT word; Int64 consumes 2 then modulo. `ModoDeSorteio` records/replays range, width and result. The C port uses a separate Delphi-style LCG. | [normative-project-source] for Pascal runtime semantics; [port-specific] for the C LCG; [unknown] for exact original executable RNG until REA/Ghidra/runtime capture | Use Fpc322Rng as the Rust default; preserve width and raw-word consumption; keep the Delphi LCG adapter experimental; capture Pascal and original dynamic traces |
| Logic comparison | Pascal project has `--gravar` and legacy-vs-script comparison support | [port-specific], OpenStreetChaves README and migration notes | Adopt deterministic trace comparison as a development practice |
| Pascal dynamic trace | Schema-v2 branch compiled with FPC 3.2.2; two fixed-seed 3,000-step captures produced byte-identical JSONL and `logica.txt`; the Rust join reconciled 8,841 calls as 8,415 unique, 0 ambiguous, 0 unmatched and 426 non-.rot contexts | [port-specific] and [runtime-observed] for the Pascal port only; no claim about original `Chaves.exe` | Draws connect to 465 timeline occurrences / 255 unique sites; 422 direct wrapper and 4 Pascal-helper calls remain outside .rot static matching; global RNG scope and per-predicate guard mapping remain open |
| Golden screenshots | C17 project documents fixed-seed headless scenarios and SHA-256 screenshot checks | [port-specific], open-street-chaves README | Keep logical-state hashes and render hashes separate |
| Asset licensing | C17 project explicitly excludes original game assets from its source license | Project README | Keep original game data external; audit every asset source |
| SDL2 platform validation | Pascal README reports Windows 64-bit testing and lists Linux/macOS/Android as pending | Project README | Do not assume SDL abstraction equals cross-platform validation |
| Language boundary and native dependencies | The Rust project still needs to define whether Rust-only applies to shipped code only or also to developer tools. SDL3 is a C library even when called through Rust bindings; the experimental JSONL validator is currently Python and is developer-only. | [design-question], current architecture proposal and experimental research tooling | Keep project-owned game/runtime logic in Rust; isolate native libraries behind Rust adapters; decide explicitly whether Python/tooling exceptions are acceptable or migrate the validator to Rust |
| SMS graphics formats | SMS project documents tile/frame/palette extraction and console-specific limits | [port-specific], SMS README and ROM reference | Reuse methodology only; do not infer PC-game behavior from SMS |

## Important disagreements and caveats

### Normative project source

The user's explicit project policy is to follow the Pascal source when Pascal and C disagree. The Rust implementation therefore treats OpenStreetChaves and its FPC 3.2.2 runtime as the normative reconstruction reference. The C project remains a comparison source, and its discrepancies are marked for original-executable verification. This policy does not claim that Pascal is necessarily identical to the original executable.

### Hitbox/hurtbox cross-port comparison

The pinned Pascal and C snapshots have exact multiset matches for all 379 hitbox rows and all 305 hurtbox rows after coordinate normalization. Code-path tracing establishes retain-on-miss in both ports. However, character 11 action/state 27 timer 1 has two strength-specific rows: Pascal filters by strength, while C scans both and the last row overwrites the first. The 379-row match is therefore data parity, not selector/runtime parity. All of these remain port-level evidence until checked against the original executable.

### README status versus implementation

The C17 + SDL3 README describes a broad completed arcade flow, then lists several screens, stage animations, AI details, and other features as not yet present. This wording needs source-level verification before using the README as a feature-completion checklist.

### Original behavior versus port behavior

The Pascal port's 3× software framebuffer, glyph atlas, custom mixer, and `.rot` interpreter are implementation choices. They are not automatically requirements of the original game.

Likewise, the C port's renderer, generated tables, build system, and headless harness are implementation choices. Their recovered numeric rules may be evidence about the original, but each should be checked against the spec's evidence labels.

### Timing terminology

A 28 ms default tick is approximately 35.7 ticks per wall-clock second. The on-screen game timer's 25-tick counter is a separate game rule. Avoid calling either one "FPS" without qualification.

### SMS adaptation

The SMS project is based on a Master System fighting-game hack and imports Chaves character art from a separate SNES hack. Its tile formats, palette restrictions, sprite limits, and adaptation decisions belong to that console project. They are not authoritative for the original PC game's hitboxes, timing, AI, or state machine.

## Open questions for the Rust project

### Language boundary and native dependencies

- [ ] Define whether "everything in Rust" applies to shipped game/runtime code only, or also to developer tools, generators, validators, build scripts and test harnesses.
- [ ] Decide whether SDL3 and other native C/system libraries are acceptable behind narrow Rust FFI adapters, or whether a different rendering/window/audio stack should be investigated.
- [ ] For each accepted native dependency, choose system linking versus vendoring/build-from-source and document version pinning, licensing, security updates, reproducible-build implications and supported platforms.
- [ ] Decide whether the experimental Python JSONL validator is an approved developer-only exception or should be replaced by a Rust CLI/test tool before the trace workflow is considered permanent.
- [ ] Require every non-Rust project-owned tool to state why it is not Rust, whether it is optional, and whether it adds any runtime or release dependency.

### Original-game compatibility

- [ ] Which original `Chaves.exe` build and SHA-256 will be our canonical reference?
- [ ] Can we obtain repeatable original-game traces for movement, attacks, blocking, throws, hit-stop, and round transitions?
- [ ] Which recovered fields and constants have been runtime-verified, rather than only decompiled?
- [ ] Are there known divergences between the Pascal and C ports, and which one matches the original?
- [ ] Which behaviors are affected by configuration files or unlock files?

### Simulation and numeric behavior

- [ ] Which fields require exact floating-point behavior, and which can use fixed-point safely?
- [x] Implement the Delphi 5 32-bit LCG transition and multiply-high range reduction with an injected seed.
- [x] Inventory C RNG call sites and the C fight-tick phase order, including variable draw-count traps.
- [ ] Inventory all Pascal application RNG calls and normalize them into tick order.
- [x] Pin the Pascal build requirement to FPC 3.2.2 and implement its MTwist Random(LongInt)/Random(Int64) consumption rules in Rust.
- [x] Generate symbolic timelines for all 15 .rot scripts and inventory Pascal RNG sites.
- [x] Compile the experimental Pascal schema-v2 trace branch with FPC 3.2.2 and SDL2.
- [x] Run two fixed-seed 3,000-step `--gravar` captures and require byte-identical JSONL plus matching `logica.txt` checkpoints.
- [x] Validate both real schema-v2 captures and commit a compact tick-316 excerpt as a Rust regression fixture.
- [x] Implement the first Rust reconciliation path and test a unique real draw match, an ambiguous same-line match, and a Pascal-helper non-match.
- [x] Reconcile all 8,841 runtime RNG draws against all 15 fighter timelines; preserve unique, ambiguous, unmatched and non-.rot classifications.\n- [x] Add a strict Rust consumer for schema-v2 recorded draws: preserve API order, validate overload width/range/raw-word cost, reject divergent requests without advancing, and detect leftover draws.
- [x] Validate real LongInt/Int64 draws and script helper/direct-call provenance in the full deterministic capture.
- [ ] Expand compact committed regression fixtures across action sections, nested scripts, loop branches, Int64 draws, helper/direct calls and more fighters. The current tick-316 sample is a real event excerpt, not broad scenario coverage.
- [ ] Decide whether to extend runtime attribution with source columns, direct `System.Random` call sites and fuller game-state snapshots; these remain outside schema v2.
- [ ] Capture the original executable's initial RNG state and algorithm in the later compatibility phase; the C LCG remains comparative only.
- [x] Trace retain-on-miss behavior in Pascal AplicarCaixas and C sel_boxes.
- [x] Trace explicit attack-flag deactivation in both ports separately from lookup misses and full record clearing.
- [ ] Verify the character 11 strength-specific selector behavior against the original executable.
- [ ] Confirm the exact Free Pascal RTL/compiler version used for the Pascal port build; current FPC RTL documentation describes System.Random as Mersenne Twister, but the project snapshot does not pin the compiler build.
- [ ] What is the exact ordering of all fight-tick side effects?
- [ ] Are timer edge cases and pause behavior covered by original-game traces?
- [ ] How should draw-time side effects in the original be represented without making rendering mutate simulation state?

### Content and asset pipeline

- [ ] Which original asset files are required for a minimal playable build?
- [ ] What are the exact sprite-strip slicing rules for all fighters and effects?
- [ ] Which font assets can be redistributed, and under what license?
- [ ] Can asset inspection tools emit provenance and hashes for every extracted file?
- [ ] Which data tables can be generated from the reverse-engineering spec, and how are generators validated?

### Platform and packaging

- [ ] Which SDL3 Rust binding version and feature set will be pinned?
- [ ] How will SDL3 be provided in CI and release builds on each platform?
- [ ] Is browser support a near-term requirement or a later backend?
- [ ] What input mapping should preserve the original arcade controls while supporting modern controllers?

## Validation priority

1. Pin the canonical original executable by hash and record the source of the legally obtained copy.
2. Build a reproducible input/trace capture procedure.
3. Verify the logical resolution, tick interval, and game-clock counter.
4. Verify player state transitions and update order.
5. Verify collision boxes and attack windows for a small representative set of moves.
6. Compare both SDL projects against the same original-game observations.
7. Only then promote uncertain values into Rust constants or data files.

## Source references

- [OpenStreetChaves README](https://github.com/danimesq/OpenStreetChaves/blob/main/README.md)
- [OpenStreetChaves structure and data notes](https://github.com/danimesq/OpenStreetChaves/blob/main/source/LEIAME.md)
- [OpenStreetChaves SDL2 migration notes](https://github.com/danimesq/OpenStreetChaves/blob/main/source/MIGRACAO_SDL.md)
- [open-street-chaves README](https://github.com/danimesq/open-street-chaves/blob/main/README.md)
- [Original executable code map](https://github.com/danimesq/open-street-chaves/blob/main/docs/spec/00-code-map.md)
- [Flow and input](https://github.com/danimesq/open-street-chaves/blob/main/docs/spec/01-flow-and-input.md)
- [Player record](https://github.com/danimesq/open-street-chaves/blob/main/docs/spec/02-player-record.md)
- [Moves](https://github.com/danimesq/open-street-chaves/blob/main/docs/spec/04-moves.md)
- [Fight engine](https://github.com/danimesq/open-street-chaves/blob/main/docs/spec/05-fight-engine.md)
- [Open questions in the C17 project](https://github.com/danimesq/open-street-chaves/blob/main/docs/spec/99-open-questions.md)
- [SMS hack research](https://github.com/danimesq/street-chaves-hack-sms)
