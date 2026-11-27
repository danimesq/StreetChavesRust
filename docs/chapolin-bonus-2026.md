# Chapolin C16: documentary custom-character example

## Decision

Chapolin is a **documentary/custom-character example only**. He is not a built-in sixteenth fighter, not part of the standard roster and **will not be shipped with the compiled program**.

The canonical roster remains C01-C15. The source repository keeps the C16 example under `examples/custom-characters/chapolin/` so users can inspect the package structure, learn from the declarative move data and adapt the sample for their own custom character. The sample must not be embedded in the release binary or added to the default Web package.

## Native folder convention

The native custom-character location intentionally follows the original Street Chaves/Delphi data-folder vocabulary:

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

This is a familiar native data-tree convention, not a promise of legacy format compatibility. The Rust custom-character loader validates the bounded `character.json` schema. It does not execute arbitrary Pascal `.ini` or `.rot` files from a user's folder. Those files remain source references.

## What the example contains

The repository sample represents source-derived C16 attributes, five special-move timelines, 50 centre offsets, 50 hurtboxes and 20 attack rows as declarative data. It can be used for headless runtime tests and as a template for a user's own package.

The original Chapolin sprite sheet and audio are not bundled. Users must provide assets they have the right to use. The sample's presence in Git is not permission to redistribute third-party artwork or sound.

## Web and other platforms

Copying a folder into `<GameData>/Dados/Lutadores/Custom` is a native filesystem workflow. It does **not** work automatically in WebAssembly or other sandboxed targets. A browser cannot freely scan the user's local data directory. It needs a separate import mechanism such as a folder/file picker, preservation of package-relative paths, byte transfer into WASM and a persistence strategy. Other platforms need equivalent platform-specific adapters.

The shared Rust package validator already accepts package-relative paths and bytes, so the package schema can remain common. The host-side picker, storage and management experience still needs to be designed for Web and each other restricted platform. Matching the Delphi directory structure alone does not solve that platform boundary.

## Non-goals

- Do not add Chapolin to the canonical C01-C15 roster.
- Do not bundle C16 into the executable, default Web package or release assets.
- Do not execute arbitrary Pascal ROT or user-supplied native code.
- Do not imply that familiar folder names make legacy `.ini`/`.rot` drop-in compatible.
- Do not redistribute art/audio without provenance and permission review.

## References

- [C16.ini](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C16.ini)
- [C16.rot](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C16.rot)
- [C16 sprite sheet](https://github.com/danimesq/OpenStreetChaves/blob/main/Lutadores/C16.bmp)
- [Custom-character package format](custom-character-package-format.md)
- [Custom-character package runtime](custom-character-packages.md)
