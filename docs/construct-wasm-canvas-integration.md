# Construct 2/3 integration: embedded WASM + Canvas

## Status and intent

This document extends the shared-engine/rendering-backend proposal with a future embedding target: exported Construct 2 and Construct 3 projects.

The exported game's HTML will load the Rust engine compiled to WebAssembly and create a dedicated Canvas surface for its rendering. Construct remains the surrounding game host and orchestrator. Construct events configure the engine, provide graphical resources, start and control fights, and react to semantic gameplay events. The engine simulates and renders its own fight presentation into its Canvas, which can be composed above or below the Construct runtime's canvas.

This is a design proposal. It does not claim that the plugin, WASM API, asset bridge, or runtime integration already exists.

## Intended composition

Conceptually, the exported page contains two independently rendered surfaces:

    +-------------------------------------------------------+
    | Construct project                                     |
    |                                                       |
    |   Construct canvas                                    |
    |   - menus, HUD, scene objects, interface, effects     |
    |                                                       |
    |   StreetChavesRust canvas                             |
    |   - fighters, stage, hit effects, fight presentation  |
    |                                                       |
    +-------------------------------------------------------+

The surfaces may be layered in either order:

- **Overlay:** the engine Canvas is above the Construct Canvas. Transparent regions reveal the Construct scene underneath.
- **Underlay:** the engine Canvas is below the Construct Canvas. The Construct scene can place HUD, menus, foreground objects, and interface elements over the fight background.

The integration must not assume that changing CSS z-index alone is sufficient in every Construct export or browser wrapper. The host adapter must locate the actual runtime canvas/container, establish a predictable DOM stacking context, and test the target export modes.

For correct composition, define and test:

- Canvas element creation and attachment point.
- Overlay/underlay order and transparent clear behavior.
- CSS position, dimensions, scale, and alignment with the Construct viewport.
- Logical resolution, device-pixel ratio, resize, fullscreen, and orientation changes.
- Pointer and keyboard event routing, including whether the engine Canvas should receive pointer events.
- Visibility, pause, scene transitions, focus loss, and destruction/recreation.
- Behavior when the Construct runtime changes its own canvas size or container.

The engine Canvas is a separate presentation surface. It must not assume ownership of, or draw directly into, Construct's internal rendering context.

## Responsibility boundaries

### Construct project

Construct owns the surrounding game experience:

- Event Sheets and game flow.
- Scene transitions, menus, HUD, UI and non-fight scene objects.
- Choosing the fight mode and supplying fighter/move/stage configuration.
- Deciding when the engine Canvas is shown, hidden, layered, paused, or reset.
- Providing graphical assets or references to assets that the integration is allowed to access.
- Reacting to semantic events emitted by the engine.

The Construct project describes what fight to run; it does not reimplement the fight rules.

### ConstructWasmCanvas plugin

A future generic plugin, tentatively named `ConstructWasmCanvas`, can provide the integration surface for Construct 2 and Construct 3.

Its generic responsibilities should be limited to:

- Loading and initializing a compatible WASM module.
- Creating, attaching, sizing, and disposing the custom Canvas.
- Selecting overlay or underlay composition.
- Exposing Construct Actions, Conditions, and Expressions (or the corresponding C3 plugin interfaces).
- Sending configuration, commands, input, and resource registrations to the WASM bridge.
- Receiving and dispatching asynchronous readiness, error, and semantic gameplay events.
- Managing asset transfer and resource lifetime at the browser boundary.
- Reporting initialization, loading, and runtime errors to the Construct project.

The plugin must not contain StreetChaves-specific combat logic, fighter rules, hitbox calculations, or animation decisions. It should be reusable by other WASM + Canvas use cases.

Construct 2 and Construct 3 have different SDK generations and runtime details. Prefer a shared conceptual contract and shared implementation modules where practical, with thin version-specific plugin adapters. Do not assume one plugin package can be copied unchanged between both SDKs.

Construct 2 is a legacy product, so browser/export compatibility must be verified against the actual supported export targets. A successful desktop browser test does not automatically prove compatibility with every wrapper, mobile export, or embedded webview.

### StreetChavesRust WASM module

The Rust engine owns authoritative fight simulation and fight-specific presentation:

- Fighter state, moves, hitboxes, hurtboxes, collision, physics, AI and combat timing.
- Logical tick progression and deterministic RNG.
- Fight animation selection and frame progression.
- Stage/fighter/effect composition and draw order.
- Resource identifiers and the mapping from logical fight assets to presentation commands.
- Semantic events such as hit, combo change, round finish, and fight finish.

The browser-specific adapter owns Canvas and browser resource handles. The platform-neutral game core must not depend on Construct SDK types, DOM types, Canvas contexts, or JavaScript object handles.

## Integration contract: commands, events, and state

Keep the Construct-facing API small, explicit, versioned, and independent from the plugin's internal implementation.

A useful conceptual command surface includes:

- Initialize the engine and select a rendering surface.
- Register, update, and release graphical resources.
- Configure fighters, movesets, stage, rules, and match parameters.
- Start, pause, resume, reset, and stop a fight.
- Submit player inputs or control changes.
- Request a state snapshot or query supported fighter values.
- Resize or change the presentation viewport.
- Dispose the engine instance.

The engine should emit asynchronous notifications such as:

- Engine ready / initialization failed.
- Resource loading / resource ready / resource failed.
- Fight ready / fight started / fight paused.
- Fighter hit / damage applied / combo changed.
- Round started / round finished.
- Fight finished, with a structured result.
- Runtime error or unsupported configuration.

The plugin can expose these as Construct triggers and expressions. For example, an Event Sheet can react to an engine hit event to play a sound, update a combo counter, or trigger a separate screen effect.

Prefer event payloads with stable IDs and explicit fields over opaque strings. Avoid requiring Construct to poll many individual WASM properties every frame. Batch input updates and state snapshots where practical to reduce JavaScript/WASM boundary overhead.

The API must include an explicit readiness state. Construct events may run before the WASM module or its assets have finished loading; commands that require readiness must either be queued under a documented policy or rejected with a clear error. Do not silently drop commands.

## Supplying sprites and other graphical resources from Construct

A key requirement is that an external host component (initially the Construct plugin/runtime) can provide the graphics used by the Rust fight engine.

The Rust core must not assume that assets are available through native filesystem paths. In an exported HTML project, the asset source may be a Construct-managed resource, a URL, a Blob, an ArrayBuffer, a typed byte array, or another browser-supported source. The plugin should normalize supported sources at the browser boundary.

### Separate resource payloads from resource identity

Use a stable logical resource ID to identify an image or atlas after registration. Fight configuration refers to IDs, not JavaScript object references or browser URLs.

A resource descriptor should be able to represent, as required:

- Stable resource ID and optional content/version hash.
- Resource kind (single image, sprite sheet/atlas, or another explicitly supported type).
- Pixel dimensions and optional pixel-density scale.
- Pixel format or encoded image format where relevant.
- Atlas frame rectangles.
- Per-frame pivot/origin and trim metadata, if the source format needs them.
- Animation/frame metadata, including frame order and timing when the engine is expected to use it.
- Optional logical names/tags for debugging and content mapping.

Do not make every field mandatory for a simple image. Define a minimal image registration path first and add atlas/animation metadata only when the source data requires it.

### Recommended registration flow

1. Construct or the plugin identifies the selected sprite/resource and obtains a supported source representation.
2. The plugin sends a resource descriptor and the image payload (or a supported source handle) to the browser-side WASM adapter.
3. The adapter validates the descriptor, decodes the image if necessary, and registers a browser-side image resource.
4. The engine receives a stable resource ID and the metadata it needs to select frames and build draw commands.
5. The engine references that ID in its presentation output. The renderer resolves it to the registered browser image and draws it.
6. The adapter reports readiness or a structured failure to Construct.
7. The resource remains registered until explicitly released or the engine instance is disposed.

The exact Construct SDK mechanism for obtaining an image's bytes or source must be verified during plugin implementation. Do not rely on undocumented access to Construct's private texture objects. If a supported SDK does not expose the needed source, use an explicit plugin-facing asset input (for example, a URL or byte payload) rather than depending on runtime internals.

### Keep browser image objects out of the game core

There are two reasonable transport strategies:

**A. Transfer encoded bytes once, decode in the browser adapter (recommended baseline).**

The plugin transfers the encoded image bytes and descriptor to the WASM/browser boundary. The browser adapter decodes the image using browser facilities and stores the resulting image object in a private resource registry. The engine core receives only the stable resource ID and platform-neutral metadata.

**B. Register a browser-side image handle.**

If the Construct SDK exposes a supported browser image source, the plugin can register that source directly with the browser adapter. The adapter retains the corresponding JavaScript/browser object in its private registry and assigns it a stable resource ID. The core still receives only the ID and metadata.

Both approaches preserve the same core boundary. A browser image object, CanvasImageSource, ImageBitmap, HTMLImageElement, Blob, JavaScript object, or WASM pointer must not become part of the platform-neutral game API.

If the Web adapter is implemented in Rust with `wasm-bindgen`/`web-sys`, it may privately retain browser objects (for example as adapter-owned JS values) and resolve resource IDs when issuing Canvas draw calls. That is an adapter detail, not a type exposed by the simulation core.

### Do not resend images every frame

Image payloads should be transferred and decoded during registration, not repeatedly during rendering.

Per-frame communication should normally contain compact information such as:

- Resource ID.
- Frame/atlas rectangle or frame index.
- Destination position and size.
- Flip/rotation/opacity/layer information as supported by the renderer.

Do not serialize full images as base64 strings on every tick or send complete pixel buffers repeatedly. Avoid unnecessary copies across JavaScript/WASM memory. If profiling later demonstrates that asset transfer is a bottleneck, evaluate typed-array views, ownership-aware buffers, or other optimized transfer paths without weakening the ownership contract.

### Asset readiness, errors, and lifetime

Image loading and decoding are asynchronous. A fight must not begin rendering a resource that has not reached the documented ready state unless the engine explicitly supports placeholders.

Define behavior for:

- Missing resource IDs.
- Decode failures and unsupported formats.
- Invalid dimensions or atlas rectangles.
- Duplicate IDs and replacement/versioning.
- Resource release while a fight still references the resource.
- Engine disposal while image decoding is pending.
- Cross-origin URLs and browser CORS restrictions.
- Context loss or browser lifecycle events where applicable.

Resource replacement should be explicit and safe. A resource ID should not silently begin referring to unrelated image content halfway through a fight. Consider immutable registration plus explicit replacement/version IDs.

The plugin should surface the failed resource ID and useful diagnostic context to Construct. It must not silently substitute unrelated sprites.

### Asset provenance

The engine's ability to consume external assets does not grant permission to redistribute them. Petmals or another Construct project may supply its own sprites, while StreetChavesRust remains responsible for documenting the provenance and license status of any assets it bundles or redistributes.

## Example: Petmals Pink&Dark SyncFight

SyncFight is a motivating use case, not a hard-coded dependency of the generic plugin.

Petmals can select the fighters and provide the corresponding sprite sets, movesets, stage, and match rules. The Construct project starts the fight and reacts to events; StreetChavesRust performs the fight simulation and draws the fight through its own Canvas.

Illustrative configuration only (not an implemented schema):

    {
      "mode": "sync_fight",
      "fighters": [
        {
          "id": "player_mon",
          "sprite_set": "petmals/player_mon",
          "moveset": "petmals/player_moves"
        },
        {
          "id": "opponent_mon",
          "sprite_set": "petmals/opponent_mon",
          "moveset": "petmals/opponent_moves"
        }
      ],
      "stage": "petmals/arena_01",
      "rules": {
        "rounds": 3,
        "time_limit": 99
      }
    }

The sprite-set and moveset IDs are resolved through registered resources/data supplied by the host. The Construct project should not need to know the engine's internal hitbox representation or combat tick implementation.

A possible event flow:

- Construct selects the SyncFight configuration and registers its required assets.
- The plugin waits for WASM initialization and resource readiness.
- Construct starts the fight.
- StreetChavesRust advances the fight and renders fighters, stage, and fight effects into its Canvas.
- The engine emits a hit event; Construct plays a sound or triggers a separate UI effect.
- The engine emits a combo update; Construct updates its HUD.
- The engine emits a round/fight result; Construct advances the game flow.

The Construct project may hide its own duplicate fighter sprites while the fight Canvas is visible, but remains free to render HUD, menus, overlays, and other scene elements.

## Relationship to the rendering-backend contract

Keep two contracts distinct:

1. **Rendering contract:** shared Rust presentation output is consumed by SDL3 or the browser Canvas backend.
2. **External-host integration contract:** Construct sends configuration, inputs, resource registrations, and lifecycle commands to the WASM-hosted engine, then receives events and state.

The Construct plugin is not another renderer for the Rust engine. It hosts and controls the WASM + Canvas subsystem inside the exported game page. The browser Canvas backend remains the renderer.

This separation also allows the same WASM/browser integration to be hosted by a standalone HTML page or another compatible web host, without duplicating combat logic.

## Suggested implementation sequence

1. Complete the renderer-neutral presentation contract and native SDL3 adapter.
2. Make the standalone browser WASM + Canvas diagnostic render the same fixture.
3. Define and test the browser resource registry and one-time external image registration path, initially with a small permitted test image.
4. Define a versioned, minimal WASM host API for initialization, resource registration, fight configuration, input, events, and disposal.
5. Build a standalone HTML embedding harness that creates the Canvas and supplies test assets through the host API.
6. Prototype the Construct 2 plugin adapter against an actual exported project. Verify asset access through documented SDK surfaces and verify overlay/underlay behavior.
7. Add a Construct 3 adapter only after the shared contract is stable, accounting for its distinct SDK/runtime.
8. Integrate the Petmals Pink&Dark SyncFight configuration and representative sprite/moveset data.

Do not begin by coupling StreetChavesRust to Construct-specific types. First prove the browser host API and asset-registration contract independently of Construct. Then make the plugin a host adapter for that contract.

## Open decisions

- Exact Construct 2 and Construct 3 SDK APIs for obtaining or supplying image data.
- Whether the first plugin release accepts URLs, encoded bytes, SDK-provided image sources, or a deliberately limited subset.
- Exact resource descriptor and atlas/animation metadata schema.
- Whether Canvas drawing is issued directly by the Rust Web adapter or by a thin JavaScript Canvas host consuming Rust draw commands. Either choice must preserve the same platform-neutral core and resource-ID contract.
- Exact WASM binding/build tool and compatibility with the targeted Construct export browsers.
- Whether one plugin instance owns one engine/Canvas or supports multiple independent instances.
- Event delivery and command queue policy before engine/resource readiness.
- Browser audio ownership and whether fight audio is handled by Construct or a later engine adapter.

These decisions require a working proof of concept and SDK verification. Do not infer undocumented Construct internals or promise compatibility with every export target before testing.

## Non-goals

- Reimplementing StreetChavesRust gameplay in JavaScript.
- Making Construct's renderer draw every StreetChavesRust fight frame.
- Making the generic Construct plugin understand Petmals-specific combat rules.
- Passing Construct SDK objects into the platform-neutral Rust core.
- Transferring full image payloads every frame.
- Requiring Construct 2 and Construct 3 to share identical plugin binaries.
- Treating arbitrary external assets as licensed for redistribution.
