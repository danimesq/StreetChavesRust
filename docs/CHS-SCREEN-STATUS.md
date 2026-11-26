# CHS screen and game-flow implementation status

This document tracks StreetChavesRust against the original state's responsibilities in the OpenStreetChaves source. Statuses describe the Rust reconstruction, not the completeness of the source project.

## Screen matrix

| Original state | Responsibility | StreetChavesRust status |
|---|---|---|
| `CHS_ABERTURA` | Opening sequence and attract/demo mode | **Partial.** A new vector intro and title flow exist. The opening is not a pixel-for-pixel reproduction. The original attract demonstration, including its random stage/fighter setup and demo fight, remains pending. |
| `CHS_CONFIG` | F1 game configuration | **First functional version.** Twelve original options have an editable menu and persistent settings. Difficulty, power mode and damage are consumed by the current runtime; round timing, rounds, blood, language, sound/music preview, flash and full HUD behavior still need their gameplay subsystems. |
| `CHS_TECLAS` | F2 player-one keyboard remapping | **First functional version.** Eight P1 physical keys can be captured, duplicate assignments are rejected, and the shared Foundation key map is applied in both SDL3 and browser hosts. P2 keeps its default layout. |
| `CHS_SELECAO` | Player entry and fighter selection | **First functional version.** The 15-character roster now uses a credit ledger: key 5 inserts, keys 1/2 spend credits to enter P1/P2, and the existing separate cursors/confirmations remain. Original locked-fighter and transition semantics are not all reproduced. |
| `CHS_ADVERSARIOS` | Campaign opponent order and next-stage presentation | **Pending.** The original seven-opponent sequence, character-specific boss, stage choice and campaign progression are not implemented as a complete flow. |
| `CHS_PREPARALUTA` | Load fighters, stage, music and round state | **Folded into fight startup.** Preparation is currently incorporated into the transition into the first fight loop rather than represented by a faithful standalone screen/state. |
| `CHS_LUTA` | Combat simulation, rendering, HUD, timing and round flow | **First executable loop plus Common.rot CPU behavior.** An unjoined P2 slot now runs the source-ordered common AI target-distance, movement and close-range probability path. Per-character AI, full campaign opponent sequencing, complete collision/effect fidelity, rounds, pauses and end-of-fight flow remain incomplete. |

## Other original systems still missing or incomplete

- **Credits/fichas:** key 5 inserts credits up to 99; key 1 and key 2 spend credits to enter P1/P2. A credit can also bring P2 into a live fight, disabling the CPU. Coin counts reset per session.
- **Attract mode:** no automatic demo loop is running from the opening screen.
- **F1 configuration:** all twelve original options are editable and persisted. Difficulty, power mode and damage currently affect the runtime. Round time, round count, blood, language, sound/music preview, flash and full fight-HUD toggles are stored but await their consuming subsystems.
- **F2 key remapping:** eight P1 physical actions are captured and persisted through the shared Foundation SDL3/WebAssembly mapping contract. Device-specific gamepad remapping remains pending.
- **Campaign/adversaries:** the original creates seven opponents plus a character-specific boss, applies exclusions and alternate/rare stages, and advances the campaign. This is not yet implemented in Rust.
- **CPU opponent AI:** the unjoined P2 slot runs the ported Common.rot [IA] target-selection, approach and close-range probability chain. The selected difficulty currently drives a documented provisional probability curve; full per-character [IA], projectile-threat logic and exact Pascal Fator parity remain pending.
- **Continue, records and initials:** the original has continue countdown, high-score entry and record-table states. These are not complete in the Rust startup.
- **Character endings and credits:** the source has per-character final scripts and the post-fight notice/credits flow. These remain pending.
- **Fight fidelity:** the first loop is not the complete original fight runtime. Remaining work includes source-ordered action/animation execution, stage animation and destructible objects, projectile/effect rendering, collision consequences, hit-stop/round transitions, full knockback/physics and sound/music selection tied to the original state.

## Suggested implementation sequence

1. Validate the integrated credits/F1/F2/CPU path on native SDL3 and browser WebAssembly.
2. Implement the opening attract/demo mode and deterministic demo setup.
3. Build the opponent-order/campaign model from the original roster metadata, then render `CHS_ADVERSARIOS`.
4. Split fight preparation into an explicit transition model so asset loading, selected stage/music and round state can be validated before combat starts.
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
