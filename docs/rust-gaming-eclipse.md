
## 🌘 10. Gradually move host mechanics out, not game identity

The long-term goal is to make StreetChaves consume Foundation host services with minimal friction. This is a gradual adapter migration, not a wholesale extraction of everything visible or audible from the game.

| Keep in StreetChavesRust | Move behind Foundation host contracts |
|---|---|
| Combat, character/move rules, game flow and state | Window creation and host event loop |
| Fixed 28 ms logical simulation tick, RNG and deterministic replay | Physical keyboard/controller polling |
| AABB, hit/hurt-box rules and collision ordering | Physical-to-logical input normalization |
| Game-specific asset meaning and logical asset references | Platform-specific asset loading/conversion and resource lifecycle |
| Renderer-neutral visual intent: sprite identity, position, animation state, effects and draw order | Actual frame presentation, GPU/window resources and backend-specific rendering |
| Semantic audio events/requests | Audio device initialization, playback and backend-specific output |
| Mapping logical actions (jump, attack, movement) to game behavior | Capability reporting, diagnostics and platform service errors |

The game tells the host what it wants presented or played; the host decides how to perform that work on the selected target. Conversely, the host reports normalized logical input, while StreetChaves decides what that input means in its domain. SDL is one possible Foundation desktop backend, not a rival to Rapier and not a game-core dependency.

Do not delete or replace the current host implementation until Foundation has a real adapter for the required services and a minimal StreetChaves consumer builds and runs against it. The initial [foundation-runtime scaffold in Foundation PR #9](https://github.com/danimesq/RustMultiConsoleFoundation/pull/9) is not yet that adapter: it does not establish production SDL/window/input/render/audio support or validated target support. Keep the current path working while the new seam is implemented and tested.

## 🌘 11. Fixed simulation time is not host frame time

The Foundation host may provide frame timing, pacing, pause/resume notifications, and opportunities to advance the game. It must not silently redefine the authoritative game tick. StreetChaves retains its source-derived 28 ms fixed logical tick and update ordering. A variable delta from a host frame coordinator is input to an accumulator/pacing layer at most; it is not a replacement for deterministic fixed-step simulation. Tests and replay must continue to run headlessly without any Foundation window or event loop.

## 🌘 12. Hybrid migration into ImpossibleEngine

The future StreetChaves import path should be hybrid, not a promise to translate every Rust source file into Event Sheets. An importer and semantic-analysis layer can map verified project metadata, assets, data, and bounded behaviors into ImpossibleEngine's versioned project model and typed Event Sheet IR. Complex or unsupported behavior remains in native Rust modules behind explicit interfaces.

Each imported element should retain source provenance and a migration status: represented in typed IR, retained as a native module, or unsupported with a diagnostic. Do not silently approximate behavior or discard constructs. Preserve the original game as an independent reference implementation and use deterministic fixtures to compare the original behavior with each generated/hosted slice. Import is not lossless until every required semantic has a tested mapping.

StreetChavesRust remains independent: ImpossibleEngine may import a documented export/interchange representation or inspect source as an input, but it must not depend on the StreetChaves Cargo crate. The compiler owns the importer and Event Sheet semantics; the game owns its domain rules; Foundation owns reusable host/platform services.
