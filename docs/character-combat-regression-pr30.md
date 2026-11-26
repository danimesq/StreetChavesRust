# Character-specific combat regression audit (PR #30)

## Scope

This change adds executable regression coverage for the pinned Pascal/ROT runtime and records a differential audit against the independent C17 implementation. It does not claim that the Rust combat resolver already reproduces every C17-specific launch, spin, or hitstun override.

## Executable regressions

- All original fighters C01-C15 must execute their pinned live `[Quadro]` section and update their frame boxes without runtime errors.
- C08 `[Quadro]`: while `P.Acao = acEspecial5`, `O.Invulneravel := False` is unconditional.
- C13 `[Quadro]`: while `P.Acao = acEspecial5`, the opponent becomes vulnerable only when `P.Acertos > 1`.
- C14 `[Quadro]`: no opponent-side writes are introduced by the Rust port; the regression asserts the opponent state remains unchanged for the selected action.
- Common combo windows: ordinary attacks use 44, crouching-strong / aerial-strong kicks use 50, and a previous combo timer <= 35 resets the count.
- An airborne defender is marked for knockdown after the standard reaction.

## C17 differential reference

Reference: `danimesq/open-street-chaves/src/fight/combat.c`, pinned for this audit at commit `a417c0621fca8438fe81ab46a39b3865dd95b49a`, function `melee_step`.

The table records the character-specific branches in that C implementation, not invented Pascal `[Acerto]` sections.

| Fighter | Combo-window override | Spin / rotation condition | Hitstun override | Vertical launch |
|---|---|---|---|---|
| C01 | Common 44/50 | — | special2, strong: 20 | — |
| C02 | Common 44/50 | — | special2, strong: 13 | — |
| C03 | Common 44/50 | — | — | — |
| C04 | special2: 46; special3: 47 | special5: 20°/tick | special5: 20 | special5: -30 |
| C05 | Common 44/50 | special5 at frame >= 15: 20°/tick | — | special5 at frame >= 15: -30 |
| C06 | special5: 50 | special5 at frame >= 22: 20°/tick | special3, strong: 25 | — |
| C07 | Common 44/50 | — | special3, strong: 18 | — |
| C08 | Common 44/50 | — | — | — |
| C09 | Common 44/50 | special5 at frame >= 14: 20°/tick | special5 at frame >= 15: 25 | — |
| C10 | Common 44/50 | special5 at frame >= 15: 20°/tick | special4, strong: 13 | — |
| C11 | Common 44/50 | special5 at frame >= 20: 20°/tick | — | — |
| C12 | Common 44/50 | special4 at frame >= 22: 20°/tick | special2, strong: 25 | — |
| C13 | Common 44/50 | — | — | — |
| C14 | Common 44/50 | — | — | — |
| C15 | special3: 50 | — | — | special3: -26 |

C17 source mapping: combo windows at lines 114-121; spin branches at 129-135; knockdown and hitstun at 139-150; launches at 151-153; multi-hit chains at 164-168.

## Known parity gaps exposed by this audit

The current Rust resolver covers the common 44/50 combo windows, standard recoil assignment in `FightSimulation`, and the generic airborne/above-ground knockdown predicate. It does **not** yet model the C17-only combo overrides for C04/C06/C15, the per-character hitstun values, the spin-step assignments, or the C04/C05/C15 collision-time launch overrides.

Rust already has `angular_velocity` and `vertical_velocity` state fields, but merely having storage is not behavioral parity. The existing `recoil` field is not silently treated as a substitute for C17's `hitstun` or `rot_step`.

Those differences are deliberately retained as explicit follow-up work rather than hidden behind tests that only validate a copied lookup table. The Pascal/ROT scripts remain the authority for the original game; C17 is a differential reference.
