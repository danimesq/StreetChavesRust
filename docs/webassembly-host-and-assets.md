# WebAssembly browser host and Foundation export package

## Scope and ownership

StreetChavesRust uses RustMultiConsoleFoundation for native and browser presentation. The game simulation, startup flow, 28 ms fixed timestep, resource interpretation, presentation composition and audio events remain in StreetChavesRust. Foundation selects and supplies the platform host:

- Native: SDL3 through `foundation-runtime/native-sdl3`.
- Browser: Canvas 2D + WebAudio through `foundation-runtime/web-canvas`.
- WebGL is intentionally deferred.

The browser build uses the shared Foundation Web/WASM export API. The same manifest contract is intended for game exporters and engine publishing backends, including ImpossibleEngine.

## Build the browser package

The browser adapter targets `wasm32-unknown-unknown`, not Emscripten:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web --features web-foundation --out-dir web/pkg
```

This produces the JavaScript glue and WASM module under `web/pkg/`. The native build continues to use:

```sh
cargo run --features native-foundation --bin street-chaves-native
```

## Foundation Web export package

The published package uses one `web-export.json` manifest at its root. A suggested directory layout is:

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
      stages/
      fighters/
    audio/
      effects/
      music/
    data/
```

All manifest paths are relative to `web-export.json`. The package keeps the JavaScript glue, WASM module and game resources in explicit locations. Folder names and filename case are preserved.

The example manifest lives at [web-export.example.json](../web/web-export.example.json). Its resource paths are a catalog template: the repository does not bundle the original copyrighted game media.

### Create the package manifest with Foundation

StreetChavesRust depends on the public `foundation_runtime::web_export` API. A game build/publishing step can create the same descriptor programmatically:

```rust
use foundation_runtime::web_export::{
    WebAssetKind, WebExportAsset, WebExportBuilder, WEB_EXPORT_MANIFEST_FILE,
};

let manifest = WebExportBuilder::new("street-chaves", "Street Chaves")
    .wasm_bundle(
        "pkg/street_chaves_rust.js",
        "pkg/street_chaves_rust_bg.wasm",
    )
    .asset(WebExportAsset::new(
        "stage_01",
        "assets/images/stages/stage-01.bmp",
        WebAssetKind::Image,
    ))
    .asset(WebExportAsset::new(
        "fighter_ini_01",
        "assets/data/C01.ini",
        WebAssetKind::Data,
    ))
    .asset(
        WebExportAsset::new(
            "fight_music",
            "assets/audio/music/Musica-01.ogg",
            WebAssetKind::Audio,
        )
        .with_fallbacks([
            "assets/audio/music/Musica-01.m4a",
            "assets/audio/music/Musica-01.mp3",
        ]),
    )
    .build()?;

std::fs::write(
    output_dir.join(WEB_EXPORT_MANIFEST_FILE),
    manifest.to_json()?,
)?;
```

The builder validates the application identity, generated WASM/JavaScript paths, resource IDs, relative paths, fallback paths and volume values. The consuming build step is responsible for compiling the game crate and copying every declared file to its matching package path. This integration slice does not yet add an archive writer, HTML shell generator, media transcoder or automatic asset optimizer.

## Street Chaves resource catalog

The example catalog maps game-facing IDs to explicit package paths:

| ID | Meaning |
|---|---|
| `stage_NN` | Stage image for stage NN |
| `fighter_NN` | Fighter sprite strip for character NN |
| `fighter_ini_NN` | Fighter INI bytes, including sprite height, speed and CentroX pivots |
| `game_settings` | Optional game settings INI bytes |
| `effect_slot_NNN` | Zero-based Pascal sound-bank slot NNN |
| `fight_music` | Optional looping track for the selected fight |

Include the selected fighter's sprite strip and INI file for every character selectable in the shipped build. The compact example includes only Chaves and Quico; a complete roster catalog must enumerate every shipped fighter and any custom packages it supports.

Audio entries may provide alternate encodings through `fallbacks` and a per-resource `volume_percent` (0–400, default 100), preserving the game's sound-bank volume settings. Foundation tries each declared path until the browser fetches and decodes one successfully. The browser's `decodeAudioData()` codec support varies by platform.

Effects and music may be placed in separate directories to keep the package organized. Both currently use Foundation's buffered WebAudio path; streamed music is not implemented by this host.

## HTML entry point

A minimal standalone page can load the generated WASM module and host the game:

```html
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Street Chaves Rust</title>
  <style>
    body { margin: 0; background: #101522; color: white; font-family: sans-serif; }
    main { display: grid; place-items: center; min-height: 100vh; }
    canvas { width: min(100vw, 133.333vh); aspect-ratio: 4 / 3; image-rendering: pixelated; }
  </style>
</head>
<body>
  <main>
    <section>
      <canvas id="street-chaves" width="400" height="300"></canvas>
      <button id="enable-audio">Enable audio</button>
      <pre id="diagnostics"></pre>
    </section>
  </main>
  <script type="module" src="./main.js"></script>
</body>
</html>
```

`main.js` (after running `wasm-pack`):

```js
import init, { StreetChavesWebRuntime } from "./pkg/street_chaves_rust.js";

await init();
const game = new StreetChavesWebRuntime("street-chaves");
await game.load_assets("./web-export.json");

document.querySelector("#enable-audio").addEventListener("click", async () => {
  await game.resume_audio();
});

const canvas = document.querySelector("#street-chaves");
const resize = () => {
  const rect = canvas.getBoundingClientRect();
  game.resize_canvas(
    Math.max(1, Math.round(rect.width)),
    Math.max(1, Math.round(rect.height)),
    window.devicePixelRatio || 1
  );
};
new ResizeObserver(resize).observe(canvas);
window.addEventListener("resize", resize);
resize();

function frame() {
  try {
    game.frame();
    const messages = game.take_diagnostics();
    if (messages.length) {
      document.querySelector("#diagnostics").textContent = messages.join("\n");
    }
    if (!game.should_quit()) requestAnimationFrame(frame);
  } catch (error) {
    document.querySelector("#diagnostics").textContent = String(error);
  }
}
requestAnimationFrame(frame);
```

The Enable audio button must be clicked/tapped because browsers can suspend a new AudioContext until a user gesture. The game can then proceed with its normal Start interaction.

## Shared runtime contract

The WASM host instantiates the same generic `FightPresentationHost` as the native executable:

- The native alias remains `NativeFightHost`.
- The browser alias is `WebFightHost`.
- Both use Foundation `RuntimeHost` and `TextureRegistrar`.
- The game host owns the fixed timestep and submits the same ordered `DrawCommand` output.
- Foundation owns the actual renderer/audio devices and platform texture handles.

The browser animation loop is presentation scheduling only. The 28 ms logical simulation ticks are computed from Foundation's monotonic browser clock and never tied to the `requestAnimationFrame` frequency. When the page becomes hidden, the harness stops presenting; on visibility return it rebases the 28 ms accumulator and releases held keys so background time cannot become a burst of catch-up simulation ticks.

## Current browser limitations

- Canvas 2D is the only web renderer in this integration; WebGL is deferred.
- Keyboard input uses the same two-player bindings as native. Gamepad/touch support is a later adapter capability.
- WebAudio delegates final mixing to the browser. Clip IDs, volume, loop, stop, restart and voice-limit behavior are preserved at the host contract; bit-exact S16 PCM output is not claimed.
- Custom character packages continue to use the existing host-supplied byte package API. This initial export catalog covers the original fighter/stage/audio/data resource IDs; generic custom-package rendering and registration remain separate integration work.
- The browser must serve the manifest and resources via HTTP(S); CORS and browser codec policies apply.
- This is a Foundation manifest and runtime integration, not a complete publishing application. The game build still creates the WASM bundle and stages the declared resource files.
