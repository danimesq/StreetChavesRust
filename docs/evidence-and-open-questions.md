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
| Fighter scripts | Pascal project uses INI + `.rot` data and an interpreter | [port-specific], OpenStreetChaves README and `source/LEIAME.md` | Data-driven design is promising; format is not binding |
| Hitbox metadata | 379/379 Pascal/C hitbox rows match in action/timer, geometry, active, strength, damage, and guard damage | [cross-project], pinned Pascal `C01.ini`–`C15.ini` and C `05a-anim-tables.txt` | Preserve complete metadata and duplicate/strength-specific rows |
| Hurtboxes | 305/305 Pascal `[Corpo]` and C `hurt frame` rows match after coordinate reordering | [cross-project], same pinned snapshots | Preserve per-frame hurtbox state and investigate absent-frame retention separately |
| Logic comparison | Pascal project has `--gravar` and legacy-vs-script comparison support | [port-specific], OpenStreetChaves README and migration notes | Adopt deterministic trace comparison as a development practice |
| Golden screenshots | C17 project documents fixed-seed headless scenarios and SHA-256 screenshot checks | [port-specific], open-street-chaves README | Keep logical-state hashes and render hashes separate |
| Asset licensing | C17 project explicitly excludes original game assets from its source license | Project README | Keep original game data external; audit every asset source |
| SDL2 platform validation | Pascal README reports Windows 64-bit testing and lists Linux/macOS/Android as pending | Project README | Do not assume SDL abstraction equals cross-platform validation |
| SMS graphics formats | SMS project documents tile/frame/palette extraction and console-specific limits | [port-specific], SMS README and ROM reference | Reuse methodology only; do not infer PC-game behavior from SMS |

## Important disagreements and caveats

### Hitbox/hurtbox cross-port comparison

The pinned Pascal and C snapshots have exact multiset matches for all 379 hitbox rows, including active/strength/damage/guard-damage metadata, and all 305 hurtbox rows after coordinate normalization. This is [cross-project] evidence only. It does not establish original executable activation timing, facing/mirroring, collision edge semantics, or runtime stale-box behavior.

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

### Original-game compatibility

- [ ] Which original `Chaves.exe` build and SHA-256 will be our canonical reference?
- [ ] Can we obtain repeatable original-game traces for movement, attacks, blocking, throws, hit-stop, and round transitions?
- [ ] Which recovered fields and constants have been runtime-verified, rather than only decompiled?
- [ ] Are there known divergences between the Pascal and C ports, and which one matches the original?
- [ ] Which behaviors are affected by configuration files or unlock files?

### Simulation and numeric behavior

- [ ] Which fields require exact floating-point behavior, and which can use fixed-point safely?
- [ ] Can we reproduce Delphi's RNG sequence exactly for a given seed?
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
