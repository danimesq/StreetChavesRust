# CHS screen and game-flow implementation status

This document tracks StreetChavesRust against the original state's responsibilities in the OpenStreetChaves source. Statuses describe the Rust reconstruction, not the completeness of the source project.

## Screen matrix

| Original state | Responsibility | StreetChavesRust status |
|---|---|---|
| `CHS_ABERTURA` | Opening sequence and attract/demo mode | **Partial.** A new vector intro and title flow exist. The opening is not a pixel-for-pixel reproduction. The original attract demonstration, including its random stage/fighter setup and demo fight, remains pending. |
| `CHS_CONFIG` | F1 game configuration | **First functional version.** Twelve original options have an editable menu and persistent settings. Difficulty, power mode and damage are consumed by the current runtime; round timing, rounds, blood, language, sound/music preview, flash and full HUD behavior still need their gameplay subsystems. |
| `CHS_TECLAS` | F2 player-one keyboard remapping | **First functional version.** Eight P1 physical keys can be captured, duplicate assignments are rejected, and the shared Foundation key map is applied in both SDL3 and browser hosts. P2 keeps its default layout. |
| `CHS_SELECAO` | Player entry and fighter selection | **First functional version.** The 15-character roster now uses a credit ledger: key 5 inserts, keys 1/2 spend credits to enter P1/P2, and the existing separate cursors/confirmations remain. Original locked-fighter and transition semantics are not all reproduced. |
| `CHS_ADVERSARIOS` | Campaign opponent order and next-stage presentation | **Core model implemented; host screen/transition pending.** Rust now generates seven no-repeat opponents while respecting the selected fighter, `NaoEnfrenta` and the configured boss; phase 8 is the boss. Campaign progress, stage selection and the 151-tick presentation clock are modeled. The host still needs to connect win/loss outcomes, render this screen and execute `CHS_PREPARALUTA`. |
| `CHS_PREPARALUTA` | Load fighters, stage, music and round state | **Folded into fight startup.** Preparation is currently incorporated into the transition into the first fight loop rather than represented by a faithful standalone screen/state. |
| `CHS_LUTA` | Combat simulation, rendering, HUD, timing and round flow | **First executable loop with source-driven CPU and human guard.** The unjoined P2 CPU runs `Comum.rot [IA]` plus C01-C15 `[IA]`, source projectile defense and the exact Pascal CPU difficulty/RNG rules. Human players now execute the original back/low-back guard branch when an enemy attack or hit-capable effect is present. The campaign model can reload the phase's CPU fighter and update `Fase`; match-end orchestration and the between-fight host screen remain pending. Full effect catalog animation/physics/collision parity, rounds, pauses and end-of-fight flow remain incomplete. |

## Other original systems still missing or incomplete

- **Credits/fichas:** key 5 inserts credits up to 99; key 1 and key 2 spend credits to enter P1/P2. A credit can also bring P2 into a live fight, disabling the CPU. Coin counts reset per session.
- **Attract mode:** no automatic demo loop is running from the opening screen.
- **F1 configuration:** all twelve original options are editable and persisted. Difficulty, power mode and damage currently affect the runtime. Round time, round count, blood, language, sound/music preview, flash and full fight-HUD toggles are stored but await their consuming subsystems.
- **F2 key remapping:** eight P1 physical actions are captured and persisted through the shared Foundation SDL3/WebAssembly mapping contract. Device-specific gamepad remapping remains pending.
- **Campaign/adversaries:** `campaign.rs` now models the seven random non-repeating opponents, `NaoEnfrenta`/selected-fighter/boss exclusions, boss at phase 8, victory/defeat progression, alternative/rare/secret stage choice and the original 151-tick presentation clock. `FightSimulation::load_campaign_opponent` refreshes the CPU character and phase-specific AI. The native/Web host still must supply the INI metadata, detect fight outcomes, render `CHS_ADVERSARIOS`, carry player state and enter `CHS_PREPARALUTA`.
- **CPU opponent AI:** the unjoined P2 slot runs source `Comum.rot [IA]` followed by the selected fighter's own C01-C15 `[IA]` section through the shared ROT interpreter and RNG stream. Projectile threat, `DistProj`, `OppY`, `Ameaca`, projectile-height response, incoming fighter-attack defense and the exact CpuNovo/defendergolpe formulas are implemented. `CampaignProgress` plus `load_campaign_opponent` now supplies the changing phase/character when the host advances the campaign. C16 Chapolin remains a documentation-only custom-character example, not part of the standard roster or executable.
- **Continue, records and initials:** the original has continue countdown, high-score entry and record-table states. These are not complete in the Rust startup.
- **Character endings and credits:** the source has per-character final scripts and the post-fight notice/credits flow. These remain pending.
- **Effects/fight fidelity:** the Rust effect arena currently supports only eight spawn/animation types (2, 7, 8, 13, 15, 16, 18 and 19). The original INI defines 40 entries: 1–36 for the base game, 37–38 for the C16 Chapolin extension and 39–40 for destructible-stage debris. Pascal `desenhaefeitos` also applies ordered sprite rules, special per-type logic, orbit/gravity/motion, effect-vs-effect reflection/destruction, effect-vs-fighter guard/hit/item consequences, lifetime and culling. Rust does not yet have full catalog, visual, physics or collision parity. These behaviors must be ported from `Comum_Sprites.inc`/`Dados/Efeitos.ini` with deterministic tests; no all-effects parity is claimed.

## Suggested implementation sequence

1. Validate the integrated credits/F1/F2/CPU path on native SDL3 and browser WebAssembly, including the original CPU difficulty and defense formulas.
2. Implement the opening attract/demo mode and deterministic demo setup.
3. Connect the new campaign model to match-end win/loss detection, per-fighter `Chefe`/`NaoEnfrenta`/stage metadata, `CHS_ADVERSARIOS` rendering and next-fight setup.
4. Complete `CHS_PREPARALUTA` as an explicit transition so asset loading, selected stage/music and round state can be validated before each fight.
5. Add the round timer/round manager, remaining F1 consumers, full fight HUD and source-accurate audio/visual toggles.
6. Continue combat parity with source-derived deterministic tests, then add continue, records, initials, endings and post-fight credits.

Do not treat a screen as complete because a placeholder panel exists. Completion requires the corresponding input, state transitions, persistent settings (where applicable), source-derived rules and regression coverage.

## Source references

The upstream source and original game resource tree are in [danimesq/OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves). Relevant state implementations:

- [Opening and attract mode: Estado1_Abertura.inc](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado1_Abertura.inc)
- [Configuration: Estado2_Config.inc](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado2_Config.inc)
- [Keyboard mapping: Estado3_Teclas.inc](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado3_Teclas.inc)
- [Fighter selection: Estado4_Selecao.inc](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado4_Selecao.inc)
- [Opponent order: Estado5_Adversarios.inc](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado5_Adversarios.inc)
- [Fight preparation: Estado6_PreparaLuta.inc](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado6_PreparaLuta.inc)
- [Fight runtime: Estado7_Luta.inc](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado7_Luta.inc)
- [Records: Estado8_Recordes.inc](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado8_Recordes.inc)
- [Continue: Estado9_Continuar.inc](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado9_Continuar.inc)
- [Character endings: Estado10_Final.inc](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado10_Final.inc)
- [Initials entry: Estado12_Iniciais.inc](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado12_Iniciais.inc)

## Source assets

The build configuration consumes the public source repository directly from a local checkout. Useful resource directories include:

- [Stage backgrounds](https://github.com/danimesq/OpenStreetChaves/tree/main/Cenarios)
- [Fighter sprite sheets](https://github.com/danimesq/OpenStreetChaves/tree/main/Lutadores)
- [Fighter and stage data](https://github.com/danimesq/OpenStreetChaves/tree/main/Dados)
- [Sound effects](https://github.com/danimesq/OpenStreetChaves/tree/main/Sons)
- [Music](https://github.com/danimesq/OpenStreetChaves/tree/main/Musicas)
- [Opening, menu and ending artwork/audio](https://github.com/danimesq/OpenStreetChaves/tree/main/Embutidos)
- [Stage animations](https://github.com/danimesq/OpenStreetChaves/tree/main/Animacoes)
- [Effects](https://github.com/danimesq/OpenStreetChaves/tree/main/Efeitos)
- [Voices](https://github.com/danimesq/OpenStreetChaves/tree/main/Falas)
- [Fighter portraits](https://github.com/danimesq/OpenStreetChaves/tree/main/Rostos)
- [Fonts](https://github.com/danimesq/OpenStreetChaves/tree/main/Fontes)

The upstream repository includes a [GPL-3.0 license](https://github.com/danimesq/OpenStreetChaves/blob/main/LICENSE). The Foundation build records upstream source URLs for each asset configured in `FoundationWebBuild.toml`.
