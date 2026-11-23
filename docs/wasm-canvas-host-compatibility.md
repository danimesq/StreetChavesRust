# Reusable WASM Canvas host compatibility contract

## Status

This document defines the compatibility target for making StreetChavesRust a reference consumer of the generic ConstructWASMCanvas host. It is a design contract, not an implemented ABI or a claim of browser/Construct compatibility.

The goal is to let a WASM application written in Rust or another language run inside a dedicated browser Canvas managed by a generic host. Construct 2/3 is one host integration; a standalone HTML page or another web host should be able to implement the same contract.

## Boundary and ownership

Keep three layers separate:

1. **Game/application core:** simulation, application state, deterministic logic and renderer-neutral presentation data. No Construct SDK, DOM, Canvas, SDL or JavaScript object types.
2. **WASM browser adapter:** application-specific glue that connects the core to browser capabilities and the host contract. It may use a language-specific WASM toolchain.
3. **Generic host/plugin:** loads the application bundle, owns the DOM Canvas element and its placement, provides resource registration and lifecycle services, and forwards commands/events.

The host owns the Canvas element and its DOM lifecycle. The application adapter owns its internal simulation and may own the Canvas rendering context after mounting. Browser objects remain on the browser side; the game core sees stable resource IDs and platform-neutral metadata only.

SDL3 remains a separate native adapter. It must not be part of the browser host contract or a dependency of the WASM target.

## Host-facing JavaScript facade

A WASM application package should expose a small JavaScript facade, regardless of the language used to compile its core. The facade hides toolchain-specific glue (for example wasm-bindgen output) from the Construct plugin.

Conceptual interface:

- `mount(options)`: attach the application to the host-provided Canvas and host services.
- `initialize(config)`: initialize the application and report readiness.
- `send(command)`: deliver an application-defined command or normalized input.
- `registerResource(descriptor, source)`: register an external resource once.
- `releaseResource(resourceId)`: explicitly release a registered resource.
- `resize(viewport)`: update logical/output dimensions and pixel ratio.
- `setPaused(paused)`: pause or resume application progression.
- `dispose()`: stop work, detach listeners and release application-owned resources.

The facade may return Promises for asynchronous operations. It must expose structured readiness and failure results. The exact JavaScript class/function shape is intentionally left to the host API specification; the semantic operations and ownership rules above are the compatibility requirements.

## Versioning and negotiation

Every application package declares a host API compatibility range or exact supported major version. The host reports its API version before initialization.

- Major-version mismatch must fail explicitly.
- Additive optional fields may be introduced without changing the major version.
- Removing or changing the meaning of a required field requires a major version change.
- Unknown optional fields must be safely ignored.
- The application must not infer capabilities from Construct version strings; use explicit host capabilities.

Initialization should report the negotiated protocol version and supported capabilities. Do not silently continue with an incompatible host.

## Lifecycle state model

The host and application should expose observable states:

`created → loading → mounted → initializing → ready → disposed`

A failure may occur during loading, mounting or initialization and must include a stable error code plus diagnostic context. Paused/running is an orthogonal runtime state after readiness.

Commands that require readiness must either be rejected with an explicit not-ready error or queued under a documented bounded policy. Silent command loss is not acceptable. Disposal must be idempotent and safe while asynchronous resource decoding is pending.

## Canvas and viewport contract

The host creates and attaches a dedicated Canvas. It supports overlay and underlay composition relative to the Construct surface, but does not promise that CSS z-index alone works in every export target.

The host communicates a viewport descriptor containing at least:

- logical width and height;
- CSS pixel width and height;
- backing-store pixel width and height;
- device-pixel ratio;
- viewport origin/offset when relevant;
- visibility and layer mode.

The application must distinguish logical coordinates from backing-store pixels. Resizing must not silently reset application state. The host owns DOM placement, stacking, visibility and element cleanup; the application owns its rendering policy and must not modify Construct's own canvas/context.

Pointer and keyboard routing are host capabilities, not assumptions. A host must declare which input capabilities it provides and whether the custom Canvas receives pointer events.

## External resource registration

Resources are registered once and referenced by stable logical IDs. The host/browser adapter retains browser-side image objects in a private registry. The application core receives only IDs and platform-neutral metadata.

A minimal image descriptor includes:

- stable resource ID;
- resource kind;
- pixel width and height;
- optional content/version hash and MIME type.

Atlas rectangles, pivots, trim data and animation metadata are optional extensions. They should be added only when a consumer needs them.

Registration is asynchronous and reports ready or failed. Duplicate IDs, replacement, release, missing IDs, decode errors, CORS failures and disposal during decode must have explicit behavior. Resource replacement must be explicit; an ID must not silently start referring to unrelated content.

The host/plugin must use documented Construct SDK asset access where available. If a supported SDK path cannot provide an image source, accept an explicit URL or encoded byte payload. Never depend on undocumented private texture objects.

## Commands, events and diagnostics

Commands and events use structured payloads with stable names/IDs and explicit fields. The host transports application-defined messages without interpreting game-specific semantics.

The application may emit lifecycle events (ready, error, disposed), resource events (ready, failed) and application-defined semantic events. Construct adapters translate those events into the corresponding SDK triggers/expressions.

Diagnostics should include protocol version, application identifier/version, operation, resource ID when relevant, and a stable error code. Avoid logging full binary payloads or sensitive URL query strings.

## Rendering interoperability

The host contract does not mandate whether drawing commands are interpreted by JavaScript or whether the application adapter draws directly into the mounted Canvas. Both models are compatible if:

- the application owns the visual result on its dedicated surface;
- the host retains Canvas lifecycle/composition ownership;
- resource IDs resolve consistently;
- logical viewport semantics are preserved;
- rendering never mutates authoritative simulation state.

For StreetChavesRust, the intended browser adapter is Rust/WASM + Canvas 2D, while the native adapter is Rust + SDL3. Both consume the same renderer-neutral presentation contract. This is a consumer implementation choice, not a requirement imposed on every WASM application.

## Minimum interoperability fixture

Before claiming compatibility, provide a tiny non-game-specific WASM application that:

1. mounts into a host-created Canvas;
2. negotiates the host API version;
3. renders a visible test shape;
4. registers and draws one external image by stable resource ID;
5. receives one host command and emits one structured event;
6. responds to resize and pause/resume;
7. disposes cleanly, including pending resource work.

Run the same fixture in a standalone HTML harness and through each supported Construct adapter. Record exact Construct version, export mode, browser and limitations. C2 and C3 require independent validation.

## Relationship to ConstructWASMCanvas

ConstructWASMCanvas is the generic host/plugin implementation. StreetChavesRust is one consumer and a potential reference implementation of the Rust/WASM + Canvas 2D application side.

The host contract must remain independent of StreetChavesRust types, fighter IDs, combat commands and asset formats. Conversely, StreetChavesRust should not import Construct SDK code or depend on the plugin repository to build its native SDL3 target.

The companion host specification lives in the [ConstructWASMCanvas repository](https://github.com/Floflis/ConstructWASMCanvas). Keep the two documents aligned when the protocol is revised.
