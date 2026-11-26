# Foundation WebAssembly host contract

## Purpose

StreetChavesRust is a consumer of RustMultiConsoleFoundation's shared runtime and Web/WASM publishing library. The browser package is built from StreetChavesRust's own Rust sources, generated JavaScript/WASM glue, selected game assets and a small HTML entry page.

There is no external editor plugin in this architecture. Foundation is a Rust library used by the game build and runtime:

1. The game compiles its WASM module and JavaScript glue.
2. The build supplies the generated files, the assets it wants to publish and their package-relative destination paths.
3. `WebExportBuilder` validates and stages those files into a deployable package directory.
4. Foundation writes `web-export.json` at the package root.
5. The browser host loads that manifest and fetches/registers resources by stable game-facing IDs.

## Package layout

```text
web/
  index.html
  main.js
  web-export.json
  pkg/
    street_chaves_rust.js
    street_chaves_rust_bg.wasm
  assets/
    images/
    audio/
      effects/
      music/
    data/
```

All manifest paths are relative to `web-export.json`. The manifest lists the compiled JavaScript/WASM bundle and the game resources. Resource IDs remain stable when physical files are renamed or moved.

## Publisher API

The shared API is `foundation_runtime::web_export`:

- `WebExportBuilder::new()` declares application identity.
- `wasm_bundle_files()` registers the generated JavaScript glue and WASM module.
- `asset_file()` registers a resource ID, kind, published path and source file.
- `package_file()` registers the HTML shell, entry point, fallback encodings and other package files.
- `write_to()` validates and stages the complete package, then writes the manifest.

Example:

```rust
use foundation_runtime::web_export::{
    WebAssetKind, WebExportAsset, WebExportBuilder,
};

WebExportBuilder::new("street-chaves", "Street Chaves")
    .wasm_bundle_files(
        "web/pkg/street_chaves_rust.js",
        "web/pkg/street_chaves_rust_bg.wasm",
        "pkg/street_chaves_rust.js",
        "pkg/street_chaves_rust_bg.wasm",
    )
    .asset_file(
        WebExportAsset::new(
            "stage_01",
            "assets/images/stages/stage-01.bmp",
            WebAssetKind::Image,
        ),
        "verified-assets/stage-01.bmp",
    )
    .package_file("web/index.html", "index.html")
    .package_file("web/main.js", "main.js")
    .write_to("dist/street-chaves-web")?;
```

Fallback encodings declared in an asset's `fallbacks` list must also be staged with `package_file()` at the exact destination paths listed in the manifest. All required bundle and resource files must be registered. Absolute paths, traversal segments, duplicate destinations and attempts to overwrite the reserved manifest path are rejected.

## Browser host

The optional Foundation `web-canvas` feature implements the shared runtime contract for `wasm32-unknown-unknown`:

- Canvas 2D presentation using ordered renderer-neutral draw commands.
- Browser keyboard normalization.
- WebAudio decoding and playback using stable audio clip IDs.
- Image decoding and registration using stable texture IDs.
- Data-resource fetching that returns unmodified bytes to the game.
- Loading through `Canvas2dHost::load_web_export()`.

The game owns its simulation, fixed timestep, game state and mapping from its domain resource IDs to Foundation texture/audio IDs. Foundation owns browser resource loading, decoding and platform handles.

## Versioning and validation

The manifest has an independent `schema_version`. Consumers should validate the schema before loading resources and preserve explicit resource IDs and paths. New optional package capabilities must be versioned and covered by fixtures.

A minimum package fixture includes:

- One generated JavaScript file and one WASM module.
- One image, one audio file and one data file.
- At least one nested resource directory.
- A filename with mixed case to verify path preservation.
- One optional audio fallback with a matching staged source file.
- A malformed path that attempts to escape the package and must be rejected.

## Current implementation boundary

The library currently stages directory packages and writes the manifest. It does not yet create ZIP archives, compile the consumer's WASM module, generate an HTML shell, transcode media, optimize images or provide a publishing CLI.

The browser host uses Canvas 2D and buffered WebAudio. Music streaming, WebGL, browser gamepad and pointer/touch adapters are not implemented by this first slice. The same game core can still build native and browser targets without depending on browser APIs.
