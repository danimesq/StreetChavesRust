# Custom character package format

## Status and scope

This is a proposed, documentation-only contract for custom character packages. No package loader, JSON validator, installer, asset API, or custom-character runtime is implemented by this document.

The package layout follows the existing Street Chaves data-root conventions. It does **not** introduce parallel `characters/<id>/sprites/` or `audio/` trees. Native packages mirror the folders already used by the game; WASM/Canvas and Construct hosts may provide the same logical resources through a host API instead of direct filesystem access.

## Compatibility principle

A character package is a partial overlay rooted at the Street Chaves data root:

- Character definition metadata lives under `Dados/Lutadores/`.
- Fighter sprite strips live under `Lutadores/`.
- Portraits live under `Rostos/`.
- Victory voice lines live under `Falas/`.
- Shared sound effects live under `Sons/`.
- Optional high-resolution counterparts mirror the original path below `HD/`.

These names and their resource roles are observed in the pinned Pascal reference's `Dados/Recursos.ini`. The Chapolin extension is a concrete example: `C16.ini` and `C16.rot` are under `Dados/Lutadores/`, while its fighter strip is `Lutadores/C16.bmp`.

## Package layout

The following is an illustrative package root. `17` is an example slot only; it is not a claim that slot 17 has been reserved or that the current game can load it.

```text
<package-root>/
├── Dados/
│   └── Lutadores/
│       └── C17.json             # proposed custom-character manifest
├── Lutadores/
│   └── C17.bmp                  # fighter sprite strip
├── Rostos/
│   ├── R17-a.bmp                # portrait source variant A
│   ├── R17-b.bmp                # portrait source variant B
│   └── R17-c.bmp                # portrait source variant C
├── Falas/
│   ├── Fala-17-a.wav            # victory voice variant A
│   └── Fala-17-b.wav            # victory voice variant B
├── Sons/
│   └── Som-##.wav               # allocated entries in the shared SFX bank
└── HD/
    ├── Lutadores/
    │   └── C17.png              # optional high-resolution counterpart
    └── Rostos/
        ├── R17-a.png
        ├── R17-b.png
        └── R17-c.png
```

The package root is not a new directory category inside the game. It mirrors the game's existing root so that its contents can be merged or mounted as a data-root overlay. A package may omit optional resource groups that its character does not use, subject to the selected game mode's resource requirements.

### File and slot naming

The numeric part of `C##`, `R##`, and `Fala-##` is a legacy roster slot, not the character's permanent identity.

- The JSON manifest has a stable string `id`, such as `chapolin`.
- A native/on-disk installation assigns a non-conflicting numeric slot and uses it consistently in the corresponding fighter, portrait, and voice filenames.
- A package must not assume that C16 is available: the inspected Pascal reference already uses C16 for Chapolin.
- Slot allocation and roster registration belong to the future package host/loader. The folder layout alone does not extend the current roster or prove that fixed-size tables and game-state limits can accept another fighter.

The example `C17` names above are illustrative. A real installer must inspect the target data root and allocate a free slot rather than blindly copying those names.

## Manifest location and responsibility

The proposed manifest is placed beside the existing per-fighter data files:

```text
Dados/Lutadores/C17.json
```

It is the authoritative definition for the new custom-character format. It should contain:

- Stable identity: `id`, display name, author, package version, and `schema_version`.
- Fighter attributes and movement parameters.
- Normal attacks, command sequences, combos, and special moves.
- Declarative behavior: allowed states, conditions, costs, transitions, and supported runtime operations.
- Animation/frame metadata, frame order, anchors, and hit/hurt-box data.
- Logical audio-event identifiers and references to voice and sound resources.
- Compatibility requirements/capabilities and per-asset provenance or license metadata.

The schema version is independent of the game/runtime version. Behavior must remain declarative and use a documented set of runtime operations; a package must not embed arbitrary Rust, Pascal, JavaScript, or Construct code.

The JSON manifest is a new format. The existing Pascal loader reads INI data and `.rot` scripts; the presence of a `.json` file in the same folder does not make that legacy loader understand it. Existing `C##.ini` and `C##.rot` files may be supplied as explicit legacy companion data for a target that needs them, but they are not required by the JSON package and must not silently become a second source of truth. A `.rot` file is executable script, not passive animation metadata, and must not be assumed to be losslessly generated from the declarative manifest.

## Resource mapping

All manifest asset paths are logical paths relative to the **game data root**, matching the base used by `Dados/Recursos.ini`. They are not relative to the manifest's own directory.

| Resource | Street Chaves path convention | Package meaning |
|---|---|---|
| Fighter sprite strip | `Lutadores/C##.bmp` | The character's fighter image strip; frame/animation interpretation is described by the manifest. |
| Portraits | `Rostos/R##-a.bmp`, `Rostos/R##-b.bmp`, `Rostos/R##-c.bmp` | Source portrait variants. The legacy registry derives mirrored entries from the same source files; separate mirrored copies are not required by this layout. |
| Victory voices | `Falas/Fala-##-a.wav`, `Falas/Fala-##-b.wav` | Character-indexed voice variants, separate from sound effects. |
| Sound effects | `Sons/Som-##.wav` | Entries in the game's shared, numbered sound-effect bank, not a private per-character bank. |
| Music | `Musicas/Musica-##.wav` | Shared numbered music bank. A character may refer to an existing track/slot where the target supports it; music is not moved into a character-specific audio folder. |
| Effect sprites/data | `Efeitos/Efeito-###.bmp`, `Dados/Efeitos.ini` | Shared effect resources. A character may reference supported shared effect IDs; introducing new effect types is a separate shared-data extension. |
| High-resolution images | `HD/<original-relative-path-without-extension>.png` | Optional high-resolution counterpart that preserves the original asset's layout and scale relationship. |

The existing `[Sons]` resource entry loads the numbered sound-effect sequence, and the Pascal sound bank assigns indexes in insertion order, reserving an index even when its file is absent. Character-specific sound additions therefore cannot be treated as an isolated unordered folder: a future native installer/adapter must allocate stable entries in the shared bank, preserve all existing indexes, and map manifest-level sound IDs to those entries. Append-only allocation is the safe compatibility rule; packages must not renumber or insert into an existing sequence.

The pinned Chapolin script provides a concrete example of the legacy mapping: its comments identify sound index 67 with `Som-68.wav` and index 68 with `Som-69.wav`. New package definitions should use stable logical sound IDs and let the target adapter resolve them to legacy bank indexes where required.

Do not enumerate files and infer semantic order from directory order. Preserve explicit manifest references and the order required by any legacy numbered bank.

### Path safety

Package paths must be relative, use the canonical Street Chaves folder names, and resolve inside the selected package/data root. Absolute paths and traversal outside that root (for example, paths containing `..`) are invalid. File-name casing must match the stored asset names, especially on case-sensitive filesystems.

## Native folders versus WASM/Construct resource APIs

The package definition and logical resource paths remain the same across targets. Only the resource source changes.

### Native/on-disk target

The native loader resolves each manifest path against the mounted game data root using filesystem access. This follows the existing convention in which resource paths in `Dados/Recursos.ini` are relative to the game's root.

### WASM/Canvas and Construct 2/3 targets

A browser export must not require the Rust core to open native filesystem paths. Instead, the browser/host adapter resolves each manifest resource through a versioned resource-provider API. The provider may supply encoded bytes, a supported browser resource handle, or another explicitly supported payload; the platform-neutral engine sees only stable logical resource IDs and metadata.

The API is a transport/resolution layer, not a second character format:

1. Read the same manifest and its logical Street Chaves paths.
2. Ask the host/resource provider to resolve and register each referenced resource.
3. Decode/validate the resource in the platform adapter.
4. Report loading, ready, or failure for the logical resource ID.
5. Start or resume a fight only when required resources are ready under the documented policy.

Resources should be registered once and reused by ID, not resent as full image/audio payloads every simulation tick. Missing files, invalid data, unsupported formats, duplicate IDs, and decode failures must produce explicit diagnostics; adapters must not silently substitute unrelated resources.

Construct 2 and Construct 3 have different SDK generations and require version-specific host adapters. The exact SDK mechanism for obtaining or supplying asset bytes remains an implementation-time verification item. Do not rely on undocumented private texture/runtime objects. This document does not claim that the WASM resource API or either Construct adapter already exists.

## Compatibility boundary and non-claims

Matching the folder layout makes a package compatible with the Street Chaves **data organization**; it does not by itself make it loadable by the current Pascal game or by the current StreetChavesRust runtime.

The following remain separate implementation tasks:

- Manifest discovery, schema validation, and safe path resolution.
- Numeric roster-slot allocation and support beyond the currently observed roster/Chapolin extension.
- Mapping custom behavior to operations actually supported by the generic runtime.
- Registering character-specific sounds in the shared sound bank without changing existing indexes.
- WASM resource-provider implementation and Construct 2/3 SDK-specific adapters.
- Tests proving that all target adapters preserve the same manifest semantics.

No original-game sprites, audio, scripts, or other copyrighted assets are included or licensed by this format specification. Asset provenance and redistribution permission must be reviewed independently.

## Source references

The folder and resource conventions above are grounded in these pinned references:

- [OpenStreetChaves `Dados/Recursos.ini`](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Recursos.ini)
- [OpenStreetChaves `ChavesDados.pas`](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/game/ChavesDados.pas)
- [OpenStreetChaves `ChavesSom.pas`](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/game/ChavesSom.pas)
- [OpenStreetChaves `C16.ini`](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C16.ini)
- [OpenStreetChaves `C16.rot`](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C16.rot)
- [StreetChavesRust Construct 2/3 integration proposal](construct-wasm-canvas-integration.md)
- [StreetChavesRust WASM/Canvas rendering architecture](rendering-backends-wasm-canvas.md)
