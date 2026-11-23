# Rendering backends and WebAssembly/Canvas architecture

## Status and scope

This document proposes an architecture for separating the StreetChavesRust game engine from its presentation backends. It describes a future implementation, not existing functionality: the repository currently has foundational Rust contracts, but no playable runtime, SDL3 frontend, or Canvas frontend.

The target is one Rust game implementation with multiple host/rendering adapters:

- Native desktop: Rust simulation + SDL3 platform/presentation adapter.
- Web: the same Rust simulation compiled to WebAssembly + browser Canvas 2D presentation adapter.

The browser build is not a second game port. Gameplay rules must not be rewritten in JavaScript or maintained separately.

## Architectural rule

The dependency direction is one-way:

```text
                 Native executable                  Browser application
              +---------------------+             +----------------------+
              | SDL3 host adapter    |             | Browser host adapter |
              | window / input /    |             | DOM / input / clock  |
              | clock / presentation|             | Canvas / lifecycle   |
              +----------+----------+             +-----------+----------+
                         |                                    |
                         v                                    v
              +---------------------+             +----------------------+
              | SDL3 renderer       |             | Canvas 2D renderer   |
              | textures / scaling  |             | canvas / ImageBitmap |
              +----------+----------+             +-----------+----------+
                         |                                    |
                         +------------------+-----------------+
                                            |
                                            v
                              +---------------------------+
                              | Renderer-neutral interface|
                              | draw list / frame snapshot |
                              +-------------+-------------+
                                            |
                                            v
                              +---------------------------+
                              | Rust game engine          |
                              | simulation / fighters /   |
                              | combat / physics / RNG /  |
                              | timers / replay           |
                              +---------------------------+
```

The diagram is conceptual. The implementation may use a single native host crate and a browser host crate, while sharing the same renderer-neutral types and game-core library.

The core must not depend on SDL, `web-sys`, `wasm-bindgen`, JavaScript objects, DOM types, Canvas contexts, window handles, audio devices, or browser/native wall-clock APIs.

## Responsibilities and boundaries

### 1. Game engine (shared Rust code)

Owns all authoritative game behavior:

- Logical game state and the fixed logical tick.
- Fighter state, commands, animation progression and AI.
- Collision, hit resolution, physics, effects and stage behavior.
- Deterministic RNG state and draw order.
- Input interpretation after physical inputs have been normalized into logical actions.
- Replay recording/playback and deterministic state snapshots.
- Semantic events and renderable state derived from the simulation.

The engine must remain headless and compile for native targets and `wasm32-unknown-unknown`. It must not conditionally change game rules to accommodate a renderer.

A browser build must execute the same simulation code and tests as a native build. Target-specific code belongs in the host/backend layer, not in combat rules.

### 2. Renderer-neutral presentation contract

The engine exposes a read-only presentation boundary. Prefer an immutable frame snapshot or an ordered draw-command list over direct drawing calls.

A first contract can describe operations such as:

- Clear/background and stage layers.
- Sprite/frame selection, source rectangle and destination rectangle.
- Horizontal/vertical flip where required.
- Ordered effect and projectile sprites.
- HUD elements, text and debug primitives.
- Explicit resource identifiers rather than SDL texture pointers or browser image objects.

The contract should preserve draw order. If the original game relies on a particular ordering for fighters, effects, stage objects or HUD layers, that ordering must be represented explicitly rather than reconstructed independently by each backend.

The renderer consumes presentation data; it must not mutate fighter state, advance timers, consume gameplay RNG, resolve collisions, or spawn simulation effects.

A useful initial logical coordinate space is the documented 400×300 game area. Scaling, letterboxing, device-pixel ratio and output resolution are presentation concerns. Do not multiply gameplay coordinates by a renderer-specific scale.

The first API should be intentionally small. Do not design a general-purpose scene graph or GPU abstraction before the game needs one.

### 3. SDL3 backend (native)

The SDL3 adapter owns native platform integration:

- Window creation, resize, fullscreen and focus lifecycle.
- Keyboard/controller events translated into logical input.
- Native timing and fixed-tick scheduling.
- Renderer resource creation and lifetime.
- Mapping renderer-neutral commands to SDL3 textures, geometry and text.
- Presentation, screenshots and optional debug overlays.

SDL-specific types remain inside the SDL adapter. No SDL handle or texture crosses into the engine or shared presentation contract.

SDL3 is a native dependency, not a dependency of the WebAssembly target. Cargo target-specific dependencies and/or separate host crates must ensure that browser builds do not attempt to compile or link SDL3.

### 4. Canvas 2D backend (WebAssembly)

The browser adapter hosts the same Rust engine compiled to WebAssembly. It owns:

- Browser lifecycle and canvas lookup/initialization.
- Canvas sizing, resize observation and device-pixel-ratio handling.
- Browser keyboard/gamepad events translated into the same logical input representation.
- `requestAnimationFrame` presentation scheduling.
- Browser image/resource loading and mapping resource IDs to browser-side image objects.
- Mapping the shared draw list to Canvas 2D operations.
- Browser-specific error reporting and lifecycle handling.

A Rust implementation can use `wasm-bindgen` and `web-sys` for the browser boundary and Canvas 2D API. Those dependencies belong only to the Web adapter. The engine and shared presentation contract should not import them.

Canvas 2D is the recommended first web backend. It is sufficient for a 2D fighting game and avoids making WebGL/WebGPU a prerequisite. A later GPU renderer can implement the same presentation contract if profiling demonstrates a real need.

The Canvas backend should use the browser's image primitives (for example, `CanvasImageSource`/decoded image resources) without exposing those objects to the engine. Asset loading is asynchronous in browsers, so the host needs a loading/readiness state and explicit diagnostics for missing resources. It must not silently substitute arbitrary assets.

## Shared timing and deterministic simulation

The game logic uses logical ticks, not display frames.

The current research identifies a default logical interval of 28 ms. Preserve that as a simulation rule only where supported by the evidence and existing contracts. Both hosts may present at different refresh rates, but they must advance the same number of logical ticks for the same input/tick sequence.

Recommended host loop:

1. Read elapsed host time.
2. Add it to an accumulator.
3. While the accumulator contains a complete logical tick, collect/consume the logical input for that tick and advance the engine exactly once.
4. Build or obtain a read-only presentation snapshot.
5. Render at the host's available presentation cadence.
6. Keep any interpolation strictly visual; never feed interpolated values back into simulation state.

On desktop, elapsed time comes from the native host. In the browser, `requestAnimationFrame` supplies presentation callbacks and a timestamp. Neither clock is allowed to redefine the game's logical tick.

For background-tab throttling and long pauses, define an explicit policy: pause, cap catch-up work, or ask the player to resume. Do not run an unbounded number of catch-up ticks in one browser callback. The policy must not alter deterministic replay behavior.

RNG consumption remains owned by the simulation. A renderer must never consume the gameplay RNG for particles, cosmetic variation or draw ordering. If cosmetic randomness is required, give it an independent presentation-only source whose state cannot affect gameplay or replay parity.

## Asset and resource boundary

Use stable logical resource IDs in shared game/presentation data. Keep platform handles private:

- Native resource ID → SDL texture/surface/font handle.
- Browser resource ID → decoded image/canvas resource.

The resource catalog and asset metadata should be platform-neutral. Decoding, upload, caching, lifetime, and failure reporting are backend responsibilities.

Do not assume that a native filesystem path can be opened in a browser. A Web package needs an explicit asset delivery strategy (for example, files served alongside the application or a generated asset bundle). The exact packaging strategy remains an open decision and must respect the project's asset provenance and redistribution rules.

Avoid embedding copyrighted game assets into the WASM binary by default. The browser build should consume only assets the user is legally entitled to provide or that the project is permitted to redistribute.

## Suggested Rust layout

Keep the current minimal crate until real boundaries justify splitting it. A possible later layout is:

```text
crates/
  chaves-core/             # deterministic simulation and game rules
  chaves-presentation/      # resource IDs, snapshots and draw commands
  chaves-platform-sdl3/     # native host, input, clock and SDL3 renderer
  chaves-platform-web/      # wasm host, browser input, clock and Canvas 2D
```

This is a destination, not a requirement to create four crates immediately. If the codebase remains small, modules within a workspace may be enough. The important property is dependency direction and the absence of platform types in the core, not the number of Cargo packages.

Possible dependency shape:

- `chaves-core` may depend on shared domain/presentation types.
- `chaves-platform-sdl3` depends on the core, shared presentation contract and SDL3 bindings.
- `chaves-platform-web` depends on the core, shared presentation contract, `wasm-bindgen` and `web-sys`.
- The shared core must not depend on either platform adapter.

If presentation types are kept inside the core initially, they should still be platform-neutral and separable later without changing game rules.

## Cargo and build strategy

Use target-specific dependencies or separate adapter crates so that SDL3 is never built for `wasm32-unknown-unknown`. Do not solve this by scattering browser/native conditionals through gameplay modules.

Illustrative target split (exact crate versions and features must be selected and tested when implementation begins):

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
# SDL3 bindings belong to the native adapter, not the core.

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "..."
web-sys = { version = "...", features = [
  "Window",
  "Document",
  "HtmlCanvasElement",
  "CanvasRenderingContext2d",
  "ImageData",
] }
```

This is an architectural illustration, not a copy-ready manifest. Prefer putting these dependencies in their respective adapter crates rather than the root package. Pin compatible versions and enable only the browser APIs actually used.

A future CI matrix should at least cover:

- Native `cargo fmt --all -- --check`.
- Native `cargo test --all-targets` for the shared Rust code.
- `cargo check --target wasm32-unknown-unknown` for the Web adapter and shared code.
- A browser smoke/integration test that loads the WASM module, initializes a canvas, advances a deterministic short fight fixture and verifies that drawing is issued.
- A native headless test proving the engine can run without initializing SDL or opening a window.

The exact packaging tool (`wasm-pack`, `trunk`, or a custom build) is intentionally undecided. Choose it after the browser entry point, asset delivery and deployment requirements are known.

## Testing and parity strategy

Keep simulation tests and rendering tests separate.

1. **Core tests:** same initial state, seed and logical inputs produce the same state snapshots on native and WASM.
2. **Replay tests:** a recorded input sequence produces the same logical state/tick hashes in both targets.
3. **Presentation-contract tests:** command ordering, source/destination rectangles, flips, resource IDs and layer order are stable.
4. **Backend tests:** each backend maps the same presentation fixture to its platform API.
5. **Visual tests:** compare screenshots/golden images separately for SDL and Canvas. Pixel identity is not guaranteed because text rasterization, filtering and browser implementations can differ.
6. **Browser lifecycle tests:** resize, device-pixel ratio, focus loss, tab suspension, resource-loading failure and input release on blur.
7. **No-platform-leak tests:** core dependency graph and public APIs must not expose SDL, DOM or Canvas types.

A renderer change must not invalidate gameplay tests. A browser rendering discrepancy must not be “fixed” by changing simulation state or timing.

## Implementation sequence

### Stage A — Formalize the boundary

- Audit the current Rust types and identify which data is simulation state versus presentation output.
- Define stable resource IDs and the smallest ordered draw-command/snapshot contract.
- Add tests proving that producing a snapshot does not mutate simulation state or consume gameplay RNG.
- Keep the existing headless build and tests working.

**Exit gate:** a deterministic fixture can produce a complete renderer-neutral presentation snapshot without SDL or browser APIs.

### Stage B — Native renderer adapter

- Implement the native host loop and SDL3 renderer against the shared contract.
- Keep SDL input mapping and resource handles inside the adapter.
- Add a diagnostic scene and a small evidence-backed fight presentation.
- Verify that rendering at different presentation rates does not change logical tick results.

**Exit gate:** the native adapter renders snapshots while the engine remains independently testable.

### Stage C — WASM and Canvas proof of concept

- Add the `wasm32-unknown-unknown` target and a browser-only adapter.
- Export a narrow initialization/input/update interface from Rust.
- Obtain a Canvas 2D context in the browser adapter and render the same diagnostic draw-command fixture used by the native adapter.
- Load a small permitted test asset and verify resize/letterboxing.
- Run the same deterministic simulation fixture in native and WASM builds.

**Exit gate:** the browser displays a diagnostic scene from the shared Rust presentation contract, with no duplicate gameplay implementation and no SDL dependency in the WASM build.

### Stage D — Fight rendering and browser hardening

- Connect fighter, stage, HUD and effect resources to the shared draw contract.
- Implement browser input and lifecycle handling.
- Compare representative fight snapshots across both backends.
- Add browser smoke tests and document build/deployment instructions.

**Exit gate:** a representative fight can be simulated by the shared engine and displayed by both backends; remaining differences are documented as presentation limitations, not hidden gameplay forks.

## Open decisions

- Whether presentation types live in `chaves-core` initially or in a separate `chaves-presentation` crate.
- Which browser build tool and deployment layout to use.
- Whether the Web target initially supports keyboard only or also browser Gamepad API.
- How fonts and text are rasterized consistently enough across SDL and browsers.
- How asset bundles are delivered without redistributing unlicensed original assets.
- Whether browser audio is in the first Web milestone or a later independent adapter.
- Whether a later GPU backend is justified by profiling; Canvas 2D remains the baseline.

These decisions must be made from implementation evidence. Do not add WebGL/WebGPU, a JavaScript gameplay engine, or a large cross-platform framework merely to make the first browser proof of concept work.

## Non-goals

- Reimplementing combat or fighter logic in JavaScript.
- Maintaining separate native and browser game rules.
- Importing SDL types into shared code.
- Letting render cadence control simulation cadence.
- Using Canvas APIs inside the game core.
- Requiring WebGL/WebGPU for the first browser build.
- Claiming full game parity or a playable Web version before the corresponding systems and assets exist.
