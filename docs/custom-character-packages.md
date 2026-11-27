# Custom character packages

## Status

Custom characters now have a shared package loader **and a bounded gameplay runtime**.

- Native directory loading and browser/WASM byte handoff use the same `character.json` package format.
- Runtime IDs are strings at package level and map to numeric engine IDs starting at 16. Built-in C01-C15 are never renumbered or replaced.
- A validated definition supplies frame-centre offsets, hurtboxes, ordered attack rows, move timelines, push windows, invulnerability gates, sound/effect events and post-hit recoil.
- `FightSimulation` can register a package, select it into either fighter slot, start its named moves and execute its frames through the same tick path used by built-in fighters.
- The runtime interprets declarative data only. It never executes packaged Pascal ROT, Rust, JavaScript, WASM or native code.

The Chapolin C16 example is a **repository-only documentary/custom-character example**. Its five specials, 50 centre offsets, 50 hurtboxes and 20 attack rows are declarative data exercised by tests. C16 is not part of the default roster, is not linked into the release executable and is not shipped with the compiled program. The repository does not bundle the original Chapolin bitmap/sprite sheet or audio files. Users may copy/adapt the sample into their own custom-character folder after reviewing provenance and asset rights.

## Package layout

The native user-data tree deliberately reuses the original Street Chaves directory vocabulary. A custom package can live below the fighter-data root:

```text
<GameData>/
├── Animacoes/
├── Cenarios/
├── Dados/
│   └── Lutadores/
│       └── Custom/
│           └── chapolin/
│               ├── character.json
│               └── assets/
│                   └── chapolin.png
├── Efeitos/
├── Lutadores/
├── Rostos/
└── Sons/
```

The directory names are familiar, but the Rust custom package uses the bounded `character.json` schema. Matching the Delphi folder layout does **not** mean arbitrary Pascal `.ini`/`.rot` files are loaded or executed. Those files remain references, not a code-injection interface.

The example at `examples/custom-characters/chapolin/` contains source-derived C16 logic tables and move timelines for documentation/testing. The original source repository also provides [the C16 sprite sheet](https://github.com/danimesq/OpenStreetChaves/blob/main/Lutadores/C16.bmp) and [its fighter metadata](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C16.ini). The release executable and default browser package do not bundle C16.

## Native installation and use

A native host loads the directory, registers the package with the simulation, selects the fighter and maps input to a named move:

```rust
use street_chaves::custom_character::load_directory;

let package = load_directory("<GameData>/Dados/Lutadores/Custom/chapolin")?;
let runtime_id = fight.register_custom_character_package(package)?;
assert_eq!(runtime_id, 16);

fight.select_custom_character(0, "chapolin")?;
fight.queue_custom_character_move(0, "special_2", 2)?; // strong Marreta Biônica
```

A typical user-writable installation path is:

```text
<GameData>/
└── custom-characters/
    └── chapolin/
        ├── character.json
        └── assets/
            └── chapolin.png
```

Copying a package folder does not rebuild or modify the executable. The host decides whether a package should be selected; installing a package does not silently replace an existing roster slot.

## Web, WebAssembly and other sandboxed platforms

The native directory convention is **not portable by itself**. A WebAssembly build cannot freely scan `<GameData>/Dados/Lutadores/Custom` on the user's disk, and other sandboxed platforms have similar restrictions. They need a platform-specific import flow: folder/file picker, preservation of relative paths, byte transfer into the Rust host and an explicit persistence strategy. IndexedDB or another platform storage API may be appropriate, but that picker/storage workflow is future host work.

The shared package validator is byte-oriented. The browser bridge exposes:

- `install_custom_character_package(paths, contents)`
- `enable_custom_character(id)`
- `disable_custom_character(id)`
- `custom_character_is_enabled(id)`
- `list_custom_characters()`
- `get_custom_character_asset(id, asset_id)`

Each content item is a `Uint8Array`; every path is relative to the imported package root. The host then constructs the same `CustomCharacterPackage` and registers it with `FightSimulation`. This does **not** turn the browser build into a Delphi filesystem scanner, nor does it make native folder-copy installation work automatically on Web or other platforms.

A browser implementation could use `webkitdirectory` where supported or multi-file selection while preserving every relative path. The actual picker, permissions, persistent storage, update/removal UI and equivalent adapters for other targets still need to be designed. Never flatten files into basenames because separate folders may contain duplicate file names.

## Shared gameplay API

The important engine entry points are:

- `register_custom_character_package(package)`: validate the declarative runtime definition and install/activate it.
- `select_custom_character(fighter_index, package_id)`: select it for player 1 or player 2.
- `queue_custom_character_move(fighter_index, move_id, force_strength)`: start a named weak/strong special.
- `custom_character_definition(runtime_id)`: expose validated frame-centre offsets, frame boxes and move metadata to render hosts.
- `custom_character_asset(runtime_id, asset_id)`: retrieve optional packaged asset bytes.

The move queue is deliberately a host-facing input seam. Built-in C01-C15 command recognizers remain intact; a host maps its custom-character controls to a package move ID rather than injecting executable command scripts.

## Declarative C16 gameplay

The Chapolin manifest records the source-observed attributes `AlturaSprite=199` and `Velocidade=9.0`, all 50 `CentroX` rows, all 50 `Corpo` rows and the 20 ordered `Ataque` rows. It also maps the five special actions:

| Move | Action | Weak / strong duration | Implemented behavior |
|---|---:|---:|---|
| Corneta Paralisadora | 25 | 30 / 30 | Frame timeline, sound and effect 37 at frame 10 |
| Marreta Biônica | 26 | 16 / 24 | Strength-specific hammer timeline, strong push and recoil 20 on strong hit |
| Chipote Voador | 27 | 10 / 16 | Rising push, kick classification and opponent-invulnerability gates |
| Pílulas de Nanicolina | 28 | 34 / 34 | Shrink/run/grow timeline, strength-specific push and recoil 15 on hit |
| Sigam-me os Bons! | 29 | 38 / 38 | Power/low-health gate, invulnerability through frame 16, slam, effect 38 and screen shake |

The runtime advances one declarative frame per simulation tick, applies the matching C16 body/attack table rows, updates push/invulnerability state and emits typed effect events. Sound IDs are exposed through the existing pending-sound field.

The Pascal C16 source is a fork extension, not an original C01-C15 roster member:

- [C16.ini](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C16.ini)
- [C16.rot](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C16.rot)

The source ROT file is used as a behavioral reference only. Its code is not loaded or executed by this package.

## Security and compatibility

- Maximum 256 files per package.
- Maximum 32 MiB total package size.
- Maximum 16 MiB per non-manifest asset.
- Only package-relative paths are accepted; absolute paths and traversal are rejected.
- Native loading rejects symlinks.
- Duplicate package IDs and duplicate runtime IDs are rejected.
- Built-in IDs 1..15 remain reserved. Custom IDs must be 16..255.
- Declarative frame arrays, move counts, durations, push values and numeric fields are bounded and validated.
- Custom packages cannot inject code or mutate arbitrary engine fields.
- Package installation does not execute arbitrary `.rot` content.

## Remaining host work

The core now simulates custom moves and emits typed effect requests. A graphical host still needs to provide/clear sprite assets, decode and draw those assets, render emitted custom-effect events and map keyboard/controller input to custom move IDs. Web persistence (for example IndexedDB) and native package discovery are host-level concerns. The repository's fight core remains renderer-independent.
