# Street Chaves Rust: WebAssembly export harness

This directory is a standalone HTML5 host example for the Foundation-backed WASM runtime.

## Build

Run from the repository root:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web --features web-foundation --out-dir pkg
```

## Prepare exported media

Copy only media/data that you are authorized to use into `web/assets/`, following the paths in `assets/street-chaves-manifest.example.json`. Copy that file to `web/assets/street-chaves-manifest.json` and edit the asset list for your package.

The compact example includes only Chaves and Quico. A complete roster build should include `fighter_NN` and `fighter_ini_NN` for all 15 original fighters and the corresponding stage/audio files. Audio entries may provide fallback encodings for browser codec differences.

## Serve the page

Do not open the page using `file://`. Serve the repository root over HTTP(S), for example:

```sh
python3 -m http.server 8000
```

Then open `http://localhost:8000/web/`.

The player should press **Enable audio** once to resume the browser AudioContext. Keyboard controls mirror the native host:

- P1: arrows + Z/X/C/V.
- P2: WASD + R/T/Y/U.
- P1 custom special slots: F1-F5.
- P2 custom special slots: Numpad 1-5.
- Escape: request quit.

The HTML host drives presentation with `requestAnimationFrame`; the Rust game host retains the fixed 28 ms logical tick.
