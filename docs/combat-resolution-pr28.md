# PR #28: Guard and damage resolution

## Normative source

Pinned Pascal source: OpenStreetChaves `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`, `source/game/Estado7_Luta.inc`, `TJogo.L7_Acertos`.

The resolver follows the source order for one detected P→O contact:

1. Cancel standing guard for crouching kicks (`acAgChuteFraco`, `acAgChuteForte`).
2. Cancel crouching guard for aerial attacks (`acArSocoFraco`, `acArChuteFraco`, `acArSocoForte`, `acArChuteForte`).
3. If the defender still guards: set `O.QuadroAcao := 1`, apply `P.DanoDefesa * NivelDano` unless entry warning / stopped clock suppresses health mutation, and consume only `P.GolpeAtivo`. A chip-kill requests `cmDerrubar`; because common `[Acao]` clears `O.Defendendo`, execution then falls through into the clean-hit branch in the same `L7_Acertos` call.
4. Otherwise: update defender facing; reset the combo when `P.TempoAcertos <= 35`; increment Byte `P.Acertos`; set `P.TempoAcertos := 44`, or 50 for crouching-strong / aerial-strong kick; apply `P.DanoGolpe * NivelDano` unless suppressed; choose the standard reaction; set `O.Recuo := 4`; consume both active-attack flags.
5. Execute defender `cmApanharBaixo` or `cmApanharAlto` through the existing common/character/common-end ROT lifecycle. If the source knockdown predicate is true, execute `cmDerrubar` afterwards.
6. Execute the attacker's actual character-specific `[Acerto]` section. This is the source hook after standard damage, reaction and recoil.

The resolver does not replace or flatten the per-character ROT scripts. It dispatches the original pinned section for C01–C15. A missing `[Acerto]` section (C08, C13, C14 in the pinned files) is parsed as absent and remains a no-op, not an invented reaction.

## Implementation

- `combat_resolution.rs` implements deterministic common guard, chip damage, damage, combo timer/count, standard reaction selection, recoil and attack-flag consumption.
- `FightSimulation::resolve_melee_contacts` consumes the ordered contacts from PR #27 and applies the common consequences.
- The defender's standard reaction and knockdown commands use the existing ROT host and live fighter state.
- The attacker's `[Acerto]` is executed through the existing ROT interpreter with the same live state and RNG stream.
- `NivelDano`, `AvisoEntrada`, sprite heights and `ChaoY` are not guessed: the relevant values are explicit method inputs or existing simulation configuration.

## Character-specific [Acerto] inventory

| Fighter | Pinned [Acerto] behavior |
|---|---|
| C01 Chaves | Strong Special 2 sets opponent recoil to 20 |
| C02 Seu Madruga | Strong Special 2 recoil; Special 4/6 chained actions; Special 6 combo increment |
| C03 Chiquinha | Special 5 chains into Special 6 |
| C04 Quico | Special 2/3 combo windows; Special 5 spin, recoil and launch |
| C05 Dona Florinda | Special 5 frame-gated spin and launch |
| C06 Professor Girafales | Special 5 combo window/spin; strong Special 3 recoil |
| C07 Sr. Barriga | Strong Special 3 recoil; Special 5 chains into Special 6 |
| C08 Bruxa | No [Acerto] section in pinned source |
| C09 Paty | Special 5 frame-gated spin and recoil |
| C10 Nhonho | Special 5 frame-gated spin; strong Special 4 recoil |
| C11 Godines | Special 5 frame-gated spin |
| C12 Sr. Furtado | Special 4 frame-gated spin; strong Special 2 recoil |
| C13 Jaiminho | No [Acerto] section in pinned source |
| C14 Dona Neves | No [Acerto] section in pinned source |
| C15 Gloria | Special 3 combo window and vertical launch |

The ROT scripts, not this table, remain executable authority.

## Tests

- Successful hit: health, combo count/timer, recoil, standard reaction and both attack flags.
- Guarded hit: chip damage and attacker-only attack consumption.
- Chip-damage KO: knockdown action clears guard, then normal hit damage/combo/reaction/recoil/character `[Acerto]` run exactly once; chip damage is not applied twice.
- Crouching kick vs standing guard and aerial attack vs crouching guard.
- Health suppression during entry warning / stopped clock.
- Integration test verifies common resolution precedes the real C01 `[Acerto]` script.

## Deliberate remaining boundaries

This PR covers the requested state consequences and the character-specific `[Acerto]` dispatch. It does not yet claim a full reproduction of every side effect in `L7_Acertos`: impact-pause duration/RNG, hit/block spark and blood effect spawning, sound selection, screen flash, score/first-hit bonus, charge gain, health-bar delay, edge-of-screen pushback, and the subsequent same-tick victim `AtualizaQuadro` / box reselection remain separate integration work. The current public resolver is an explicit simulation operation; automatic placement into the authoritative tick phase is also a follow-up.

No SDL, renderer, audio backend, or Foundation dependency is introduced.
