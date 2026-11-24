# 🌈 PR #24 — Frame update and box application

## 🌈 Objective

Connect the source-audited Pascal frame/box lifecycle to the authoritative `FightSimulation`, without implementing melee collision consequences in this PR.

The normative source remains OpenStreetChaves at `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`. The C port is a comparison source only. The original executable remains the final compatibility authority for unresolved differences.

## 🌈 Required per-fighter order

The Pascal `AtualizaQuadro` path is ordered:

1. Execute common `[Quadro]`.
2. Execute the current fighter's character-specific `[Quadro]`.
3. Apply the current hurtbox and action/timer hitbox rows through `AplicarCaixas`.

This must become one reusable simulation operation for either fighter. It must use the authoritative `CombatState` and the fight's existing `Fpc322Rng`; no parallel fighter snapshot or private RNG is allowed.

## 🌈 Box-selection semantics to preserve

### Hurtbox

- Select by the current sprite frame.
- Player two's sprite strip uses the 50-frame offset before table lookup.
- Only a defined body row writes new bounds.
- A lookup miss retains the previous body bounds.
- Mirror horizontal endpoints when facing left, using the source endpoint swap/negation.

### Hitbox

- Scan Pascal attack rows in original INI order, without sorting or early exit.
- Match action and action timer, then evaluate each row's optional force condition against the current force at that exact point in the scan.
- A nonzero assigned force changes the fighter's force and may affect later row matches.
- A matched inactive row replaces stored geometry/metadata with `active = false`.
- No matching row retains the previous attack record and active state.
- Keep the fighter-level `FighterState.attack_active` (`TLutador.GolpeAtivo`) distinct from the selected row's `HitboxRecord.active`.
- Do not globally remirror stored coordinates when a selector miss occurs.

## 🌈 Scope boundaries

Included:
- common and character `[Quadro]` execution for explicitly supported scripts/branches;
- live hurtbox and hitbox state on each fighter;
- source-ordered `AplicarCaixas` behavior;
- deterministic reports/errors and source-backed tests.

Not included:
- AABB hit detection between fighters;
- attack qualification, blocking, damage, hit-stop or hit consequences;
- `ExecutarAcerto` and character `[Acerto]` scripts;
- claiming complete `[Quadro]` coverage for all 15 fighters before every host field/helper and branch is mapped.

Unsupported ROT fields/helpers/branches must fail visibly with source context. A bounded first slice is preferable to silently skipping source behavior.

## 🌈 Validation matrix

- [ ] Common `[Quadro]` runs before character `[Quadro]`.
- [ ] Character `[Quadro]` mutations are visible to the same tick's box selection.
- [ ] Player-one and player-two lookup, including the 50-frame offset.
- [ ] Hurtbox lookup miss retains prior bounds.
- [ ] Left-facing horizontal mirroring uses endpoint swap/negation.
- [ ] Attack rows preserve original order and evaluate force conditions against evolving force.
- [ ] A later matching row overwrites an earlier row.
- [ ] A matched inactive row replaces geometry and metadata but disables the row.
- [ ] No attack-row match retains the prior record and active state.
- [ ] Fighter-level `attack_active` remains independent of row-level `active`.
- [ ] Existing command, ROT, effect and RNG tests remain green.
- [ ] `cargo fmt --all -- --check` and `cargo test --all-targets` pass in CI.

## 🌈 Known compatibility question

The Pascal selector and C port differ for fighter 11, action/state 27, timer 1. Keep Pascal's strength-conditioned row selection normative for this implementation, preserve the discrepancy in tests/docs, and do not claim original-executable parity until that case is checked against the original game.

## 🌈 Follow-up milestones

The next separate milestones are **Collision Resolution** and **Acerto + Character Reactions**, as recorded in [the project roadmap](project-roadmap.md). They must build on this frame/box pipeline rather than being folded into this PR.
