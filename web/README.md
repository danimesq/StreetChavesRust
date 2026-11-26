# Street Chaves Rust: Foundation Web build

This folder contains the game-owned browser bootstrap and HTML source template. Rust compilation, WebAssembly binding generation, HTML assembly, resource staging and ZIP publication are owned by RustMultiConsoleFoundation.

## Source checkouts

From the directory containing the StreetChavesRust checkout, clone the two independent inputs:

```sh
git clone https://github.com/danimesq/OpenStreetChaves.git ../OpenStreetChaves
git clone https://github.com/danimesq/RustMultiConsoleFoundation.git ../RustMultiConsoleFoundation
```

The OpenStreetChaves checkout supplies the source BMP, INI and WAV resources referenced by [FoundationWebBuild.toml](../FoundationWebBuild.toml). The Foundation checkout supplies the source build/publishing command.

## Build

Install the WebAssembly target and the `wasm-bindgen` CLI matching the crate version used by StreetChavesRust:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

After the Foundation source-build PR is merged to `main`, run this from the StreetChavesRust repository root:

```sh
cargo run \
  --manifest-path ../RustMultiConsoleFoundation/Cargo.toml \
  -p foundation-packaging \
  --bin foundation-web-build -- \
  --config FoundationWebBuild.toml
```

Foundation performs the Cargo build, runs `wasm-bindgen`, generates the HTML shell, copies the configured source assets, writes `web-export.json` and `web-build-provenance.json`, and creates:

```text
dist/
  street-chaves-web/
    index.html
    main.js
    web-export.json
    web-build-provenance.json
    pkg/
      street_chaves.js
      street_chaves_bg.wasm
    assets/
      images/
      audio/
      data/
  street-chaves-web.zip
```

## Source resources

The configuration currently includes the first-stage assets required by the browser fight loop: stage 01, all 15 original fighter sheets and INI metadata, the game/resource INIs, all 71 sound-bank WAV files, fight music and menu music.

The upstream resource directories remain available for extending the package as more game systems are implemented: [Cenarios](https://github.com/danimesq/OpenStreetChaves/tree/main/Cenarios), [Lutadores](https://github.com/danimesq/OpenStreetChaves/tree/main/Lutadores), [Dados](https://github.com/danimesq/OpenStreetChaves/tree/main/Dados), [Sons](https://github.com/danimesq/OpenStreetChaves/tree/main/Sons), [Musicas](https://github.com/danimesq/OpenStreetChaves/tree/main/Musicas), [Falas](https://github.com/danimesq/OpenStreetChaves/tree/main/Falas), [Embutidos](https://github.com/danimesq/OpenStreetChaves/tree/main/Embutidos), [Animacoes](https://github.com/danimesq/OpenStreetChaves/tree/main/Animacoes), [Efeitos](https://github.com/danimesq/OpenStreetChaves/tree/main/Efeitos), [Rostos](https://github.com/danimesq/OpenStreetChaves/tree/main/Rostos) and [Fontes](https://github.com/danimesq/OpenStreetChaves/tree/main/Fontes).

The upstream repository includes its [GPL-3.0 license](https://github.com/danimesq/OpenStreetChaves/blob/main/LICENSE); the Foundation output records each configured source URL in its provenance file.

## Serve the published package

```sh
python3 -m http.server 8000 --directory dist/street-chaves-web
```

Open `http://localhost:8000/`. The HTML page's Enable audio button resumes WebAudio after the browser's required user gesture.

The Rust simulation retains its 28 ms logical tick. Browser `requestAnimationFrame` schedules presentation only.
