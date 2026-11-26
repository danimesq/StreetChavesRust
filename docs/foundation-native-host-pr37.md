# Foundation native host integration (PR #37)

## Ownership

Foundation owns SDL3 initialization, the native window, event queue, logical presentation, texture upload, WAV decoding, audio device and shared PCM mixer.

StreetChavesRust owns game semantics: Pascal resource-bank paths and indices, SomPendente one-shot handling, the source 28 ms simulation clock, two-player logical input sampling, and the mapping from P.Quadro values to audited sprite rectangles. The combat core remains renderer- and platform-independent.

## Native flow

NativeFightHost::new(RuntimeConfig) creates the Foundation SDL3 host. poll_and_tick translates both normalized player snapshots into the existing Rust InputSamplers, executes every due 28 ms simulation tick, and consumes pending Pascal sound once after each tick. render submits ordered DrawCommands to Foundation.

The generic Foundation Runtime helper is intentionally not used as the game scheduler: the current game orchestrator has two input frames and an evidence-based fixed 28 ms tick. The native host reuses the lower-level Foundation RuntimeHost implementation without changing game timing semantics.

## Sprite assets

P.Quadro is not an atlas address. The adapter does not guess a sheet cell, frame order, pivot, or sprite dimensions. Local BMP/PNG images are decoded and uploaded by Foundation; the game registers explicit SpriteFrameMapping entries from audited metadata. A debug silhouette is drawn for frames that have no mapping yet.

No extracted original sprites or WAV files are committed. The adapter reads the user's local game-data folder.

## Audio

FoundationAudioOutput preserves the Pascal bank layout and INI-based volume precedence while delegating WAV decode, clip storage, playback, stop and mixing to Foundation. Effects reserve their zero-based indices, music uses bank slot zero, and victory clips preserve fighter/variant order. SomPendente is consumed after the simulation tick, never from render.

## Build

    cargo test --all-targets --features native-foundation
    cargo check --target wasm32-unknown-unknown

The native feature is opt-in. The wasm32 target does not compile or link SDL3.
