# WebAssembly host and Foundation-owned builds

## Ownership

StreetChavesRust remains an independent game project. It does not depend on ImpossibleEngine and is not being ported into it at this stage.

StreetChavesRust depends on RustMultiConsoleFoundation for native and browser presentation, runtime services and target builds:

- **StreetChavesRust:** gameplay, startup state machine, the 28 ms simulation clock, game-specific resource IDs, source asset mapping and browser bootstrap.
- **RustMultiConsoleFoundation:** native/browser host APIs, Cargo target compilation, WASM glue generation, HTML shell, asset ingestion, Web export manifest, ZIP and platform-specific build processing.
- **ImpossibleEngine:** separate editor and native `.ieproj` project format. StreetChavesRust does not import or link against it.

This separation stays in place until native and browser builds work reliably. Porting the game into ImpossibleEngine is a later decision, not a dependency of the current project.

## Build the Web version through Foundation

StreetChavesRust does not run `wasm-pack` or assemble the final package itself. Foundation compiles the Rust source and publishes the build.

Clone the source assets alongside the StreetChavesRust checkout:

```sh
git clone https://github.com/danimesq/OpenStreetChaves.git ../OpenStreetChaves
```

Install the Web target and a `wasm-bindgen` CLI version matching the crate version used by the project:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

With the Foundation repository available as a sibling checkout, run its build command from the StreetChavesRust root:

```sh
cargo run \
  --manifest-path ../RustMultiConsoleFoundation/Cargo.toml \
  -p foundation-packaging \
  --bin foundation-web-build -- \
  --config FoundationWebBuild.toml
```

The root [FoundationWebBuild.toml](../FoundationWebBuild.toml) describes:

- The StreetChavesRust Cargo package, library crate name and `web-foundation` feature.
- The `wasm32-unknown-unknown` release build.
- The local OpenStreetChaves asset root.
- Stable game-facing resource IDs and package paths.
- Source URLs retained in the published provenance file.
- The HTML canvas, browser bootstrap, output directory and ZIP archive.

Foundation then runs the Cargo build, runs `wasm-bindgen`, generates `index.html`, stages the bootstrap and resources, writes `web-export.json`, emits `web-build-provenance.json`, and creates `dist/street-chaves-web.zip`.

The consumer supplies source code and an asset catalog. It does **not** supply a precompiled WASM file and does not own the packaging step.

## Included source resources

The current build configuration includes the resources used by the first browser fight loop:

- Stage 01 background: [Cenarios/Cenario-01.bmp](https://github.com/danimesq/OpenStreetChaves/blob/main/Cenarios/Cenario-01.bmp).
- Fighter sprite sheets C01–C15: [Lutadores/](https://github.com/danimesq/OpenStreetChaves/tree/main/Lutadores).
- Fighter metadata C01–C15: [Dados/Lutadores/](https://github.com/danimesq/OpenStreetChaves/tree/main/Dados/Lutadores).
- Game settings and resource index: [Dados/Jogo.ini](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Jogo.ini) and [Dados/Recursos.ini](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Recursos.ini).
- Sound bank slots 001–071: [Sons/](https://github.com/danimesq/OpenStreetChaves/tree/main/Sons).
- Fight music and menu music: [Musicas/](https://github.com/danimesq/OpenStreetChaves/tree/main/Musicas) and [Embutidos/Musica.wav](https://github.com/danimesq/OpenStreetChaves/blob/main/Embutidos/Musica.wav).

The full upstream repository also contains all 13 stage images, 16 fighter sheets, stage INIs, 97 animation images, 111 effect images, 48 portraits, 32 voice files, opening/ending artwork and font resources. They are linked in [the CHS screen status and source-assets map](CHS-SCREEN-STATUS.md). The configuration can expand as those game systems are implemented.

The upstream source repository carries a [GPL-3.0 license](https://github.com/danimesq/OpenStreetChaves/blob/main/LICENSE). Each configured source entry retains its source URL in the Foundation build provenance.

## Runtime resource contract

The browser host consumes Foundation's root `web-export.json`. Its resource IDs are game-facing names rather than filenames:

| ID | Purpose |
|---|---|
| `stage_XX` | Stage image |
| `fighter_XX` | Fighter sprite strip |
| `fighter_ini_XX` | Fighter INI metadata, including sprite height, speed and CentroX pivots |
| `game_settings` | Damage/game settings |
| `resource_settings` | Original resource index |
| `effect_slot_NNN` | Audio clip at the matching Pascal sound-bank slot |
| `fight_music` | Browser fight music |
| `menu_music` | Browser menu music |

The current `load_web_resources()` implementation loads those IDs, decodes the fighter/stage images and buffered audio, and parses the INI bytes required by the first fight loop. The loader uses Canvas 2D and WebAudio through Foundation. Streaming music, WebGL and browser gamepad/touch adapters are not implemented in this slice.

## Browser entry point

Foundation generates the HTML shell and copies [web/main.js](../web/main.js) as the configured bootstrap. That module:

1. Initializes the generated `street_chaves.js` WebAssembly glue.
2. Creates the `StreetChavesWebRuntime` on the generated canvas.
3. Loads `./web-export.json`.
4. Resizes the backing canvas for CSS dimensions and device-pixel ratio.
5. Drives presentation while StreetChavesRust retains the 28 ms logical tick.
6. Resumes WebAudio from the explicit user-gesture button.

The bootstrap is game-specific source owned by StreetChavesRust. Foundation owns the compilation and publication that places it beside the generated WASM module.

## Current browser limits

- Canvas 2D is the active browser renderer; WebGL is deferred.
- Audio clips are decoded into WebAudio buffers. Music streaming is not implemented.
- The current game bootstrap uses keyboard input; browser gamepad and pointer/touch adapters remain future work.
- The current asset configuration stages the resources used by the first fight loop. Opening, campaign, attract, opponent selection, credits, settings and endings still require their respective game systems and additional resource mappings.
