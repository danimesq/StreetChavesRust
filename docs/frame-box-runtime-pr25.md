# 🌈 PR #25 — Live frame update and AplicarCaixas

## 🌈 Source order

The implementation follows OpenStreetChaves at commit `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`. Pascal's `AtualizaQuadro(Jog)` calls, in this exact order:

1. Common `[Quadro]` from `Comum.rot`.
2. Fighter-specific `[Quadro]` from `Cnn.rot`, when present.
3. `AplicarCaixas(Jog)`.

The simulation now runs the common section through the shared ROT interpreter and live fight host. The supported character-specific frame sections for C02-C15 and C01's non-manual branches also use that interpreter. C01's already-tested special/victory/pose frame branches retain their existing source-derived implementation for now; they run after the common section and before box selection.

A section error is recorded in `FightTickReport.errors` and prevents the box step for that fighter on that tick. The runtime does not silently skip an unsupported field/helper and pretend the frame update completed.

## 🌈 Imported frame data

`data/fighter-boxes.ini` contains the `CentroX`, `Corpo` and `Ataque` sections extracted from the 15 pinned Pascal fighter INIs.

| Source table | Rows | Meaning |
|---|---:|---|
| `CentroX` | 686 | Sprite centre offsets |
| `Corpo` | 305 | Defined hurtbox/body rectangles |
| `Ataque` | 379 | Action/timer attack rectangles and metadata |

Section rows remain in original file order. The parser rejects malformed values and retains duplicate selector keys. The file records the upstream commit and GPL-3.0-only source license.

## 🌈 Applied box semantics

### Hurtbox / Corpo

- Look up the fighter's current sprite frame.
- Subtract 50 for player two before the lookup, and only for this body-frame lookup.
- Write the body bounds only when a defined row exists.
- Mirror horizontal endpoints for left-facing fighters using the original endpoint swap/negation.
- Retain the prior body bounds when no defined row matches.

### Attack / Ataque

- Scan every attack row in original INI order.
- Match current action and action timer.
- Evaluate each optional force condition against the current force at that point in the scan.
- Apply nonzero row force assignments immediately, so they can affect later rows.
- Allow later matching rows to overwrite earlier matches.
- Keep a matched inactive row as a stored attack box with `active = false`.
- Retain the previous attack box, fighter force and attack-active flag when no row matches.
- Keep `FighterState.attack_active` separate from the stored `FighterState.attack_box.active`; a selected row writes the former from its active metadata.

The existing `PascalAttackState::apply_ordered_rows` contract is used for this behavior rather than implementing a second selector.

## 🌈 Runtime integration

`FightSimulation::advance_fighter_frame` performs the frame sequence for each fighter when the fight is not hit-stopped and the screen is not flashing. It shares the fight's existing `Fpc322Rng` with ROT frame scripts, then calls the live box selector and records successful frame/box work in `FightTickReport`.

The common script now has the `AndandoFrente` binding needed by its walking branch. Current unsupported character ROT fields, helpers and nested `AcaoJogador` calls still return errors. They are not treated as complete roster support.

## 🌈 Validation added

- Load and validate all 15 fighter datasets and assert the 305/379 row totals.
- Preserve fighter 11's duplicate action 27 / timer 1 strength rows in Pascal order.
- Run common and character frame sections for both players and verify player-two's 50-frame offset.
- Apply left-facing hurtbox and attack-box mirroring.
- Verify weak/strong fighter-11 selectors choose their own Pascal rows.
- Verify lookup misses retain the old attack record and active flag.
- Verify a matched inactive row replaces the stored geometry and disables the active flag.
- Run `cargo fmt --all -- --check` and `cargo test --all-targets` in CI.

## 🌈 Boundaries and follow-ups

This PR connects frame scripts and box selection; it does **not** resolve collisions. AABB overlap, hit qualification, blocking, damage, hit-stop consequences, `ExecutarAcerto`, and character `[Acerto]` reactions remain the separate Roadmap milestones.

The fighter-11 action/state 27, timer 1 disagreement between the Pascal and C source selectors remains a known compatibility question against the original executable. Pascal's strength-specific behavior remains normative in this project.

Action-end transitions and the complete L7 physics/fight-loop order are not claimed to be fully reconstructed by this frame-update connection.
