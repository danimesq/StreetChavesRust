# Comparative analysis: the SDL implementations

## Scope and methodology

This comparison focuses on the two independent SDL-based projects:

- **OpenStreetChaves**: Lazarus / Free Pascal, migrated from the DelphiX-era game to SDL2.
- **open-street-chaves**: C17 + SDL3, reconstructed from the original `Chaves.exe` using static analysis and Ghidra.

The SMS project is deliberately secondary. It is useful for reverse-engineering methods and asset work, but its Master System adaptation changes the execution environment and cannot establish the original PC game's gameplay rules.

The projects are not successive versions of one codebase. Similarities may reflect the same original game, while differences may reflect implementation decisions, different evidence, or bugs. Each claim needs provenance.

---

## 1. OpenStreetChaves: Pascal reconstruction on SDL2

Repository: [danimesq/OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves)

### Runtime architecture

The project documents a clean separation between an SDL host and the game object:

- `ChavesSDL.pas` owns the SDL window, event loop, keyboard/gamepad input, fullscreen handling, and presentation.
- `ChavesJogo.pas` owns `TJogo`, the game logic.
- `source/game/` contains the game types, constants, moves, resource handling, graphics, audio, text, and per-screen state code.
- `Estado0..12_*.inc` split the arcade flow into individual game states.
- The game uses a logical 400×300 coordinate space and renders into a software buffer at a configurable scale, currently documented as 3× (1200×900).
- The SDL host presents that buffer through an SDL texture.

This is a useful architectural precedent: the platform host is not the game itself. The Rust implementation should preserve that boundary while making it stricter at the crate/API level.

### Timing and deterministic simulation

The documented default game interval is 28 ms. The host invokes one logical game step through `TJogo.PassoSDL`, then draws and presents.

The `--gravar` mode is especially valuable as a testing idea:

- it runs the logic without waiting for wall-clock time;
- it uses a fixed random seed and scripted input;
- it records periodic BMP frames and a textual logic trace;
- repeated runs can compare logic traces independently of rendering.

This is close to the replay-and-golden-test approach planned for Rust. The Rust version should make the simulation deterministic by construction, rather than relying on a special mode bolted onto the interactive executable.

### Data-driven fighter behavior

The Pascal project moves much of the fighter configuration out of hard-coded code:

- `Dados/Lutadores/*.ini` stores character properties, move lists, command sequences, buttons, centers, and collision boxes.
- `Comum.rot` and per-character `.rot` files describe actions, animation-frame updates, AI choices, and hit reactions.
- `ChavesRoteiro.pas` interprets the simplified Pascal-like script format.
- `ChavesConferencia.pas` supports comparing legacy code with the new script behavior.

This is a strong precedent for data-driven fighters, but Rust should not automatically reproduce the Pascal scripting language. A typed schema with explicit versioning, validation, and deterministic semantics is likely easier to maintain. A small embedded scripting language should be considered only if real content needs justify it.

### Rendering, audio, and input

The project uses its own software graphics layer and BMP reader, with optional 3× PNG assets in an `HD/` tree. Text uses pre-rendered glyph atlases. Audio uses a custom mixer over SDL's audio callback rather than SDL_mixer. Input uses SDL scancodes and supports gamepads and generic joysticks.

Notable design trade-offs:

- **Software rendering** offers predictable pixel output and a simple logical framebuffer, but may constrain future effects or scaling choices.
- **Pre-rendered text atlases** avoid runtime font dependencies, but introduce an asset-generation pipeline and font licensing concerns.
- **Custom audio mixing** reduces add-on dependencies, but means the project owns mixing, voice lifetime, and synchronization behavior.
- **SDL scancodes** avoid dependence on keyboard layout and are a good default for physical arcade controls.

### Portability status

The README describes the game as SDL2-based and free of LCL in the main executable, but its stated build verification is Windows 64-bit. Linux and macOS builds, as well as Android, are still listed as work to do. Therefore, SDL abstraction should not be confused with proven cross-platform support.

---

## 2. open-street-chaves: C17 reconstruction on SDL3

Repository: [danimesq/open-street-chaves](https://github.com/danimesq/open-street-chaves)

### Runtime and platform strategy

This is an independent C17 + SDL3 reimplementation. Its README describes a cross-platform target, including Windows, macOS, Linux, and browser builds through Emscripten.

The project expects the original game data to be supplied separately at runtime. It explicitly distinguishes its source code and documentation from the original game's sprites, sounds, music, stage art, and text. That separation is important for the Rust project too.

SDL3 is the platform layer, not the source of game rules. The reverse-engineered specification in `docs/spec/` is treated as the contract for the implementation.

### Reverse-engineering specification

The spec is unusually useful because it records both the recovered values and how they were obtained:

- `[static]`: observed in binary bytes or Delphi metadata;
- `[decomp]`: read from Ghidra pseudocode, which is an analysis artifact, not original source;
- `[infer]`: interpretation not yet confirmed;
- `UNKNOWN`: not established.

This vocabulary should be adopted in our research notes. It prevents an inferred field name or behavior from quietly becoming a supposedly certain rule.

The spec covers:

- executable identity and original DelphiX components;
- screen flow, keys, configuration, and saves;
- player records and global state;
- asset structures;
- move commands and command parsing;
- fight timing, state machine, physics, hit/hurt boxes, and damage;
- effects and screen layouts;
- open questions and verification steps.

### Fight engine and recovered behavior

The fight-engine specification describes a 28 ms default timer interval, approximately 35.7 logical ticks per second. It also records a separate game-clock rule: one displayed game second advances every 25 ticks. These are distinct clocks and must not be collapsed into a conventional 60 FPS assumption.

The player record is reconstructed field by field, including positions, health, meter, stocks, state, frame, facing, hit/hurt boxes, animation timers, velocity, input sequence state, and AI-related fields.

The fight state machine includes idle, walking, standing and crouching attacks, jumping and airborne attacks, hit reactions, knockdown, lying down, blocking, taunt/special stance, throws, and character-specific specials. The spec also records move-start rules, meter gain, hit-stop, collision behavior, camera movement, and the order of operations within a fight tick.

These details are valuable, but the Rust port must still verify them against the original executable or reproducible fixtures. A decompiler's pseudocode is evidence, not ground truth.

### Deterministic regression testing

The C project documents headless, fixed-frame scenarios with a fixed RNG seed and screenshot SHA-256 comparisons. Its golden suite covers fights, effects, screens, endings, menus, and Spanish text. The original game data is required, so the README notes that CI cannot run the suite without those files.

This is a particularly strong testing pattern for Rust:

1. run the simulation without a window;
2. feed a known input sequence;
3. use a fixed RNG seed;
4. serialize or hash the logical state;
5. optionally render a frame and compare its image hash.

Logical-state tests and rendered-image tests should remain separate. A renderer change should not invalidate a gameplay test.

### A documentation caveat

The SDL3 README's status section describes a broad arcade flow and fight-engine feature set as complete, but its following "Not there yet" list also names screens, stage animations, AI choices, and other items. The status wording is therefore not fully self-consistent at a glance. Before treating any feature as implemented, inspect the corresponding source and tests. The spec itself also contains unresolved or inferred items.

---

## 3. Direct comparison

| Area | OpenStreetChaves (Pascal + SDL2) | open-street-chaves (C17 + SDL3) | Rust direction |
|---|---|---|---|
| Main role | Port/reconstruction with a strong migration story | Independent reverse-engineered reimplementation | Independent implementation informed by both |
| Game/platform boundary | SDL host calls into `TJogo` | SDL3 runtime around a C game engine | Core crate must not depend on SDL |
| Logical resolution | 400×300, software buffer, documented 3× output | Original executable spec identifies 400×300 display | Preserve logical coordinates; scale only in renderer |
| Simulation timing | 28 ms default step | 28 ms default tick recovered from original | Fixed logical tick; wall clock only schedules ticks |
| Fighter data | INI + `.rot` scripts | Generated/recovered tables and C logic | Typed, validated data first; scripts only if justified |
| State organization | One include file per game screen/state | Explicit engine and recovered state tables | Explicit enums/state transitions, separated by domain |
| Rendering | Software framebuffer, SDL texture presentation | SDL3 renderer; headless screenshot tests | Renderer backend isolated behind an interface |
| Text | Glyph atlases | Runtime font fallback strategy | Font/rendering service isolated from simulation |
| Audio | Custom mixer over SDL audio callback | SDL3 audio/runtime approach | Audio events emitted by core; backend consumes them |
| Input | SDL scancodes, gamepads, joysticks | Keyboard/gamepad support documented | Normalize device input into game actions |
| Determinism | `--gravar` fixed seed + logic log | Fixed-seed headless scenarios + golden hashes | First-class replay format and deterministic tests |
| Reverse-engineering evidence | Legacy-vs-script comparison mechanism | Detailed static/decomp/infer/unknown specification | Evidence ledger with source, confidence, and test |
| Portability | SDL2 host is separated, but README reports Windows-only build verification | SDL3, desktop and Emscripten targets documented | Start desktop SDL3; keep core platform-neutral |

---

## 4. What to adopt, adapt, and avoid

### Adopt

- The Pascal project's host/game separation.
- The Pascal project's data-driven fighter configuration and legacy-vs-new comparison mindset.
- The C project's explicit reverse-engineering evidence labels.
- The C project's detailed player/state/move/effect specification.
- Both projects' deterministic, headless testing ideas.
- A 400×300 logical coordinate system as the initial compatibility target, subject to validation against the original.

### Adapt

- Replace Pascal `.rot` scripts with typed Rust data and explicit behavior tables initially.
- Replace generated C tables with Rust-generated or checked-in typed tables, keeping the generation source and provenance visible.
- Preserve the original's tick semantics, including the 25-tick displayed-second counter, rather than treating 28 ms as a conventional frame rate.
- Treat software rendering as a compatibility/reference backend or initial renderer, not as a permanent architectural constraint.
- Model input as logical actions, with SDL scancodes/gamepad events translated at the edge of the system.

### Avoid

- Translating either codebase line-for-line.
- Mixing SDL types into fight-engine structures.
- Assuming all README status claims are verified by tests.
- Treating inferred Ghidra field names as original source identifiers.
- Making a 60 FPS simulation and trying to retrofit the original's 28 ms behavior.
- Copying assets, decompiled code, or third-party source before checking provenance and applicable licenses.

---

## 5. Role of the SMS project

Repository: [danimesq/street-chaves-hack-sms](https://github.com/danimesq/street-chaves-hack-sms)

Use it as a supporting research tool, not as a gameplay authority.

Its documented strengths include:

- tracing a ROM through boot and into a live match;
- capturing VRAM and other graphics state;
- decoding tile, frame, palette, and sprite structures;
- building repeatable asset extraction and conversion tools;
- recording hardware limits such as sprite-per-scanline constraints;
- comparing candidate animation poses using image processing.

These techniques may inspire our own asset-inspection and evidence tooling. However, its base is a Master System hack and its game behavior is constrained by an 8-bit console engine. Those constraints and adaptations must not be projected onto the original Windows game.

---

## 6. Initial recommendation

Build a Rust game core around a deterministic fixed-tick simulation. Let SDL3 handle windowing, presentation, audio output, and physical input, but keep those dependencies outside the core.

The first milestone should not be a playable clone. It should be a small, testable compatibility kernel:

1. define the 400×300 logical world and tick scheduler;
2. define player state and input-action types;
3. implement a deterministic replay harness;
4. encode a small set of verified state transitions;
5. compare traces against the best available original-game evidence;
6. add rendering only after logical behavior can be tested headlessly.

This makes the first Rust code a measurement instrument as much as a game engine.
