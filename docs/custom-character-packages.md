# Custom character packages

## Status

This PR adds the first executable package-management layer:

- Platform-neutral validation from package-relative file paths and bytes.
- Native directory loading on non-WASM targets.
- An opt-in in-memory registry with install/enable/disable operations.
- A browser/WASM bridge that accepts files supplied by the browser host and exposes installation, activation, listing, and asset-byte retrieval to JavaScript.
- Defensive checks for schema version, package size, asset size, file count, duplicate paths, missing assets, and path traversal.

This is **not yet a claim that arbitrary custom fighters are playable in the fight simulation**. The current combat/runtime tables and FightSimulation are still based on the original 15 fighters. The package loader is real; wiring arbitrary package definitions into the full fighter lifecycle, animation, collision, commands, AI, effects, portraits, and audio remains a separate integration milestone. The Chapolin sample is a data-package example, not a shipped binary or a fully playable C16 port.

## Package format

Each package is a directory with a character.json manifest and package-relative assets:

\`\`\`text
chapolin/
├── character.json
└── assets/
    └── chapolin.png        # artwork supplied/cleared by the package author
\`\`\`

Minimal manifest:

\`\`\`json
{
  "schema_version": 1,
  "id": "chapolin",
  "display_name": "Chapolin Colorado",
  "author": "StreetChaves custom character example",
  "version": "1.0.0",
  "fighter": {
    "attributes": {
      "sprite_height": 199,
      "walk_speed": 9.0
    },
    "moves": [
      { "id": "special_1", "name": "Corneta Paralisadora" },
      { "id": "special_2", "name": "Marreta Biônica" },
      { "id": "special_3", "name": "Chipote Voador" },
      { "id": "special_4", "name": "Pílulas de Nanicolina" },
      { "id": "special_5", "name": "Sigam-me os Bons!" }
    ],
    "animations": {}
  },
  "assets": {
    "fighter_strip": "assets/chapolin.png"
  }
}
\`\`\`

The Chapolin move names and observed sprite height/walk speed are source-derived from the Pascal C16 reference. This manifest intentionally does not pretend that an empty animation table or an arbitrary sprite is a complete C16 port. Package authors must provide the referenced asset; the loader rejects missing declared assets.

## Native installation

A native host can load a package directory without compiling the package into the executable:

\`\`\`rust
use street_chaves::custom_character::{load_directory, CustomCharacterRegistry};

let package = load_directory("custom-characters/chapolin")?;
let mut characters = CustomCharacterRegistry::default();
characters.install(package)?;
characters.enable("chapolin")?;
# Ok::<(), Box<dyn std::error::Error>>(())
\`\`\`

A game host can scan a user-writable directory such as:

\`\`\`text
<GameData>/
└── custom-characters/
    └── chapolin/
        ├── character.json
        └── assets/
            └── chapolin.png
\`\`\`

The package is data, not a plugin binary. Do not load executable code from character packages. Keep user-installed packages outside the read-only application installation directory and offer a clear enable/disable control.

## Browser and WebAssembly installation

WASM cannot assume direct access to the user's disk. The browser host must let the user select a package folder (or its files), read each selected file as bytes, and pass relative paths plus byte arrays into the WASM bridge.

The WASM bridge exports:

- install_custom_character_package(paths, contents)
- enable_custom_character(id)
- disable_custom_character(id)
- custom_character_is_enabled(id)
- list_custom_characters()
- get_custom_character_asset(id, asset_id)

The arrays passed to install must have the same length. Each path is relative to the package root; each content item is a Uint8Array. The bridge validates the exact same package format as native loading. It does not need a native filesystem, and no separate Web-only character format is introduced.

A minimal browser host flow (the host owns the file picker and reads the files):

\`\`\`javascript
async function installSelectedCharacter(files, wasm) {
  const paths = [];
  const contents = [];

  for (const file of files) {
    // webkitRelativePath preserves the selected directory structure.
    const path = file.webkitRelativePath.split("/").slice(1).join("/");
    paths.push(path);
    contents.push(new Uint8Array(await file.arrayBuffer()));
  }

  const installed = wasm.install_custom_character_package(paths, contents);
  wasm.enable_custom_character(installed.id);
  return installed;
}
\`\`\`

For a normal browser file input, set webkitdirectory where supported or allow multi-file selection and preserve each file's relative path. A host should display validation errors and never silently flatten files with duplicate names. The browser host decodes image/audio bytes and registers them with the renderer; the Rust game core only sees logical asset IDs and bytes.

The WASM bridge registry is in-memory for the lifetime of the WASM instance. Persistence across reloads is a host responsibility (for example IndexedDB or reselecting the folder). Package bytes are not baked into the WASM binary.

## Security and limits

- Maximum 256 files per package.
- Maximum 32 MiB total package size.
- Maximum 16 MiB per non-manifest asset.
- Only package-relative paths are accepted.
- Absolute paths, parent traversal, backslashes, drive-letter/colon paths, empty path segments, and control characters are rejected.
- Native directory loading rejects symlinks.
- Duplicate package IDs cannot be installed twice into the same registry.
- Package data is declarative JSON. No Rust, Pascal, JavaScript, WASM, or native plugin execution is supported.

These limits are conservative defaults for an initial implementation and can be revised with explicit tests and product requirements.

## Base roster compatibility

Custom package IDs are stable strings (for example, chapolin), not legacy numeric slots. They do not renumber C01-C15 or claim C16 is an original-roster member. Enabling a package changes only the custom-package registry; it does not yet insert a new fighter into the current fixed roster selection or simulation state.

The eventual roster integration must allocate a runtime identity independently from original IDs 1..15 and ensure packages cannot overwrite built-in fighters.

## Chapolin provenance and assets

The Pascal reference contains C16 INI/ROT data and generated art, but the existence of those files does not establish that every asset is safe to redistribute. This PR does not copy the C16 BMP/PNG art or executable ROT script into the Rust repository. The sample manifest records source-observed metadata and move names only. A distributable Chapolin package needs a separate provenance/permission review and cleared artwork/audio.

The Pascal C16 ROT script is executable game logic, not passive animation metadata. This loader does not execute arbitrary ROT supplied by custom packages. C16's full behavior must be translated into the documented, bounded custom-character behavior model and covered by tests before the sample is called playable.

## API example

The same core API is used by native and browser hosts:

- Native: call load_directory(path), then install/enable the returned package.
- Browser/WASM: call CustomCharacterPackage::from_files through the exported JS bridge using user-selected file bytes.

This keeps package validation and identity rules shared. Platform-specific file selection, persistence, image decoding, and rendering remain in the host.
