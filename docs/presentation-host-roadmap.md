# Native and browser presentation host roadmap

## Current milestone: renderer-neutral snapshot

The combat core exposes `FightSimulation::presentation_frame()`. It returns a read-only snapshot containing tick, camera, each fighter's slot/character ID/sprite frame/action/action frame/facing/world position/health/power/invulnerability, and pending sound/screen flash/shake requests.

The API does not advance simulation, timers, RNG, nor mutate fighter state. The snapshot is deliberately platform-neutral and can be consumed by SDL, Canvas2D or another renderer.

## Audio host slice

The first native audio slice is documented in audio-backend-pr36.md. The Foundation consumer integration is documented in foundation-native-host-pr37.md: Foundation owns the SDL3 window, textures, WAV decoding/device/mixer; StreetChavesRust retains Pascal resource-bank indexing, the two-player input bridge and the 28 ms fixed-step simulation. SomPendente is consumed once after a simulation tick. The opt-in dependency is excluded from wasm32.

### Native SDL host

- Window and 400x300 logical canvas with resize/letterboxing.
- Asset search paths supplied by command line/environment and game-data folders.
- Sprite-strip/atlas manifest loader. C01-C15 and optional C16 are user-supplied files; runtime must not assume assets are redistributable.
- Frame extraction and source rectangle mapping based on audited original sprite-sheet metadata.
- Keyboard/gamepad mapping into normalized `InputFrame`.
- Fixed 28 ms simulation tick, independent from presentation frame rate.
- Draw stage, fighters, effects, HUD, health/power and camera.
- Audio backend for source sound IDs and optional custom-character sounds; event queue prevents replaying a held pending ID every render frame.
- Debug overlay: fighter ID, sprite/action frame, boxes, tick, missing assets and audio diagnostics.

### Web/WASM host

- Compile the same Rust simulation for `wasm32-unknown-unknown`.
- Canvas2D host draws the shared presentation snapshot.
- Host-owned asset loading; optional package folder/file selection hands bytes to the existing package parser.
- WebAudio sound queue, gated by the first user gesture.
- Browser lifecycle, resize, input, visibility and error reporting.
- No combat mechanics or command rules reimplemented in JS.

## Asset policy

The user's statement that Street Chaves was distributed as freeware is relevant context and should be recorded. Freeware status alone is not a file-level reuse license, however. Until the original creator's redistribution terms or explicit permission covering the extracted sprites/audio is documented, the repository will support local asset loading without committing extracted original-game files. The original asset source may be used as a local installation/reference path for testing, subject to the user's lawful access.

The Chapolin C16 manifest and runtime data already exist in PR #34. Its renderer needs a cleared/local sprite-sheet source and an audited frame-index mapping; it must not invent a new frame order.

## Output event design

Presentation output is non-authoritative. Sound, flash, shake and visual effect requests are host-consumed events. The renderer/audio backend must not mutate combat state, RNG, hitboxes or action timers.
