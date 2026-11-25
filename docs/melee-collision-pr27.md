# PR #27: Melee collision detection

## Scope

This PR adds renderer-free, deterministic melee contact detection for the two fighters. It does not apply damage, resolve guard, consume RNG, spawn effects, alter action states, or execute character-specific Acerto scripts. Those ordered consequences remain a separate follow-up so all 15 fighter scripts stay authoritative.

## Source contract

The implementation follows Pascal TJogo.L7_Acertos from the pinned OpenStreetChaves revision:

- Attack world X: Round(P.X) + P.AtaqueX1/X2.
- Attack world Y: Round(P.Y) - P.AlturaSprite + P.AtaqueY1/Y2.
- Body world X: Round(O.X) + O.CorpoX1/X2.
- Body world Y: Round(O.Y) - O.AlturaSprite + O.CorpoY1/Y2.
- Contact requires P.GolpeAtivo, a non-invulnerable defender, strict AABB overlap, no hit-stop and no screen flash.
- Edge contact alone is not a hit.
- Directional checks run P1 attacking P2, then P2 attacking P1.

Pascal Round uses ties-to-even behavior. The Rust projection helper preserves it rather than relying on Rust's default half-away-from-zero f64::round.

## Architecture

- lib.rs exports the new melee_collision module.
- melee_collision.rs contains pure geometry projection and contact detection.
- detect_fight_melee_contacts accepts the two live FighterState values and returns ordered collision events.
- No SDL, WASM, canvas, audio, renderer, or Foundation dependency is introduced.

## Explicit source-data boundary

FighterState does not yet carry Pascal AlturaSprite. The detector therefore requires both sprite heights as explicit inputs. This avoids inventing a universal height or silently producing incorrect world-space Y coordinates. A later data-integration step can source these values from fighter sprite metadata and schedule detection in the authoritative tick phase.

The selected hitbox row's active metadata remains distinct from fighter-level attack_active / Pascal GolpeAtivo. Eligibility follows the latter, matching L7_Acertos. The row metadata remains available on the collision event.

## Tests

- World-space projection and metadata preservation.
- Strict edge-touch rejection.
- Attack-active, invulnerability, hit-stop and screen-flash gates.
- Pascal ties-to-even rounding, including negative coordinates.
- Non-finite coordinate rejection.

## Deferred to PR #28+

Guard outcomes, damage/chip damage, combo counters, character-specific Acerto rules, knockback, hit-stop RNG, effects, sounds, and chained special moves are deliberately not guessed or folded into this detector.
