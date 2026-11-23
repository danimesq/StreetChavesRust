# Web rendering backend study: SDL3, Emscripten and Construct hosts

## Status

This document records the StreetChavesRust-side conclusions from the broader Construct 2/3 and Rust/WASM rendering study. It is an architecture and feasibility plan, not an implemented browser backend or benchmark report.

Companion deep study: [ConstructWASMCanvas rendering performance and Rust/WASM integration](https://github.com/Floflis/ConstructWASMCanvas/blob/main/docs/rendering-performance-and-rust-wasm.md).

## 1. The important distinction

An HTML Canvas element is a surface. It may host Canvas 2D or WebGL. SDL3 + Emscripten does not automatically turn SDL drawing into Canvas 2D calls.

There are two viable browser strategies:

### A. Keep SDL3 with Emscripten

Rust's `wasm32-unknown-emscripten` target can interoperate with Emscripten's SDL and JavaScript ecosystem.

SDL3's Emscripten backend maps its 2D renderer through GLES2 to WebGL. The result is a WebGL-backed HTML Canvas, not a Canvas 2D context. SDL3 exposes `SDL_HINT_EMSCRIPTEN_CANVAS_SELECTOR` to select the CSS selector for its target Canvas before creating the window.

Advantages:
- Less rewriting of existing SDL-facing platform/rendering code.
- SDL continues to provide its window, input, event, audio and renderer abstractions where supported.
- WebGL is a GPU-oriented route and avoids full CPU framebuffer uploads.

Constraints:
- Requires Emscripten and a compatible Rust/SDL3 build.
- Rust's target availability does not prove a specific SDL3 crate version/build script works.
- The browser main loop must yield; use SDL callbacks/Emscripten main loop and vsync/requestAnimationFrame.
- SDL's Emscripten guidance says rendering must occur on the main thread.
- Threaded WASM is a separate build/deployment profile requiring COOP/COEP.
- Browser audio may be gated on user interaction.
- Construct export packaging and lifecycle must be tested.

### B. Replace SDL at the browser boundary

Use `wasm32-unknown-unknown` with `wasm-bindgen`/web bindings and implement a browser presentation adapter that targets Canvas 2D or WebGL.

The shared game core should not call SDL directly. Instead, native SDL3 and browser rendering adapters implement the same platform/presentation contract.

Advantages:
- True browser-native Canvas 2D is possible.
- Clean separation between game logic and platform APIs.
- Browser host can own resource registration and renderer initialization.
- Renderer can be tailored to web scheduling, context lifecycle and device-pixel ratio.

Constraints:
- SDL is not compiled into this path; its APIs must be replaced at the platform boundary.
- A naive WASM-to-JavaScript call for every sprite/draw operation can become expensive.
- Canvas2D may be slower than WebGL for large sprite counts, overdraw or effects.
- Native and browser backends need shared conformance tests to prevent behavior drift.

### C. Avoid as the primary path: software framebuffer upload

A CPU RGBA framebuffer uploaded every frame through `ImageData`/`putImageData` is useful as a diagnostic/fallback, but it moves a full frame across the CPU/browser boundary repeatedly. Prefer GPU-side rendering for the normal game path.

## 2. Recommended engine boundary

The game core should produce renderer-neutral presentation data and remain independent of SDL, DOM, Canvas, WebGL and Construct SDK types.

Proposed direction:

```text
                StreetChavesRust core
           simulation / combat / deterministic RNG
                           |
              renderer-neutral presentation
                           |
              +------------+-------------+
              |                          |
        native adapter              browser adapter
          Rust + SDL3            Rust + WASM bindings
              |                          |
        SDL3 renderer             Canvas2D or WebGL
```

The native SDL3 adapter and browser adapter may differ internally, but they must consume the same logical scene state and preserve the same simulation tick/replay semantics.

Do not implement a hidden shim that intercepts arbitrary SDL drawing calls and translates each one to Canvas2D. That duplicates renderer semantics and creates a chatty boundary.

## 3. Feasibility spike before choosing a default

Build two tiny browser prototypes using the same deterministic scene:

1. Rust + SDL3 compiled for `wasm32-unknown-emscripten`, rendering through SDL3's WebGL backend.
2. Rust compiled for `wasm32-unknown-unknown`, rendering through a direct browser Canvas2D adapter.

The spike should test:
- toolchain and dependency compatibility;
- canvas selector / host-supplied canvas ownership;
- initialization, resize, visibility, pause/resume and disposal;
- input and browser audio restrictions;
- resource loading and stable resource IDs;
- frame pacing and context loss;
- integration in standalone HTML before Construct;
- performance at multiple sprite counts and transparency/overdraw levels.

Measure frame time, FPS, CPU/main-thread time, GPU time where available, memory, startup/compile time, asset decode/upload and input latency. Do not select a renderer based on API preference alone.

## 4. Construct compatibility constraints

- Construct 2's in-renderer plugin API has separate `draw(ctx)` and `drawGL(glw)` callbacks. The latter receives a Construct GL wrapper, not ownership of the renderer's raw context. A separate DOM Canvas is a different integration model and needs export-level validation.
- Construct 3's Addon SDK v2 supports HTML layers for custom HTML/Canvas content. Its runtime may run in a worker with OffscreenCanvas, while DOM access belongs on the main thread through DOM-side scripts and asynchronous messaging.
- SDL's Emscripten documentation requires rendering on the browser main thread. Therefore the first SDL3/Emscripten + Construct 3 prototype should keep the SDL window/context/rendering on the DOM/main-thread side. Do not assume the C3 runtime worker is a valid SDL rendering thread.
- Never reuse Construct's internal WebGL/WebGPU context for StreetChavesRust. Use a separate host-owned Canvas surface.

## 5. Recommended staged decision

1. Preserve and test the platform-neutral engine boundary.
2. Run the SDL3/Emscripten WebGL feasibility spike.
3. Run the direct Rust Canvas2D feasibility spike.
4. Compare both using identical fixtures on desktop and mobile.
5. Choose a default web backend based on measured performance and integration reliability.
6. Keep ConstructWASMCanvas as an optional generic host; StreetChavesRust must also run in a standalone HTML harness.

## 6. References

- [Rust wasm32-unknown-emscripten target](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-emscripten.html)
- [SDL3 Emscripten guide](https://wiki.libsdl.org/SDL3/README-emscripten)
- [SDL3 Canvas selector hint](https://wiki.libsdl.org/SDL3/SDL_HINT_EMSCRIPTEN_CANVAS_SELECTOR)
- [Construct 2 renderer and plugin drawing hooks](https://www.construct.net/en/construct-2/manuals/construct-2-javascript-sdk/sdk-documentation/runtime-functions)
- [Construct 3 Addon SDK worker/DOM split](https://www.construct.net/en/make-games/manuals/addon-sdk/guide/runtime-scripts)
- [Construct 3 Addon SDK v2 HTML layers](https://www.construct.net/en/forum/construct-3/plugin-sdk-10/addon-sdk-v2-182122)
- [Construct 3 performance tips](https://www.construct.net/en/make-games/manuals/construct-3/tips-and-guides/performance-tips)

## 7. Unresolved questions

- Which Rust SDL3 crate/build setup links cleanly with the current Emscripten SDL3 port?
- Can the Emscripten SDL3 output target a host-created Canvas inserted by Construct 3's HTML-layer APIs?
- Does Construct 2's exported/minified runtime preserve the selected Emscripten loader/glue?
- Which route is faster for the same game scene on target devices?
- What are the actual memory and startup trade-offs of both routes?

These are feasibility gates, not assumptions.
