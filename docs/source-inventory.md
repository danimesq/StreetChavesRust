# Source and data inventory

## Purpose and scope

This document is the Phase 1 inventory baseline for StreetChavesRust. It records the repository revisions inspected, the major source/data areas, and the current understanding of how each reference project is structured.

The three repositories are independent projects. The two SDL repositories are the primary references for the PC game. The SMS project is a supporting reverse-engineering and graphics-pipeline reference only; it is not evidence for PC combat rules.

This is a repository-tree inventory, not a claim that every file has been semantically inspected. Counts below are the number of Git tree entries returned by GitHub's recursive tree endpoint at the pinned commit. They include directories as entries. They are useful for scope, not as a measure of implementation completeness.

## Pinned reference revisions

| Repository | Revision inspected | Tree entries | Declared license |
|---|---|---:|---|
| [OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves) | `e1b025c35ecbc2b14d8635564aafd23c2107d3d1` | 967 | GPL-3.0 |
| [open-street-chaves](https://github.com/danimesq/open-street-chaves) | `e76ad2188af6842ff6618ecdb8f425bba61c9183` | 136 | GPL-3.0 |
| [street-chaves-hack-sms](https://github.com/danimesq/street-chaves-hack-sms) | `6c439d880efa89ab4fccd4accaa70af743c8cda1` | 4,262 | No root license file found |

The revisions are snapshots, not permanent upstream pins. Refresh this table only when a new inspection is performed, and record the new SHA and the reason for changing it.

## OpenStreetChaves: Pascal / Lazarus / SDL2

Repository: [danimesq/OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves)

The repository describes a Lazarus/Free Pascal reconstruction of the original Delphi 5 + DelphiX game, migrated to SDL2. Its README states that the runtime uses SDL2 for window/video/audio/input and that game coordinates are logical 400×300, with a software buffer scaled for presentation.

### Runtime and code ownership

| Area | Important paths | Responsibility observed in source/docs |
|---|---|---|
| SDL host | `source/ChavesSDL.pas` | Window, SDL event loop, keyboard/controller input, focus handling, fixed-interval game stepping and presentation |
| Game object | `source/ChavesJogo.pas` | `TJogo`, main game state and included state implementations |
| Shared game types/data | `source/game/ChavesTipos.pas`, `ChavesConst.pas`, `ChavesGolpes.pas`, `ChavesDados.pas` | Types, constants, move data and INI/resource loading |
| Fighter scripts | `source/game/ChavesRoteiro.pas`, `Dados/Lutadores/*.rot` | Simplified Pascal-like script interpreter and per-character/common behavior |
| State implementation | `source/game/Estado*.inc`, `Comum_*.inc` | Menu, selection, fight, endings, shared rendering/resources/input and other game states |
| Graphics | `source/game/ChavesGfx.pas` | Software surfaces, image lists, BMP loading and drawing helpers |
| Audio | `source/game/ChavesSom.pas` | Sound bank/mixer on SDL audio |
| SDL bindings | `source/3party/SDL2/` | Pascal SDL2 bindings; the project notes that only the core binding is used by the game |
| Auxiliary tools | `source/tools/GeraFonte/`, `source/tools/GeraHD/`, `source/tools/Documentacao/` | Font atlas generation, high-resolution PNG generation and source documentation |

The game logic is split between Pascal units and `.inc` fragments included into the game object. A Rust port must follow actual ownership and call flow rather than treating each `.inc` file as a standalone module.

### Data and resource roots

| Path | Current tree observation | Role |
|---|---|---|
| `Dados/Lutadores/` | 16 `.ini` files named C01..C16 and 17 `.rot` files including `Comum.rot` | Fighter parameters, collision boxes, command mapping and scripts |
| `Dados/Cenarios/` | 13 `.ini` files | Stage animation and stage-specific object configuration |
| `Dados/Efeitos.ini` | One INI | Effect type definitions and animation rules |
| `Dados/Jogo.ini` | One INI | Game defaults and initial progression/record data |
| `Dados/Recursos.ini` | One INI | Named resource registry and numbered resource series |
| `Dados/Textos/` | PT and ES INI files | Localized strings, dialogue and ending text |
| `Animacoes/` | 97 BMP files | Stage/scene animation images |
| `Cenarios/` | 13 BMP files | Stage backgrounds |
| `Efeitos/` | 111 BMP files | Effect/projectile/spark images |
| `Lutadores/` | 16 files in the repository tree | Fighter sprite strips and related fighter resources; inspect individual names before assuming a one-to-one count |
| `Rostos/` | 48 files | Fighter portraits/faces |
| `Sons/` | 71 files | Sound effects |
| `Falas/` | 32 files | Voice lines |
| `Musicas/` | 6 files | Music |
| `Embutidos/` | 42 files | Images extracted/used for intro, endings and interface |
| `HD/` | 326 PNG files | High-resolution image counterparts generated from BMP assets |

The tree contains 967 entries total. The source tree has 111 files and 9 directory entries. The root also contains a large `Chaves.idc` file; it is not part of the runtime source path described by the README and should be classified separately before any toolchain decision.

### Entry points and data flow

- The Lazarus project is `source/StreetChaves.lpi` with its program source `source/StreetChaves.lpr`.
- `ChavesSDL.pas` is the SDL host. Its documentation describes a 28 ms default interval and a call into `TJogo.PassoSDL`.
- `ChavesJogo.pas` assembles the game object from `source/game/` includes.
- `ChavesDados.pas` loads INI data from `Dados/` and resolves relative resource paths.
- `Dados/Recursos.ini` maps semantic resource names to files. Numbered series use `#` placeholders.
- `Dados/Lutadores/CNN.ini` and `CNN.rot` are consumed through the data loader and script interpreter. `Comum.rot` supplies shared script sections.
- The README describes `HD/` as an optional 3× PNG counterpart for BMP assets, produced by the separate GeraHD tool. The BMP remains the layout/reference source for fighter frame slicing.

### Important caveats

- The repository's runtime data count is not equivalent to the original game's roster count. The INI sequence currently contains C01..C16, while other source/spec material may refer to 15 original fighters. The extra slot must be explained from source and runtime behavior before hard-coding a roster size.
- The root `Lutadores/` directory contains 16 files, but this tree-level count does not establish whether every file is a fighter strip, a duplicate, or another auxiliary resource.
- Some source files are generated/includes or tool inputs. Their roles should be tracked separately from hand-maintained runtime code.
- The repository README mentions a `Chaves.ini` file created at runtime; it is not part of the checked-in data inventory at this revision.

## open-street-chaves: C17 / SDL3

Repository: [danimesq/open-street-chaves](https://github.com/danimesq/open-street-chaves)

The README describes an independent C17 + SDL3 reimplementation of Street Chaves 1.5A. It states that game behavior was recovered from the original `Chaves.exe`, and that the repository contains no original game assets. Users provide their own game data folder at runtime.

### Runtime and code ownership

| Area | Important paths | Responsibility observed in tree/source |
|---|---|---|
| Application | `src/main.c`, `src/game.c`, `src/game.h` | Process entry, game lifecycle and shared game state |
| Fight engine | `src/fight/fight.c`, `fight.h`, `fight_internal.h` | Explicit fight tick and internal fight interfaces |
| Fighter state | `src/fight/fighter.c` | Fighter initialization, movement, counters, frame progression and physics |
| Move selection | `src/fight/selector.c`, `commands.c`, `cpu.c` | Frame/move selection, input sequences and CPU decisions |
| Combat/effects/stage | `src/fight/combat.c`, `effects.c`, `stage.c` | Defense, melee resolution, effect lifecycle and stage objects |
| Rendering | `src/render/` | Fight drawing, canvas, HUD, fonts and text |
| Screens/flow | `src/screens/` | Arcade flow, menus, selection, VS, fight, continue, records and endings |
| External data loading | `src/data/` | BMP/strip/WAV/executable-image/text loading |
| Generated numeric tables | `src/gen/tables_gen.c`, `tables_gen.h` | C tables generated from documentation/specification data |
| Table generator | `tools/gen_tables.py` | Converts specification inputs into generated C tables |
| Reverse-engineering tools | `tools/re/` | PE/DFM parsing, disassembly/decompilation support, extraction and table-building scripts |
| Tests | `tests/golden.txt` and test scripts | Deterministic/golden scenario evidence; inspect scripts before relying on their exact invocation |

The fight API exposes `osc_fight_tick(g, in1, in2)`, which is a useful seam for headless simulation testing. The rendering code is separated into `src/render/`, although this inventory does not imply every renderer call is side-effect free; that must be verified per function.

### Specification and generated-data pipeline

The `docs/spec/` directory is a first-class source for the project:

- `00-code-map.md`: original executable identity, architecture and recovered code map.
- `01-flow-and-input.md`: original game flow, input and configuration.
- `02-player-record.md`: player record and global fields.
- `03-assets.md`: original data folder, asset loading and frame-strip slicing.
- `04-moves.md` and `04a-move-tables.txt`: move list and command mapping.
- `05-fight-engine.md`, `05a-anim-tables.txt`, `05b-move-setup.txt`, `05c-frame-offsets.txt`, `05d-cpu-table.txt`: fight rules and numeric tables.
- `06-effects.md` and `06a`–`06g`: effect definitions, special sounds/motion/spawns, frame selection and selector stores.
- `07-screens-and-text.md`, `07b-layout.md`, `07c-stage-animations.txt`, `07d-text-index.txt`, `07e-endings.txt`: screens, layouts, stage animations, text indices and endings.
- `99-open-questions.md`: remaining unknowns and validation status.

The generator header explicitly says that `tables_gen.c` and `tables_gen.h` are generated from `docs/spec` and should not be edited manually. The generated header currently declares, among other tables, 413 animation rows, 95 special-motion rows, 1,966 text-index rows, 137 stage-animation rows, 37 effect types, 125 effect-sprite rows, 106 effect-spawn rows and 57 effect-sound rows. These are table dimensions in this pinned revision, not counts of unique gameplay actions.

### External data contract

The README says the repository does not include sprites, sounds, music, stage art or in-game text. It expects the user to provide the original Street Chaves 1.5A data folder, including `Chaves.exe` and directories such as `Animacoes`, `Cenarios`, `Efeitos`, `Falas`, `Lutadores`, `Musicas`, `Rostos` and `Sons`.

This is materially different from OpenStreetChaves, which checks a large set of data and derived images into its repository. It also means that the C project’s generated tables/specs and its runtime asset loader must be inventoried separately.

## street-chaves-hack-sms: supporting reference

Repository: [danimesq/street-chaves-hack-sms](https://github.com/danimesq/street-chaves-hack-sms)

This is a separate Master System ROM-hack project using a Street Fighter II SMS base and character art sourced from Gerson Almeida's Street Chaves 2 SNES hack. It is not a port of the PC game's original mechanics.

### Repository shape

The pinned tree has 4,262 entries. The large count is heavily affected by:
- `work/poses/`: captured pose images and related working data;
- `pylibs/`: vendored Python/Z80 dependencies;
- `tools/`: ROM, graphics, pose-matching and build tools;
- `emu/`: a headless SNES tracing/emulation helper;
- `release/`: BPS patches and a title-screen image.

The repository README and PACKAGE document state that ROM images are deliberately excluded from the source package because they are copyrighted game data. The toolchain expects legally obtained inputs and creates a patched SMS ROM.

### Useful areas for this project

- `docs/ROM-REFERENCE.md`: evidence-led technical notes on SMS/SNES ROM layouts, graphics encodings, palette constraints, asset addresses and tracing.
- `tools/sms.py`, `smsgfx.py`, `snesgfx.py`, `snes_capture.py`, `fighter_frames.py`: supporting examples of hardware graphics inspection and extraction workflows.
- `work/poses/`: captured SNES pose references. These are not automatically reusable assets.
- `release/*.bps`: patches, not a license to redistribute the source ROMs or underlying artwork.

Do not import SMS character timing, collision, movement, state IDs or combat rules into the PC implementation. The SMS project can inform reverse-engineering methodology and graphics tooling only.

## Cross-project subsystem map

| Concern | Pascal / SDL2 | C17 / SDL3 | SMS support project |
|---|---|---|---|
| Main loop | `ChavesSDL.pas` + `TJogo` | `main.c`, `game.c`, `osc_fight_tick` | Emulator/probe scripts |
| Fighter data | `Dados/Lutadores/*.ini`, `*.rot` | `docs/spec/04*`, `05*`, generated tables | Pose mapping/art pipeline only |
| Animation/frame selection | `ChavesRoteiro.pas`, `Comum_Quadro.inc`, fighter scripts | `selector.c`, `fighter.c`, generated tables | Sprite frame extraction, not PC animation semantics |
| Combat/collision | `Estado7_Luta.inc`, `Estado7_Golpes.inc`, `ChavesGolpes.pas` | `combat.c`, `fighter.c` | Not authoritative |
| Effects/projectiles | `Dados/Efeitos.ini`, `desenhaefeitos` path | `effects.c`, effect tables | Graphics/ROM tracing only |
| Stage data | `Dados/Cenarios/*.ini`, `Cenarios/*.bmp` | `stage.c`, stage-animation spec/tables | Not authoritative |
| Text/audio/assets | checked-in resources + `Recursos.ini` | user-supplied original data + loaders | SNES/SMS assets and derived art |
| Build/runtime | Lazarus/FPC + SDL2 | CMake/C17 + SDL3 | Python tools, ROM patches, emulator helper |

## Phase 1 inventory status

- [x] Pin the inspected revisions.
- [x] Map top-level repository structure and major source directories.
- [x] Identify the main runtime and fight-engine entry points.
- [x] Identify the main data roots and generated-table pipeline.
- [x] Separate PC-game references from SMS-only supporting material.
- [ ] Inspect every fighter/stage/effect data schema and parser edge case.
- [ ] Classify every checked-in asset by provenance and redistribution status.
- [ ] Verify the exact original executable/data revision used by each reverse-engineering result.
- [ ] Record the full dependency and subcomponent license inventory.

The unchecked items are deliberate follow-up work. This document does not claim that Phase 1 is fully complete.
