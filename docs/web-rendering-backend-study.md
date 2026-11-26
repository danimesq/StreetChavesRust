# Web rendering backend study: native SDL3 and browser Canvas 2D

## Scope

This document records the browser-rendering direction for StreetChavesRust. The game is one Rust codebase with platform-specific hosts supplied by RustMultiConsoleFoundation.

- Native builds use Foundation's SDL3 host.
- Browser builds target `wasm32-unknown-unknown` and use Foundation's Canvas 2D/WebAudio host.
- Game simulation, resource IDs, presentation commands and logical timing remain in StreetChavesRust.
- Browser URLs, DOM, Canvas objects and WebAudio nodes remain inside the Foundation browser adapter.

The browser build is a first-class published target, not an embedded plugin running inside another game editor.

## Selected architecture

```text
StreetChavesRust game core
          |
          +-------------------------+
          |                         |
          v                         v
Foundation native host       Foundation browser host
      SDL3                     Canvas 2D + WebAudio
          |                         |
          v                         v
    Desktop binary            WebAssembly package
                                     |
                                     v
                            Browser HTML entry page
```

The two hosts consume the same backend-neutral runtime contracts. The game must not fork its combat simulation or use renderer refresh rate as its logical clock.

## Web/WASM build and publishing

The browser module is built from the StreetChavesRust crate:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web --features web-foundation --out-dir web/pkg
```

The generated JavaScript glue and WASM module are inputs to Foundation's export library. The game build selects its resources and calls `WebExportBuilder` to stage the complete directory package and write the root `web-export.json` manifest.

The package has a suggested layout:

```text
web/
  index.html
  main.js
  web-export.json
  pkg/
  assets/
    images/
    audio/
    data/
```

See [WebAssembly host and exported assets](webassembly-host-and-assets.md) and [Foundation host contract](wasm-canvas-host-compatibility.md) for the builder API, resource ID conventions and deployment details.

## Rendering requirements

Both hosts consume ordered draw commands. The renderer must not infer draw order from object IDs or hash-map iteration. The game owns the presentation composition and submits the same semantic frame to each host.

The browser adapter must:

- Preserve command order and source rectangles.
- Support the existing game texture registration and image decoding path.
- Preserve logical dimensions while adapting the Canvas backing store to CSS size and device-pixel ratio.
- Use nearest-neighbor sampling for pixel-art presentation when requested.
- Surface missing resources and decode failures as diagnostics.
- Keep browser object handles out of game-core structures.

The native adapter must continue to provide the desktop behavior without importing browser or WASM APIs.

## Timing and lifecycle

StreetChavesRust's simulation clock remains independent from `requestAnimationFrame`. Browser frame callbacks only schedule presentation; the game advances every due 28 ms simulation tick using elapsed monotonic time.

When a browser page becomes hidden, the host should avoid accumulating an unbounded catch-up burst. On visibility return, the application rebases presentation scheduling and releases stale held-key state according to the documented lifecycle policy.

Audio contexts may begin suspended. The page must resume audio from a user gesture, such as the game's Start interaction.

## Capability boundaries

Current support:

- Native: SDL3 window, keyboard input, texture registration, audio decoding and PCM mixer through Foundation.
- Browser: Canvas 2D, keyboard normalization, image decoding and buffered WebAudio playback through Foundation.
- Shared: game-facing runtime traits, stable resource IDs and renderer-neutral presentation commands.

Deferred capabilities include WebGL, browser gamepad, pointer/touch input, worker/OffscreenCanvas hosting, ZIP/archive publishing, a command-line publisher, media transcoding, image optimization and streamed music.

## Validation plan

Use the same deterministic gameplay fixtures on native and browser hosts:

1. Verify the same simulation inputs and logical tick sequence.
2. Compare submitted draw-command order and resource IDs.
3. Confirm that source rectangles, pivots, flips and logical dimensions match.
4. Verify image/audio/data manifest loading and explicit fallback behavior.
5. Exercise missing files, malformed paths, decode failures and browser CORS failures.
6. Confirm audio resumes only after a valid user gesture.
7. Test resize, device-pixel ratio, hidden/visible transitions and focus loss.

A successful WASM compile does not prove visual parity or correct audio output. Browser smoke tests and representative screenshot/audio comparisons remain required.

## Decision

Use Foundation's Canvas 2D/WebAudio host for the current browser target. Keep the game core renderer-neutral, and keep publishing in the Foundation library so ImpossibleEngine and StreetChavesRust can use the same Web/WASM package contract.
