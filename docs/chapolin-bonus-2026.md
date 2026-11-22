# Chapolin: recoverable 2026 bonus

## Decision

Preserve Pascal C16 (Chapolin) as a **future optional bonus for the 2026 project**, not as a sixteenth member of the original roster and not as a required part of the first Rust implementation.

The original/base roster remains the 15 characters represented in both SDL references. Chapolin is a separately documented Pascal-side extension. This is a scope decision to retain the possibility of recovering him later; it is not a promise that he will ship in a release.

## Why preserve him?

- He already exists in the OpenStreetChaves Pascal reference as C16.
- His INI/ROT pair documents a distinct character configuration and behavior.
- His INI header identifies generated art and `CentroX`, `Corpo`, and `Ataque` data, with `source/tools/GeraChapolin` named as generator.
- His documented CPU tuning differs from the original roster (including walk speed 9.0, stronger attacks, and more frequent special use).
- The C/SDL3 reference does not include C16, so a port cannot treat it as shared/reference-confirmed content.

## Recovery plan (future, not current implementation)

1. **Provenance and permission review**: inspect the generator, source inputs, art origin and licenses. Do not copy generated assets/data into this repository until the status is understood and allowed.
2. **Availability research**: identify which original Pascal modes/screens expose C16 and whether this is a deliberate extension, debug option, or special opponent. Record evidence rather than assume.
3. **Behavioral capture**: collect move, CPU, animation, hitbox, effect and sound observations for C16. Mark source-derived vs original-executable-confirmed findings.
4. **Optional content boundary**: define a bonus/extension registration path that cannot alter the default 15-fighter roster, IDs, campaign logic, or compatibility assumptions.
5. **Data recovery**: recover only the documented/legitimate data needed for a compatible implementation; prefer independently authored Rust data from verified observations where reuse is not appropriate.
6. **Tests**: ensure base-roster tests run without bonus content; bonus tests are isolated and opt-in.
7. **Release decision**: if recovered and cleared, present Chapolin as optional bonus content with attribution/provenance and compatibility notes. Otherwise retain this plan and inventory as research.

## Non-goals

- Do not renumber the original 15 fighters or add Chapolin to their canonical roster.
- Do not invent a sixteenth original fighter or infer that C's array dimension means one.
- Do not import art, audio, scripts, or generated data blindly.
- Do not promise release inclusion, game-mode access, or legal clearance.
- Do not let bonus content delay the base roster or Phase 2 data-model research.

## Evidence snapshot

- Pascal reference revision: `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`.
- C reference revision: `e76ad2188af6842ff6618ecdb8f425bba61c9183`.
- Inventory and representation details: [fighter data deep dive](fighter-data-deep-dive.md).
- File coverage: [fighter roster matrix](fighter-roster-matrix.md).

Treat this document as a preservation/recovery note. The bonus remains planned and unimplemented until future work explicitly begins.
