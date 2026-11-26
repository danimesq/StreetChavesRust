# Street Chaves Rust: Foundation WebAssembly package

This folder contains the HTML shell, browser entry point and example resource manifest for the Foundation-backed Web/WASM build. The deployable package is assembled by the `street-chaves-web-export` command, which calls Foundation's `WebExportBuilder`.

## Build

Run from the repository root:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web --features web-foundation --out-dir pkg
```

This creates the generated JavaScript glue and WASM module under the repository's `pkg/` directory.

## Supply local media

Copy only media/data that you are authorized to use into `web/assets/`, following the paths in [web-export.example.json](web-export.example.json). The repository does not bundle original copyrighted game media.

The compact example includes Chaves and Quico, one stage, a sample effect/music entry and their INI data. A complete roster build should include `fighter_NN` and `fighter_ini_NN` for every fighter selected for that package, plus the required stage/audio/data resources.

The builder requires every primary resource and every declared fallback source to exist. Remove unavailable fallback paths from the manifest or supply the corresponding alternate encoding; it will not silently publish a broken package.

## Assemble the deployable directory

After the WASM build and local media are ready, run:

```sh
cargo run --bin street-chaves-web-export -- \
  web/web-export.example.json \
  pkg/street_chaves_rust.js \
  pkg/street_chaves_rust_bg.wasm \
  web \
  web/index.html \
  web/main.js \
  dist/street-chaves-web
```

The command reads the manifest, registers the generated bundle and all resource sources with `WebExportBuilder`, stages the page shell and assets, and writes `web-export.json` into `dist/street-chaves-web/`.

The resulting package has this shape:

```text
dist/street-chaves-web/
  index.html
  main.js
  web-export.json
  pkg/
    street_chaves_rust.js
    street_chaves_rust_bg.wasm
  assets/
    images/
    audio/
    data/
```

Foundation preserves the exact case and relative paths declared by the manifest. It rejects unsafe paths, duplicate destinations and missing source registrations.

## Serve the package

Do not open the page using `file://`. Serve the assembled directory over HTTP(S):

```sh
python3 -m http.server 8000 --directory dist/street-chaves-web
```

Then open `http://localhost:8000/`.

The player should press **Enable audio** once to resume the browser AudioContext. Keyboard controls mirror the native host:

- P1: arrows + Z/X/C/V.
- P2: WASD + R/T/Y/U.
- P1 custom special slots: F1-F5.
- P2 custom special slots: Numpad 1-5.
- Escape: request quit.

The HTML host drives presentation with `requestAnimationFrame`; the Rust game host retains the fixed 28 ms logical tick. The browser runtime loads resources from the root `web-export.json` manifest through `Canvas2dHost::load_web_export()`.
