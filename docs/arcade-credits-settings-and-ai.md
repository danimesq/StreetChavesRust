#🌈 Arcade cabinet: credits, F1 configuration, F2 remapping and CPU

##🌈 Scope

This delivery adds the first connected arcade-cabinet layer to StreetChavesRust. The state machine owns credits, menu navigation, fighter entry, and the decision between a human P2 and a CPU opponent. The same Rust flow is used by the native SDL3 executable and the browser/WebAssembly runtime.

##🌈 Credits and player entry

- Key 5 or numpad 5 inserts one credit, capped at 99.
- Key 1 or numpad 1 spends one credit to enter P1.
- Key 2 or numpad 2 spends one credit to enter P2, including during an active fight.
- When P2 has not joined, the second slot is controlled by the CPU. A paid P2 join disables the CPU and gives the second slot to the human player.
- Credits are session state and start at zero, matching the original cabinet behavior. Preferences do not persist coin counts.

##🌈 F1 configuration

The screen exposes the twelve source options, with original value ranges and defaults:

| Option | Range / values | Currently consumed by the Rust runtime |
|---|---|---|
| Game level | 1–8 | Selects the CPU probability factor |
| Round time | 30 / 60 / 90 seconds / unlimited | Stored; round manager is not implemented yet |
| Power | Normal / P1 max / P2 max / both max | Initializes power stocks when a fight starts |
| Damage | 0.3–2.0, step 0.1 | Applied to melee damage resolution |
| Blood | On / Off | Stored; complete blood presentation is pending |
| Rounds | 1–3 | Stored; round-transition manager is pending |
| Language | Portuguese / Spanish | Stored; current screen text is still not fully localized |
| Sound test | Off / sound slot | Stored; preview playback is pending |
| Music track | Off / selected track | Stored; menu music selection is pending |
| Flash | On / Off | Stored; special-effect rendering is not yet gated by this setting |
| Show level | On / Off | Stored; complete fight HUD is pending |
| Show credits | On / Off | Applied to the current startup/menu credit counter |

Native preferences are saved to StreetChavesRust.settings.json in the working directory. The browser stores the same schema in localStorage under street-chaves-arcade-preferences-v1. The JSON has a schema version and validates numeric ranges and the uniqueness of the keyboard map.

##🌈 F2 player-one keyboard remapping

The remap screen captures eight physical keys in this order:

1. Up
2. Down
3. Left
4. Right
5. Weak punch
6. Weak kick
7. Strong punch
8. Strong kick

Duplicate assignments are rejected. The map is shared through Foundation's PhysicalKey and PlayerOneKeyMap contracts, so SDL3 and the browser KeyboardEvent.code adapter use the same mapping. P2's existing WASD and R/T/Y/U defaults, along with the separate special-key slots, remain unchanged.

##🌈 CPU opponent

When P2 does not spend a credit and join, the second fighter is driven by the already-ported Common.rot [IA] path: target-distance selection, approach movement and the source-ordered close-range probability chain. The F1 game level selects a provisional probability-factor curve. The exact Pascal Fator formula has not yet been recovered, so the curve is explicitly a first tuning choice rather than a claim of exact parity.

Character-specific [IA] scripts are not all connected to the live fight orchestrator. Projectile-threat detection and the campaign opponent sequence are also pending. The current CPU is therefore a functional first cabinet opponent, not full original-game AI parity.

##🌈 Current limits

The F1 menu persists all twelve options, but not every option has a gameplay subsystem to consume it yet. In particular, a full round clock/round manager, blood rendering, complete language localization, sound preview, menu music selection and complete fight HUD still need implementation. The attract/demo mode, CHS_ADVERSARIOS campaign and end-of-fight flow remain separate work.

##🌈 Validation

Regression tests cover original defaults, option bounds, preference round-tripping, duplicate-key rejection, credit caps and spending, P1/P2 entry, CPU selection, mid-fight P2 joining, F1 changes and F2 capture. Native/WebAssembly CI remains the final compiler gate for the integrated branch.
