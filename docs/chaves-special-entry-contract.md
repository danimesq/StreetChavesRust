# 🌈 C01 special-command entry contract

## 🌈 Scope and evidence

This document records the first executable Rust slice for Chaves (fighter C01): the command-entry logic in the Pascal `[Acao]` section. It does not implement or claim parity for the later per-frame `[Quadro]` section.

Normative source revisions:

- [OpenStreetChaves / C01.rot](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot), commit `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`.
- [OpenStreetChaves / ChavesTipos.pas](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/game/ChavesTipos.pas), same pinned commit.

Pascal is the normative project source. These are source-derived contracts, not observations from the original executable. The experimental Pascal JSONL exporter does not yet provide a focused C01 entry/exit fixture for these five moves.

## 🌈 Numeric identifiers

| Pascal item | Value | Rust representation |
|---|---:|---|
| `pnChaves` | 1 | `CHARACTER_CHAVES` |
| `cmEsp1Fraco` / `cmEsp1Forte` | 16 / 17 | `ChavesSpecialCommand::Esp1Weak/Strong` |
| `cmEsp2Fraco` / `cmEsp2Forte` | 18 / 19 | `ChavesSpecialCommand::Esp2Weak/Strong` |
| `cmEsp3Fraco` / `cmEsp3Forte` | 20 / 21 | `ChavesSpecialCommand::Esp3Weak/Strong` |
| `cmEsp4Fraco` / `cmEsp4Forte` | 22 / 23 | `ChavesSpecialCommand::Esp4Weak/Strong` |
| `cmEspFinal` | 24 | `ChavesSpecialCommand::Final` |
| `acEspecial1..5` | 25..29 | `ACTION_SPECIAL_1..5` |
| `fgFraco` / `fgForte` / `fgEspecial` | 1 / 2 / 3 | `STRENGTH_WEAK/STRONG/SPECIAL` |
| `puloNenhum` | 0 | `JUMP_DIRECTION_NONE` |
| `sndEspFinal` | 19 | `SOUND_FINAL_SPECIAL` |

The source declares `TLutador.Poder` as `Word`, `Vida` and `CargaPoder` as `Double`, `Quadro` as `Byte`, and `ForcaGolpe`, `DirPulo`, `TempoBaixo`, and `SeqEstado` as byte-sized fields. Rust now retains these C01-relevant values with matching widths.

## 🌈 Command entry rules

All state names below refer to Pascal action IDs. The entry contract is fighter-specific and must not be reused as a generic rule for the other 14 fighters.

| Command | Allowed current actions | New action | Duration |
|---|---|---|---:|
| Special 1, weak/strong | idle, walking, standing guard, low guard | `acEspecial1` (25) | 28 |
| Special 2, weak/strong | idle, walking, standing guard, low guard | `acEspecial2` (26) | 16 weak / 27 strong |
| Special 3, weak/strong | idle, walking, jumping, low guard, Special 2 | `acEspecial3` (27) | 10 weak / 16 strong |
| Special 4, weak/strong | idle, walking, standing guard, low guard | `acEspecial4` (28) | 22 |
| Final | idle, walking, strong punch, crouching, standing guard, low guard | `acEspecial5` (29) | 38 |

Additional gates:

- Special 1 is rejected while `ExisteEfeito(7, Jog)` is true.
- Special 3 additionally requires `ChaoY - 90 <= P.Y`. The comparison is kept in floating-point space.
- Final additionally requires `P.Vida <= 15` **or** `P.Poder > 0`.
- No command entry here consumes RNG.

## 🌈 Ordered entry mutations

`CombatState::start_chaves_special` emits a `ChavesSpecialEvent` stream while applying the same ordered writes. The stream is diagnostic evidence for the Rust transition, not a render queue.

| Move | Ordered mutations |
|---|---|
| Special 1 | action; action timer 0; duration 28; charge + `5 * GanhoCarga` only when the old charge is below `CargaMax`; weak/strong force |
| Special 2 | action; action timer 0; conditional charge +5; duration 16/27; weak/strong force |
| Special 3 | action; conditional charge +6; Y snaps to `ChaoY`; `NoAr := False`; action timer 0; `VelY := 0`; `DirPulo := puloNenhum`; `TempoBaixo := 0`; `SeqEstado := 0`; duration 10/16; weak/strong force |
| Special 4 | action; action timer 0; conditional charge +5; duration 22; weak/strong force |
| Final | action; action timer 0; duration 38; charge reset only when power is 3; power decremented; sprite frame set to `Desloc + 36`; special force; screen flash 22; pending sound 19 |

Charge is not clamped after the increment. Pascal checks only whether the pre-increment value is below `CargaMax`, so a value near the cap can rise above it. The Rust contract preserves that behavior.

The final move's power field is a Pascal `Word`, not a signed integer. The interpreter converts the assignment result back through `Word(...)`; Rust therefore uses 16-bit wrapping subtraction. This matters for the low-health path where the command is accepted with zero power.

For player 1, `Desloc = 0`; for player 2, `Desloc = 50`. The final move writes the sprite frame immediately at entry. Ordinary-special sprite changes happen later in `[Quadro]`.

## 🌈 State fields added to the Rust fighter

| Pascal field | Rust field | Type |
|---|---|---|
| `Vida` | `health` | `f64` |
| `Quadro` | `sprite_frame` | `u8` |
| `Poder` | `power` | `u16` |
| `CargaPoder` | `charge_power` | `f64` |
| `ForcaGolpe` | `attack_strength` | `u8` |
| `DirPulo` | `jump_direction` | `u8` |
| `TempoBaixo` | `low_time` | `u8` |
| `SeqEstado` | `sequence_state` | `u8` |
| global `FlashTela` | `CombatState::screen_flash` | `u8` |
| global `SomPendente` | `CombatState::pending_sound` | `u8` |

The environment-dependent inputs `GanhoCarga`, `CargaMax`, `ChaoY`, and the result of the real `ExisteEfeito(7, Jog)` query arrive through `ChavesSpecialContext`.

## 🌈 Deliberately not implemented in this slice

The later C01 `[Quadro]` section has additional behavior which must remain separate from entry:

- Special 1 creates effect type 7 at timer 10, choosing variant A/B by strength, and queues sound 14.
- Special 2 queues sound 15 at timer 0, creates three type-2 effects with two RNG calls per iteration when the sprite frame is +41, pushes the opponent, creates type-16 effects from timer 9 onward, and marks the move as a kick.
- Special 3 queues sound 16, alternates sprite frames +43/+44, moves the fighter vertically and forward, changes opponent invulnerability according to hit count/timer, and sets the airborne flag at timer 20.
- Special 4 queues sound 17, changes sprite frames at its timer boundaries, pushes the opponent during timers 4–13, and marks the move as a kick from timer 12.
- Final changes sprite frames at timers 0/3/6/9, queues sound 18 at timer 9, creates effect type 8 and sets screen shake to 20.

There is not yet a concrete Rust effect-slot store/update/render system to receive those operations. The `effect_7_exists` input is a seam for querying the real effect system when it exists; it is not a fabricated effect list. The ordinary move sounds/effects above are not written into `SomPendente` by the C01 `[Acao]` branch.

Also pending: C01 special AI command dispatch, C01 `[Acerto]` (Special 2 strong sets opponent recoil to 20), and a focused Pascal capture containing the complete fighter/global/instance state around each entry. Static source tests do not replace that capture or original-executable validation.

## 🌈 Validation status

- [x] Model the C01 special command IDs and action/strength identifiers.
- [x] Model the five C01 entry guards and their source-order mutations.
- [x] Preserve the Pascal field widths needed by the entry contract.
- [x] Cover accepted and rejected transitions with deterministic Rust contract tests.
- [ ] Connect `effect_7_exists` to the real effect-slot query.
- [ ] Implement the C01 `[Quadro]` operations against the real effect/audio/rendering systems.
- [ ] Integrate the special commands into the character-script dispatcher and AI path.
- [ ] Implement C01 `[Acerto]` after the shared hit-resolution order exists.
- [ ] Capture and compare real Pascal entry checkpoints and RNG/control traces.
- [ ] Validate behavior against the original executable.
