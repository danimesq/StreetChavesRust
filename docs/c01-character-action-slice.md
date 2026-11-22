# 🌈 C01 character-action slice: source contract

## 🌈 Purpose

Define the first bounded character-specific action-script slice for the Rust runtime, using Chaves (fighter 1, C01) as the reference. This document records source behavior and an implementation boundary; it does **not** claim that C01's [Acao] has been ported.

Normative source: [OpenStreetChaves C01.rot](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot), pinned at e1b025c35ecbc2b14d8635564aafd23c2107d3d1.

The character-specific [Acao] runs after Comum.rot [Acao]. The full action composition then runs the character-specific section before the common [AcaoFim]. Keep this order explicit in the runtime.

## 🌈 Ordered command families

The source contains seven independent top-level if statements. They are evaluated in source order, not as an exclusive match. A command can only affect the state when its own condition passes; do not silently turn the sequence into first-match dispatch.

| Command | Eligible current actions | State writes |
|---|---|---|
| cmVitoria | idle, walking, crouching | action = victory; action frame = 0; duration = 250 |
| cmPose | idle, walking | action = pose; action frame = 0; duration = 20 |
| cmEsp1Fraco/Forte | idle, walking, standing defense, low defense; effect type 7 for this player must not exist | action = special 1; frame = 0; duration = 28; add 5 × charge gain when below max; set weak/strong force |
| cmEsp2Fraco/Forte | idle, walking, standing defense, low defense | action = special 2; frame = 0; add 5 × charge gain when below max; duration 16/27; set weak/strong force |
| cmEsp3Fraco/Forte | idle, walking, jump, low defense, special 2; additionally ChaoY - 90 <= P.Y | action = special 3; add 6 × charge gain when below max; snap to ground; clear airborne/vertical/jump/directional-sequence state; duration 10/16; set weak/strong force |
| cmEsp4Fraco/Forte | idle, walking, standing defense, low defense | action = special 4; frame = 0; add 5 × charge gain when below max; duration = 22; set weak/strong force |
| cmEspFinal | idle, walking, strong punch, crouching, standing defense, low defense; and either health <= 15 or power > 0 | action = special 5; frame = 0; duration = 38; clear charge when power is exactly 3; decrement power; set sprite frame to displacement + 36; force = special; request screen flash 22 and final-special sound |

Names in the table are semantic descriptions of the Pascal enum values, not Rust enum variants. The Rust implementation must use its own typed identifiers and map them explicitly.

## 🌈 Important semantics to preserve

- **Ordered independent conditions:** retain the source's sequential if evaluation. Do not replace it with an exclusive dispatcher.
- **Effect ownership gate:** special 1 checks for effect type 7 owned by the current player before starting.
- **Charge arithmetic:** the source checks P.CargaPoder < CargaMax before adding the gain. It does not clamp the result in that statement.
- **Weak/strong selection:** the source sets force in separate conditional statements after the action setup.
- **Special 3 landing:** this path changes several movement and command-sequence fields together. A partial implementation must not claim the complete transition.
- **Final move power:** the condition permits the move at low health even when power is zero; the source then decrements power. Preserve the observed rule and investigate the field's signedness/type before implementing it.
- **Side effects:** flash, sound, effect creation, screen shake, opponent invulnerability, pushback and RNG belong to other script sections and must not be inferred from the action-start section.
- **No fabricated fallback:** missing or unsupported script semantics must fail or remain explicitly unimplemented, never be filled with guessed values.

## 🌈 Recommended implementation boundary

The first executable increment should cover only cmVitoria and cmPose, because their visible [Acao] branches have bounded state writes and no calls to effects, RNG, physics helpers or global presentation state.

That increment still requires:
1. A typed command/action mapping grounded in the existing Rust model.
2. Exact eligible-action predicates.
3. Action-frame reset and duration writes.
4. Tests for eligible and ineligible starting actions.
5. A source-order test showing that the character script runs between common [Acao] and common [AcaoFim].

Special commands should be added in separate increments. Special 1 requires effect lookup; special 3 requires the complete movement/sequence state; special final requires verified power-field semantics and global flash/sound event handling.

## 🌈 Out of scope

This contract does not port C01 [Quadro], [IA] or [Acerto]; does not port any other fighter; does not interpret arbitrary Pascal-like .rot; and does not establish original-executable equivalence.

The current shared-RNG child callback remains an injection seam. Historical Pascal evidence of a child script consuming RNG is still required before claiming nested-script RNG parity.

## 🌈 Validation checklist

- [ ] Verify the Rust action/command identifiers against the pinned Pascal enums.
- [ ] Implement only the bounded victory/pose increment first.
- [ ] Test every eligible and ineligible source action.
- [ ] Verify common-action -> C01-action -> common-action-end ordering.
- [ ] Capture a Pascal scenario for at least one C01 character-specific action and compare the resulting state.
- [ ] Keep unsupported special commands explicit until their dependencies are implemented.
