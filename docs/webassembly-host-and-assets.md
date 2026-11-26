# WebAssembly browser host and exported assets

## Scope

StreetChavesRust uses RustMultiConsoleFoundation for native and browser graphics/audio. The game simulation, startup flow, 28 ms fixed timestep, Pascal resource interpretation, presentation composition and audio events remain Rust. The Foundation selects the platform host:

- Native: SDL3 through `foundation-runtime/native-sdl3`.
- Browser: Canvas 2D + WebAudio through `foundation-runtime/web-canvas`.
- WebGL is intentionally deferred.

## Build the browser package

The browser adapter targets `wasm32-unknown-unknown`, not Emscripten:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web --features web-foundation
```

This generates the JavaScript glue and WASM module in `pkg/`. The native build continues to use:

```sh
cargo run --features native-foundation --bin street-chaves-native
```

## Exported HTML5 asset layout

The web build follows the familiar Construct-style export model: HTML, JavaScript, WASM, a JSON manifest, and the referenced media/data files are served together. The browser never receives native filesystem paths.

Suggested layout:

```text
web/
  index.html
  main.js
  assets/
    street-chaves-manifest.json
    stages/
      stage-01.bmp
    fighters/
      C01.bmp
      C02.bmp
      ...
    data/
      C01.ini
      C02.ini
      ...
      Jogo.ini
    audio/
      Som-01.ogg
      Som-01.m4a
      Som-02.ogg
      ...
      Musica-01.ogg
      Musica-01.m4a
```

Copy only game assets that the distributor is legally entitled to use. The repository does not bundle original copyrighted game media.

### Manifest example

```json
{
  "schema_version": 1,
  "assets": [
    {
      "id": "stage_01",
      "path": "stages/stage-01.bmp",
      "kind": "image"
    },
    {
      "id": "fighter_01",
      "path": "fighters/C01.bmp",
      "kind": "image"
    },
    {
      "id": "fighter_ini_01",
      "path": "data/C01.ini",
      "kind": "data"
    },
    {
      "id": "fighter_04",
      "path": "fighters/C04.bmp",
      "kind": "image"
    },
    {
      "id": "fighter_ini_04",
      "path": "data/C04.ini",
      "kind": "data"
    },
    {
      "id": "game_settings",
      "path": "data/Jogo.ini",
      "kind": "data"
    },
    {
      "id": "effect_slot_001",
      "path": "audio/Som-02.ogg",
      "fallbacks": ["audio/Som-02.m4a"],
      "kind": "audio"
    },
    {
      "id": "fight_music",
      "path": "audio/Musica-01.ogg",
      "fallbacks": ["audio/Musica-01.m4a"],
      "kind": "audio"
    }
  ]
}
```

The runtime expects these resource IDs:

| ID | Meaning |
|---|---|
| `stage_NN` | Stage bitmap/image for stage NN |
| `fighter_NN` | Fighter sprite strip for character NN |
| `fighter_ini_NN` | Fighter INI data bytes, including sprite height, speed and CentroX pivots |
| `game_settings` | Optional game settings INI data bytes |
| `effect_slot_NNN` | Zero-based Pascal sound-bank slot NNN |
| `fight_music` | Optional looping track for the selected fight |

Include the chosen fighter's sprite strip and INI file for every character selectable in the shipped build. The example above includes only Chaves and Quico as a compact demonstration; a full roster manifest should enumerate all 15 original fighters and any custom packages it supports.

Audio entries may provide alternate encodings using `fallbacks` and a per-resource `volume_percent` (0–400, default 100), preserving the original sound-bank volume configuration in the browser package. The Foundation tries each path until the browser fetches and decodes one successfully. Use formats appropriate to the deployment browsers, for example OGG plus M4A/MP3 alternatives. The decoder is the browser's native `decodeAudioData()`; supported codecs vary by browser.

## HTML entry point

A minimal standalone page can load the WASM module and host the game:

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

`main.js` (after running wasm-pack):

```js
import init, { StreetChavesWebRuntime } from "../pkg/street_chaves_rust.js";

await init();
const game = new StreetChavesWebRuntime("street-chaves");
await game.load_assets("./assets/street-chaves-manifest.json");

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

The Enable audio button must be clicked/tapped by the player because browsers can suspend a new AudioContext until a user gesture. The game's keyboard Start interaction can then proceed normally.

## Shared runtime contract

The WASM host instantiates the same generic `FightPresentationHost` as the native executable:

- The native alias remains `NativeFightHost`.
- The browser alias is `WebFightHost`.
- Both use Foundation `RuntimeHost` and `TextureRegistrar`.
- The game host owns the fixed timestep and submits the same ordered `DrawCommand` output.
- Foundation owns actual renderer/audio devices and platform texture handles.

The browser animation loop is presentation scheduling only. The 28 ms logical simulation ticks are computed from the Foundation's monotonic browser clock and never tied to the requestAnimationFrame frequency. When the page becomes hidden, the web harness stops presenting; on visibility return it rebases the 28 ms accumulator and releases held keys so background time cannot become a burst of catch-up simulation ticks.

## Current browser limitations

- Canvas 2D is the only web renderer in this PR; WebGL is deferred.
- Keyboard input uses the same two-player bindings as native. Gamepad/touch support is a later adapter capability.
- WebAudio delegates final mixing to the browser. Clip IDs, volume, loop, stop, restart, and voice-limit behavior are preserved at the host contract; bit-exact S16 PCM output is not claimed.
- Custom character packages continue to use the existing host-supplied byte package API. This initial asset manifest covers the original fighter/stage/audio/data resources; generic custom-package rendering/registration remains a separate integration step.
- The browser must serve the manifest and resources via HTTP(S); CORS and browser codec policies apply.
