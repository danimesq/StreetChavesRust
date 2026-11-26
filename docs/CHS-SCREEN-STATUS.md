# CHS screen and game-flow implementation status

This document tracks StreetChavesRust against the original state's responsibilities in the OpenStreetChaves source. Statuses describe the Rust reconstruction, not the completeness of the source project.

## Screen matrix

| Original state | Responsibility | StreetChavesRust status |
|---|---|---|
| `CHS_ABERTURA` | Opening sequence and attract/demo mode | **Partial.** A new vector intro and title flow exist. The opening is not a pixel-for-pixel reproduction. The original attract demonstration, including its random stage/fighter setup and demo fight, remains pending. |
| `CHS_CONFIG` | F1 game configuration | **Pending.** No complete configuration screen or persistent setting editor exists yet. |
| `CHS_TECLAS` | F2 player-one keyboard remapping | **Pending.** Runtime keyboard mappings are fixed; the original capture-and-remap flow is not implemented. |
| `CHS_SELECAO` | Player entry and fighter selection | **First functional version.** The Rust startup flow has a 15-character selection screen with separate P1/P2 cursors and confirmation. Original coin/credit entry, full grid behavior, locked fighters and source-specific transition rules are not all reproduced. |
| `CHS_ADVERSARIOS` | Campaign opponent order and next-stage presentation | **Pending.** The original seven-opponent sequence, character-specific boss, stage choice and campaign progression are not implemented as a complete flow. |
| `CHS_PREPARALUTA` | Load fighters, stage, music and round state | **Folded into fight startup.** Preparation is currently incorporated into the transition into the first fight loop rather than represented by a faithful standalone screen/state. |
| `CHS_LUTA` | Combat simulation, rendering, HUD, timing and round flow | **First executable loop.** StreetChavesRust has a working first fight loop and a 28 ms logical tick. It is not full gameplay parity: combat commands, CPU behavior, effects, collision consequences, rounds, pauses and end-of-fight flow still have gaps. |

## Other original systems still missing or incomplete

- **Credits/fichas:** the original inserts credits and lets P1/P2 join using them. The current Rust startup is not yet driven by a faithful credit ledger or coin-input flow.
- **Attract mode:** no automatic demo loop is running from the opening screen.
- **F1 configuration:** difficulty, round duration, power mode, damage, blood, round count, language, sound/music selection, flash, level display and credit display need an editable Rust settings model and screen.
- **F2 key remapping:** the original captures eight P1 actions and persists the mapping. StreetChavesRust still uses fixed keyboard bindings.
- **Campaign/adversaries:** the original creates seven opponents plus a character-specific boss, applies exclusions and alternate/rare stages, and advances the campaign. This is not yet implemented in Rust.
- **CPU opponent AI:** when P2 does not join, the current second fighter is a stationary training opponent, not an AI-controlled adversary.
- **Continue, records and initials:** the original has continue countdown, high-score entry and record-table states. These are not complete in the Rust startup.
- **Character endings and credits:** the source has per-character final scripts and the post-fight notice/credits flow. These remain pending.
- **Fight fidelity:** the first loop is not the complete original fight runtime. Remaining work includes source-ordered action/animation execution, stage animation and destructible objects, projectile/effect rendering, collision consequences, hit-stop/round transitions, full knockback/physics and sound/music selection tied to the original state.

## Suggested implementation sequence

1. Establish a shared persistent game-settings model and credit/coin state.
2. Implement the F1 configuration screen against that model.
3. Implement F2 key remapping as a platform-neutral binding map, with native and browser adapters translating physical keys into logical actions.
4. Implement attract-mode scheduling and deterministic demo setup.
5. Build the opponent-order/campaign model from the original roster metadata, then render `CHS_ADVERSARIOS`.
6. Split fight preparation into an explicit transition model so asset loading, selected stage/music and round state can be validated before combat starts.
7. Continue combat parity with source-derived deterministic tests, then add continue, records, initials, endings and credits.

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
