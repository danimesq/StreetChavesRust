# Data formats and resource pipelines

## Scope and evidence labels

This document records formats and pipelines observed in the pinned source revisions. Pascal/OpenStreetChaves is the normative project source for Rust reconstruction; C is comparison-only. The ROT normalizer is a static research parser, not yet a runtime game-script engine or a proposed final data schema.

Evidence labels:
- **Observed in source/data**: directly described by a checked-in loader, data file, generator or project documentation.
- **Observed in reverse-engineering specification**: described in the C project's analysis of the original executable. Some claims derive from static binary inspection or Ghidra pseudocode, not original source code.
- **Needs validation**: incomplete, ambiguous, inferred, or not yet tested against the original executable.
- **Not reusable by default**: the data may be copyrighted or have unresolved provenance even if the file is present in a public repository.

## Pascal / SDL2 data model

Primary references:
- `OpenStreetChaves/source/game/ChavesDados.pas`
- `OpenStreetChaves/source/game/ChavesRoteiro.pas`
- `OpenStreetChaves/source/LEIAME.md`
- `OpenStreetChaves/Dados/Recursos.ini`
- `OpenStreetChaves/Dados/Jogo.ini`
- `OpenStreetChaves/Dados/Lutadores/Comum.rot`

### INI conventions

The loader uses Free Pascal's `TMemIniFile` (FCL, no LCL dependency in the game runtime). The source comments describe:
- Lines beginning with `; ` are comments. Inline comments after a value are not supported by the documented convention.
- Decimal values use a dot independently of the OS locale.
- Lists use comma separators.
- Missing keys are read with caller-provided defaults.
- Resource paths are relative to the game root and resolved by the loader.
- Numbered filenames use one or more `#` characters as a zero-padded numeric placeholder.

The source implementation also includes helpers for decimal parsing/writing, splitting comma lists, resolving relative paths, and generating numbered filenames. Exact behavior for malformed quoting, duplicate keys, repeated sections, whitespace and invalid numeric values still needs dedicated tests before a Rust parser is considered compatible.

### Global game configuration: `Dados/Jogo.ini`

Observed sections:
- `[Jogo]`: version string, default tick interval, gravity, special-meter gain and flying stage-object probability/range.
- `[Padroes]`: initial difficulty, round timer, power mode, damage level, blood/flash/display toggles, number of rounds and language.
- `[Liberacao]`: initial unlocked roster and progression order.
- `[Recordes]`: initial high-score rows.

The checked-in example uses `IntervaloTimer=28` and `Gravidade=4`. These are defaults in this implementation, not proof that every original-game build uses the same values.

### Resource registry: `Dados/Recursos.ini`

The file documents two resource styles:
1. Named sections such as `[Icones]`, `[Abertura]` and `[Final]`, with `Name=relative/path,transparent` entries. The documented transparency flag is 1 for white-to-transparent and 0 for opaque. The order of entries assigns image indices, while code can retrieve resources by semantic name.
2. Numbered series with a path pattern containing `#` and a separate quantity, e.g. `Animacoes/Anim-##.bmp` and `Efeitos/Efeito-###.bmp`.

The same registry points to external data INIs for effects, endings, fighters and stages. A Rust resource manifest should preserve semantic names and explicit order where order is observable; it should not reduce everything to arbitrary filesystem enumeration.

### Fighter INI: `Dados/Lutadores/CNN.ini`

The project documentation identifies these sections:
- `[Geral]`: sprite height, movement speed, boss/opponent exclusions, stage and music selection rules.
- `[Golpes]`: special-move list shown for the fighter.
- `[Sequencias]`: directional input transitions for special-move recognition.
- `[Botoes]`: button plus sequence state to move/command mapping.
- `[CentroX]`: per-frame horizontal center/anchor information.
- `[Corpo]`: per-frame body/hurt boxes.
- `[Ataque]`: per-frame attack boxes.

The parser/consumer relationship is documented in `ChavesGolpes` and the fight-state code. The exact key grammar, missing-frame fallback, box coordinate conventions and validation behavior must be captured from the loader and representative files before writing a Rust schema.

The current tree has `C01.ini` through `C16.ini`. Do not silently normalize this to 15 fighters: determine what C16 represents and how it is selected/used.

### Fighter scripts: `Dados/Lutadores/Comum.rot` and `CNN.rot`

These are not passive data tables. They contain a simplified Pascal-like language interpreted by `ChavesRoteiro.pas`.

Documented sections:
- `[Acao]`: action-start handling, with common behavior executed before the fighter-specific section.
- `[AcaoFim]`: action completion.
- `[Quadro]`: frame update, sprite selection, animation progression, effect/sound creation and move motion.
- `[IA]`: CPU decisions; common rules followed by fighter-specific special-move choices.
- `[Acerto]`: changes when an attack hits, including combo, rotation, recoil, launch and chained attacks.

The common script documents symbols for player/opponent records, game constants, pending sound, camera/floor/meter values, random/math helpers, effect creation, push and action functions. It also documents local variable declarations and comments. Treat this as a small embedded language with a parser/interpreter, not as a configuration format.

Important parser compatibility questions:
- Is evaluation order strictly left-to-right for boolean expressions?
- What are the exact operator precedence and integer/float conversion rules?
- Are script variables scoped per section, per fighter or per interpreter instance?
- What is the exact semantics of `Random`, `Round`, `Trunc`, `ShortInt` and set membership?
- How are runtime errors surfaced, and can script execution partially mutate state before failure?
- How are common and fighter-specific sections concatenated, and what happens when one is absent?

No Rust script interpreter should be started until these semantics are inventoried and tests exist.

### Stage INI: `Dados/Cenarios/Cenario-NN.ini`

The project documentation identifies:
- `[Geral]`: stage music.
- `[AnimacaoN]`: background animation frames, character-based visibility conditions and cycle ranges.
- `[Item]`: breakable stage item.
- `[Voador]`: flying object configuration.

Camera bounds are tied to stage image dimensions in the documented implementation. Exact field grammar and update order remain a parser/specification task.

### Effects INI: `Dados/Efeitos.ini`

The Pascal documentation describes 36 effect types. Each type can define creation-time properties such as lifetime, damage, hit box, speed and randomization, plus per-frame animation rules (named `AnimN` in the documentation). The current repository tree contains 111 effect BMP files, which is a resource count, not the number of logical effect types.

Do not equate image index, effect type ID and effect animation phase. Their relationship must be extracted from the INI and consumer code.

### Endings, sound and localization

- `Dados/Finais.ini`: per-character ending script with time/condition-driven commands for loading stage/fighters, drawing, dialogue, black screens, explosions and effects.
- `Dados/Som.ini`: sound volume percentages and lookup/fallback rules.
- `Dados/Textos/PT.ini` and `ES.ini`: localized strings, dialogue, endings and interface text. The source docs describe numbered lists/sections and special handling for leading/trailing spaces.
- `Chaves.ini`: runtime user configuration/progression file, created by the game and not included in the pinned tree.

These formats should remain separate domain models in Rust even if a common low-level INI reader is used.

### Image and audio resources

Observed resource types include BMP, PNG and WAV:
- BMP resources are the primary image assets referenced by the registry.
- HD PNGs are optional 3× counterparts under `HD/`, preserving dimensions/layout relationships required by the original BMP-based frame cuts.
- WAV resources are used for sound effects, voice and music.
- The source README describes a font atlas format (`.atlas`) generated by a separate tool.

The high-resolution generation tool is not the same as the game data loader. Generated HD images should be tracked as derived artifacts, with the original source file and transformation settings recorded.

## C17 / SDL3 data model (comparative only)

Primary references:
- `open-street-chaves/docs/spec/`
- `open-street-chaves/tools/gen_tables.py`
- `open-street-chaves/src/gen/tables_gen.h`
- `open-street-chaves/src/data/`

### Original-game external files

The C project README explicitly says no original sprites, sounds, music, stage art or in-game text are included. The user supplies the original Street Chaves 1.5A data folder and executable. The runtime data loader is implemented in `src/data/`, including modules for BMP, strips, WAV, executable images and executable text.

The original executable is therefore both a source of embedded resources/text and a reverse-engineering target. The project documentation distinguishes static executable observations, Ghidra decompilation observations, inference and unknowns.

### Specification-to-generated-C pipeline

The C project stores recovered tables in `docs/spec/` and generates `src/gen/tables_gen.c/.h` with `tools/gen_tables.py`. The generated header says not to edit generated files by hand.

Examples of declared generated structures:
- Animation selector rows (`osc_anim_row`).
- Selector field-store rows (`osc_sel_store_row`).
- Hurt-box and hit-box rows (`osc_hurt_row`, `osc_hit_row`).
- Move setup, sequence and button mapping rows (`osc_move_row`, `osc_seq_row`, `osc_golpe_row`).
- CPU decision rows (`osc_cpu_row`).
- Ending command rows (`osc_ending_row`).
- Effect type, phase-to-sprite, spawn and sound rows.
- Stage animation and text-index rows.

At the pinned revision, the generated header declares these dimensions:
- `osc_anim[413]`
- `osc_motion[95]`
- `osc_text_index[1966]`
- `osc_stage_anim[137]`
- `osc_fx_types[37]`
- `osc_fx_sprites[125]`
- `osc_fx_spawns[106]`
- `osc_fx_sounds[57]`

These are compile-time table dimensions. They are not equivalent to unique moves, fighters, frames, or effect instances.

### Configuration and save data

The C project has `src/core/config.c/.h` and `src/core/unlocks.c/.h`. The README says configuration is saved in the SDL preferences directory rather than the supplied game-data folder. The exact serialized format and migration behavior must be inspected separately before a Rust compatibility decision.

### Asset decoding and strip slicing

The reverse-engineering specification describes the original PC data folder and the C loader's use of fighter strips, portraits, animations, effects, stage images and WAV audio. Fighter strips are horizontal BMP strips split into frames using dark separator columns, with mirrored frames also produced. Treat the slicing convention, transparency color, image dimensions and frame ordering as part of the data contract.

The exact original asset counts in `docs/spec/03-assets.md` differ from the current OpenStreetChaves checked-in tree counts in some categories. These inventories refer to different snapshots/distributions and must not be silently reconciled. The C spec's counts are the reverse-engineered original data-folder inventory; the Pascal counts above are the current Git tree at its pinned revision.

## SMS project data model (supporting only)

Primary references:
- `street-chaves-hack-sms/README.md`
- `PACKAGE.md`
- `docs/ROM-REFERENCE.md`
- `tools/`, `emu/`, `work/`

The project manipulates ROM images and patches, extracts graphics and poses, and rebuilds a Master System ROM. Its useful technical formats include:
- SMS 4bpp planar 8×8 tiles, 32 bytes per tile.
- 16-bit tilemap/name-table entries.
- Composite frame data with tile count and tile payload, described in the ROM reference.
- BPS patch files.
- Python-generated/intermediate pose images and serialized working assignments.

These are hardware/ROM formats, not the PC game's BMP/INI/ROT data formats. They belong in tooling research only unless a later, explicit art-conversion task calls for them.

## Rust implementation implications

1. Implement a low-level INI reader only after edge cases are tested against the Pascal loader.
2. Keep semantic resource names and explicit ordering. Do not rely on directory iteration order.
3. Model fighter numeric tables separately from executable script behavior.
4. Treat `.rot` as an interpreter language. Do not translate it to ad-hoc JSON or hard-code its behavior without a complete semantic inventory.
5. Track generated C tables as evidence-derived data, not hand-authored Rust constants. Keep source specification and generation provenance.
6. Keep original executable resources, external user-supplied assets, derived HD art and SMS/SNES working assets as distinct source classes.
7. Preserve original-vs-port confidence labels on every behavior or field whose semantics are not validated.
8. Do not import copyrighted assets or binary blobs into StreetChavesRust merely because a public repository contains them.

## Open format questions

- Full grammar and evaluation semantics of `.rot`.
- Exact per-section key schema, duplicate-key policy and defaults for fighter/stage/effect INIs.
- C configuration serialization and save compatibility.
- Exact BMP strip separator algorithm and behavior on malformed/edge-case strips.
- Font atlas binary/text format and generation metadata.
- Whether the two SDL projects use identical original data archives/executable hashes.
- Per-resource provenance and license/permission status.
- Whether any data tables have been manually corrected beyond direct reverse-engineering observations.
