# C01 special-command entry and character AI

## Scope

This slice connects the C01 (Chaves) special command IDs to the live combat state and to the character-specific CPU script. It is deliberately separate from the C01 [Quadro] executor already present in CombatState::update_c01_character_frame.

Normative Pascal source: OpenStreetChaves Dados/Lutadores/C01.rot and source/game/ChavesTipos.pas, pinned to commit e1b025c35ecbc2b14d8635564aafd23c2107d3d1.

These are source-derived Rust contracts and tests. They are not Pascal golden captures or original-executable validation.

## Command dispatch

| Pascal command | Numeric code | Rust variant |
|---|---:|---|
| cmEsp1Fraco / cmEsp1Forte | 16 / 17 | Esp1Weak / Esp1Strong |
| cmEsp2Fraco / cmEsp2Forte | 18 / 19 | Esp2Weak / Esp2Strong |
| cmEsp3Fraco / cmEsp3Forte | 20 / 21 | Esp3Weak / Esp3Strong |
| cmEsp4Fraco / cmEsp4Forte | 22 / 23 | Esp4Weak / Esp4Strong |
| cmEspFinal | 24 | Final |

dispatch_chaves_special_code is the numeric entry point for input adapters. The full dispatcher executes Common [Acao] (clears P.Defendendo), then C01 [Acao], then Common [AcaoFim]. The action-end phase advances QuadroAcao only if the fighter remains in acAndando, wrapping after frame 16. This also applies when a special is rejected and the fighter remains walking. start_chaves_special remains the character-only transition for callers that already executed the common prelude; it does not run Common [AcaoFim].

Special 1 queries the live 80-slot EffectArena for a type-7 instance owned by the current player. This is the actual ExisteEfeito(7, Jog) query against simulation state, not an injected boolean or a parallel effect list.

The existing sprite_frame and force_strength fields are reused. Additional C01 state is represented with Pascal widths: health: f64, power: u16, charge_power: f64, and byte-sized jump/low-time/sequence fields. The final move also has a separate screen_flash request; it does not overwrite screen_shake. Its pending sound uses the shared pending_sound field.

## Entry rules

| Command | Eligible current actions | New action | Duration |
|---|---|---|---:|
| Special 1 weak/strong | idle, walking, standing guard, low guard | Special 1 | 28 |
| Special 2 weak/strong | idle, walking, standing guard, low guard | Special 2 | 16 weak / 27 strong |
| Special 3 weak/strong | idle, walking, jumping, low guard, Special 2; also ChaoY - 90 <= P.Y | Special 3 | 10 weak / 16 strong |
| Special 4 weak/strong | idle, walking, standing guard, low guard | Special 4 | 22 |
| Final | idle, walking, strong punch, crouching, standing guard, low guard; and P.Vida <= 15 or P.Poder > 0 | Special 5 | 38 |

Charge increments occur only when the pre-increment value is below CargaMax; the result is not clamped. Special 3 snaps to the ground and clears airborne, vertical velocity, jump direction, low-input timer and sequence state. The final command clears charge only when power is exactly 3, decrements the Pascal Word with 16-bit wraparound, writes sprite frame Desloc + 36, requests flash 22 and queues sound 19.

## Character-specific CPU dispatcher

ChavesCharacterActionScript implements the C01 [IA] section as a child of the common AI tick. tick_common_ai_with_chaves runs the shared Comum.rot [IA] logic first and C01 [IA] second, passing the exact same mutable Pascal RNG cursor through both.

The six independent C01 decisions remain in source order:

1. Threat present and projectile distance above 140: Random(Round(7 * Fator)), then Random(2) to choose Special 1 weak/strong.
2. No threat and distance above 120: Random(Round(30 * Fator)), then Special 1 weak/strong selection.
3. No threat and distance below 200: Random(Round(40 * Fator)), then Special 2 weak/strong selection.
4. Distance below 150 and opponent above ground: Random(Round(50 * Fator)), then Special 3 weak/strong selection.
5. No threat and distance below 200: an independent Random(Round(40 * Fator)), then Special 4 weak/strong selection.
6. Distance below 150 and opponent above ground: an independent Random(Round(50 * Fator)), then the final special without a strength roll.

Each probability call uses the LongInt overload because the Pascal expression is Random(Round(...)). Each ordinary special's second roll is also a separate LongInt Random(2). Rejected command-entry gates do not undo or skip RNG calls that have already occurred. The decisions are independent if branches, not an exclusive match.

The globals GanhoCarga, CargaMax and ChaoY are injected through ChavesSpecialContext; they are not guessed from incomplete AI snapshots.

## Deliberate boundaries

- C01 [Quadro] remains in update_c01_character_frame; this dispatcher does not advance frames or create effects.
- C01 [Acerto] remains the existing apply_c01_hit_reaction hook. The hook is not claimed to be wired to a complete melee collision pipeline.
- Human input recognition/SDL event routing is not implemented here. The public numeric command entry point is ready for that adapter.
- The current AI tests use deterministic scripted RNG values and source-order expectations. A real Pascal C01 AI/entry capture is still required for differential validation.
- SDL rendering of effect snapshots, effect collision consequences and original-executable validation remain separate work.
