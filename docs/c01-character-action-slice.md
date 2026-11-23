# 🌈 Character-script parity: 15-fighter contract

## 🌈 Scope and roster

This PR is the consolidated workstream for character-specific combat scripts across the complete original 15-fighter roster, not a C01-only milestone.

The original roster is C01-C15: Chaves, Seu Madruga, Chiquinha, Quico, Dona Florinda, Prof. Girafales, Sr. Barriga, Bruxa do 71, Paty, Nhonho, Godines, Sr. Furtado, Jaiminho, Dona Neves and Glória. Pascal C16 (Chapolin) is a generated bonus character and remains outside the original 15-fighter completion criterion; preserve it as a separately tracked optional 2026 bonus.

Normative Pascal source revision: OpenStreetChaves e1b025c35ecbc2b14d8635564aafd23c2107d3d1. The roster inventory is documented in [fighter-roster-matrix.md](fighter-roster-matrix.md). File/section presence does not prove identical behavior.

## 🌈 Consolidated objectives for this PR

This PR intentionally absorbs the previously proposed follow-up work. Do not split these into separate PRs #11, #12 or #13:

1. **Character-script contract for all 15 fighters:** inspect and document each C01-C15 [Acao], [Quadro], [IA] and optional [Acerto] behavior, including character-specific differences, dependencies, source ordering and missing sections. Do not infer behavior from another fighter.
2. **Victory and pose implementation:** implement the source-backed victory and pose behavior for every fighter that actually defines the corresponding command, with per-character eligibility and state-write tests. Do not assume identical predicates or durations.
3. **Special-command implementation:** expand character-specific specials across the roster, incrementally and only where each source branch and its dependencies are understood. Preserve effect ownership, charge arithmetic, weak/strong variants, movement/sequence transitions, power semantics, global presentation events and RNG consumption exactly where the source requires them.
4. **Pascal child-script RNG capture:** add a real historical Pascal capture in which a parent action script invokes a child script that consumes the shared RNG stream; verify the cursor continues at the exact subsequent index and compare the resulting state/draw sequence against Rust.

These are all part of PR #10's intended scope. Completion is not implied by this initial contract update; keep the PR open until the implementation and evidence are actually present and validated.

## 🌈 Execution strategy

- Build a per-fighter source matrix before generalizing any behavior. Track each command, eligibility predicate, state mutation, script call, side effect, RNG draw and evidence scenario independently.
- Use shared Rust helpers only after at least two source implementations demonstrate the same semantics. Keep character-specific differences explicit in data or code.
- Preserve Pascal evaluation order. Independent sequential if statements must not be rewritten as an exclusive match unless the source proves exclusivity.
- Keep common script order explicit: common [Acao], character [Acao], then common [AcaoFim], with any other ordering verified from the actual caller.
- Add focused tests per fighter and command, plus integration/replay fixtures. A passing generic helper test is not evidence that all 15 source scripts match.
- Unsupported commands or missing source sections remain explicitly unimplemented. Never synthesize a missing [Acerto] or other section.
- Keep Chapolin C16 outside the 15-fighter acceptance gate; its bonus status and provenance remain separately documented.

## 🌈 C01: first inspected reference, not the whole roster

C01 (Chaves) is the first detailed source inspection and a reference for building the audit format. It is not a template that can be copied blindly to C02-C15.

The C01.rot [Acao] section contains seven independent top-level command branches: cmVitoria, cmPose, cmEsp1Fraco/Forte, cmEsp2Fraco/Forte, cmEsp3Fraco/Forte, cmEsp4Fraco/Forte and cmEspFinal. Their eligibility predicates and state writes are recorded below as C01-specific observations only.

| C01 command | Eligible current actions | Observed state writes |
|---|---|---|
| cmVitoria | idle, walking, crouching | action = victory; action frame = 0; duration = 250 |
| cmPose | idle, walking | action = pose; action frame = 0; duration = 20 |
| cmEsp1Fraco/Forte | idle, walking, standing defense, low defense; effect type 7 for this player must not exist | action = special 1; frame = 0; duration = 28; add 5 × charge gain when below max; set weak/strong force |
| cmEsp2Fraco/Forte | idle, walking, standing defense, low defense | action = special 2; frame = 0; add 5 × charge gain when below max; duration 16/27; set weak/strong force |
| cmEsp3Fraco/Forte | idle, walking, jump, low defense, special 2; additionally ChaoY - 90 <= P.Y | action = special 3; add 6 × charge gain when below max; snap to ground; clear airborne/vertical/jump/directional-sequence state; duration 10/16; set weak/strong force |
| cmEsp4Fraco/Forte | idle, walking, standing defense, low defense | action = special 4; frame = 0; add 5 × charge gain when below max; duration = 22; set weak/strong force |
| cmEspFinal | idle, walking, strong punch, crouching, standing defense, low defense; and either health <= 15 or power > 0 | action = special 5; frame = 0; duration = 38; clear charge when power is exactly 3; decrement power; set sprite frame to displacement + 36; force = special; request screen flash 22 and final-special sound |

Important C01-only semantics:
- Conditions are independent and evaluated in source order, not an exclusive dispatcher.
- Special 1 checks for effect type 7 owned by the current player.
- Charge is incremented only after checking P.CargaPoder < CargaMax; that statement does not clamp the result.
- Special 3 changes multiple movement and command-sequence fields together.
- The final special can pass at low health with zero power, after which the source decrements power. Verify the Rust field's signedness and exact source type before implementing.
- Flash, sound, effect creation, screen shake, opponent invulnerability, pushback and RNG must be traced to their actual script sections; do not infer them from the action-start branch.

## 🌈 Roster audit checklist

Each fighter must be audited and validated independently. Mark an item complete only with source evidence and tests/replay where applicable.

| Fighter | Character | Source audit | Victory/pose | Specials | Pascal replay |
|---:|---|---|---|---|---|
| C01 | Chaves | [ ] | [ ] | [ ] | [ ] |
| C02 | Seu Madruga | [ ] | [ ] | [ ] | [ ] |
| C03 | Chiquinha | [ ] | [ ] | [ ] | [ ] |
| C04 | Quico | [ ] | [ ] | [ ] | [ ] |
| C05 | Dona Florinda | [ ] | [ ] | [ ] | [ ] |
| C06 | Prof. Girafales | [ ] | [ ] | [ ] | [ ] |
| C07 | Sr. Barriga | [ ] | [ ] | [ ] | [ ] |
| C08 | Bruxa do 71 | [ ] | [ ] | [ ] | [ ] |
| C09 | Paty | [ ] | [ ] | [ ] | [ ] |
| C10 | Nhonho | [ ] | [ ] | [ ] | [ ] |
| C11 | Godines | [ ] | [ ] | [ ] | [ ] |
| C12 | Sr. Furtado | [ ] | [ ] | [ ] | [ ] |
| C13 | Jaiminho | [ ] | [ ] | [ ] | [ ] |
| C14 | Dona Neves | [ ] | [ ] | [ ] | [ ] |
| C15 | Glória | [ ] | [ ] | [ ] | [ ] |

## 🌈 Validation gates

- [x] Add persistent `sprite_frame` and explicit victory animation context for sprite offset, ground and camera bounds.
- [x] Execute audited C01-C15 victory-frame branches with ordered shared RNG; C12 now creates five type-2 live effect instances.
- [x] Test C06, C09 and C14 using both possible previous sprite frames.
- [x] Test C02's one-draw behavior, C12's ordered draw sequence/movement/effects, and C08/C13 ground-relative movement.
- [ ] Complete the source audit for all 15 original fighters, recording absent sections and per-character exceptions.
- [x] Verify the implemented Rust action mapping for pose against the Pascal enum (acPose = 24); other command mappings remain to be audited.
- [x] Implement per-character victory-frame behavior and C01-C12 pose [Quadro] frame updates; C13-C15 remain unsupported for pose because their ROT files do not define `cmPose`.
- [x] Implement C01 [Quadro] specials 1-4 and final-special frame consequences with live effects, ordered RNG, movement, sound and invulnerability tests.
- [ ] Continue the per-character special-frame implementation for C02-C15 with focused state-diff and RNG tests.
- [ ] Verify common-action -> character-action -> common-action-end ordering.
- [ ] Capture and replay at least one Pascal child-script RNG consumption case; prove shared cursor continuity across parent and child.
- [ ] Add representative and edge-case Pascal/Rust state-differential fixtures; expand coverage across all 15 fighters.
- [ ] Keep unsupported branches explicitly unimplemented; do not claim complete combat equivalence until all required systems and evidence are covered.

## 🌈 Explicit non-claims

This PR does not establish full combat equivalence merely by completing the character action scripts. CameraX, RelogioParado, additional checkpoints and other combat systems remain separate outstanding systems unless explicitly added with source evidence and validation. It does not add a general arbitrary Pascal .rot interpreter.

No Pascal source or game assets are copied into this repository. The source revision is cited for reproducibility.

## 🌈 First executable implementation: victory command

The first runtime behavior landed in `src/combat_runtime.rs` as
`CombatState::apply_character_victory`.

Source audit result for the pinned C01-C15 ROT files:

- All 15 define the same `cmVitoria` eligibility predicate:
  `P.Acao in [acParado, acAndando, acAgachado]`.
- All 15 write `P.Acao := acVitoria`, `P.QuadroAcao := 0` and
  `P.DuracaoAcao := 250`.
- The runtime applies only those three source-backed writes. It does not clear
  unrelated fighter state or consume RNG.
- Tests cover all 15 character IDs, each eligible starting action, rejected
  action states, and an invalid fighter index.

This is a bounded implementation of the character [Acao] command entry only.
It does **not** implement the per-character victory animation in [Quadro].
Those animation branches differ across the roster and remain independently
audited. Pose is also not included in this first code slice; its presence and
frame behavior must be handled per source file rather than assumed for C13-C15.

## 🌈 Victory animation audit: C01-C15

The complete character [Quadro] victory branches were inspected at the pinned Pascal revision. They are not interchangeable:

| Fighter | Frame selection / transition | Additional behavior |
|---|---|---|
| C01 | Frames 33/34/35 at action frames 0/7/14/21/28; resets action frame at 28 | Loops a 28-frame sequence |
| C02 | Defaults to 33; independently draws Random(2) and selects 34 when zero | One RNG draw on each victory-frame update |
| C03 | 33 for frames <5; then 34/35/34/33/34/35/34 at 5/10/15/20/25/30/35 | Eight explicit frame boundaries |
| C04 | 33 while frame <13; 34 at frame 13 | Two-stage sequence |
| C05 | 33 while frame <5; alternates 34/33 at 9/13/17/21/25/29/33/37; 35 at 41 | Nonuniform 4-frame transitions |
| C06 | Every fourth action frame toggles between 33 and 34 based on the current sprite frame | Depends on prior P.Quadro |
| C07 | 33 on even action frames, 34 on odd action frames | Alternating every frame |
| C08 | Sprite frame 33 throughout | Moves Y by -4/+4 according to parity, then clamps to ChaoY |
| C09 | Every third action frame toggles between 33 and 34 based on current sprite frame | Depends on prior P.Quadro |
| C10 | 33 while frame <12; 34 at frame 12 | Two-stage sequence |
| C11 | Sprite frame 33 throughout | No extra write in this branch |
| C12 | 33/34 according to parity | Moves X by 12 according to facing, clamps facing at camera-relative bounds, and may consume Random(2) when frame mod 8 = 0 to reverse direction |
| C13 | Sprite frame 33 throughout | Cosine-based Y motion with a floor clamp at ChaoY - 10 |
| C14 | Every third action frame toggles between 33 and 34 based on current sprite frame | Same frame rule as C09, independently confirmed in C14 |
| C15 | 33 while frame <=10; then 34 at frame 10; 35 at frame 14 | Independent source-ordered ifs mean frame 10 ends on 34 |

The C02 and C12 branches consume RNG from the shared Pascal stream. C12 also moves the fighter, changes facing at camera-relative thresholds and now allocates five type-2 instances in the live effect arena. SDL rendering and comparison against a captured Pascal victory trace remain outstanding.

## 🌈 Verified pose enum and first runtime implementation

The numeric mapping is grounded in the original source/game/ChavesTipos.pas at source revision e1b025c35ecbc2b14d8635564aafd23c2107d3d1:

- acVitoria = 23
- acPose = 24

C01-C12 each define cmPose with the same guard (P.Acao in [acParado, acAndando]) and writes (P.Acao := acPose, P.QuadroAcao := 0, P.DuracaoAcao := 20). C13-C15 do not define cmPose. Rust now exposes ACTION_POSE = 24 and CombatState::apply_character_pose, which applies that branch only to C01-C12. Tests cover all 12 eligible character IDs, both allowed starting actions, rejected actions, C13-C15 and invalid fighter indices.

This implements the [Acao] command entry only. The per-character [Quadro] pose frame assignments are now implemented by `CombatState::update_pose_animation`: frame `Desloc + 12` at action frame 0 and `Desloc + 30` at action frame 2, retaining the previous sprite frame at other action frames. Tests cover both boundaries and retention for all C01-C12.


## 🌈 Victory animation runtime: state and execution contract

The victory audit is not implemented as a per-character frame-number lookup. The Pascal [Quadro] branches read and mutate more than the action timer, and several branches depend on values that are currently absent from `FighterState` or `CombatState`.

### Runtime state that must become explicit

| Pascal concept | Required Rust representation | Why it is not interchangeable |
|---|---|---|
| `P.Quadro` | Persistent `sprite_frame` on each fighter | C06, C09 and C14 inspect the existing sprite frame before deciding whether to toggle it. |
| `P.X`, `P.Y` | Existing fighter coordinates, with source-ordered mutation | C08, C12 and C13 move the fighter as part of the victory frame update. |
| `P.Lado` | Existing `facing`, with explicit transition semantics | C12 can reverse facing after movement and a conditional RNG draw. |
| `ChaoY` | Explicit ground reference in the animation update context | C08 clamps to the ground; C13 clamps against a ground-relative offset. |
| `CameraX` and screen limits | Explicit camera/bounds context, not an assumed constant | C12's horizontal clamp is camera-relative. |
| Pascal `Random(2)` | The caller's existing mutable `PascalRandomSource` | C02 and C12 must advance the same replay cursor used by the rest of the tick. |

The current AI replay snapshots do not contain `P.Quadro`, `ChaoY` or `CameraX`. The new animation API therefore requires the caller to provide a source-backed `VictoryAnimationContext`; tests supply explicit context values, not values inferred from those AI snapshots. Captured Pascal victory-frame checkpoints are still needed for end-to-end state differential.

### Ordered update contract

The runtime entry point should be a fallible operation that receives the fighter index, the current animation context, and the caller's mutable Pascal RNG source. It should execute the fighter's [Quadro] branch in Pascal statement order and return an ordered effect/event trace for assertions and differential diagnostics.

The operation must preserve these rules:

1. Read the pre-update action frame, sprite frame, position and facing exactly where the Pascal statement reads them.
2. Apply each assignment immediately in source order. Do not turn independent Pascal `if` statements into an exclusive `match`; C15 demonstrates why the final write at a shared boundary matters.
3. Request RNG only at the exact source statement that requests it. C02 draws on every victory-frame update; C12 draws only on its audited frame-modulo boundary. A branch that does not draw must not advance the cursor.
4. Apply movement, facing changes and clamps in their original order. Do not calculate a final position from a simplified direction enum if the source mutates intermediate values.
5. Keep animation frame writes distinct from action-frame advancement. The caller owns the tick/update cadence; this function must not invent an extra increment or reset.
6. Return an explicit unsupported-input/error result when required context is unavailable. Never report a successful frame update after skipping a source-visible side effect.

### Incremental implementation boundary

The runtime now receives the explicit context and executes the previously blocked movement/RNG branches. C12's five type-2 effects are allocated in the live arena; the new C01 frame runtime also allocates types 2, 7, 8 and 16. The arena produces ordered snapshots for the future renderer, while SDL drawing and full collision consequences remain pending.

C06, C09 and C14 are tested with both possible prior sprite frames and with a non-boundary action frame to verify that they neither overwrite the prior value nor consume RNG when the source condition is false.

The C01-C15 victory [Quadro] branch behavior is now implemented in the runtime and covered by focused tests. Full Pascal parity is not claimed until historical captures include the relevant sprite-frame, movement and effect checkpoints.


## 🌈 Executable victory animation runtime slice

Commit implementation adds persistent `FighterState::sprite_frame`, explicit
`VictoryAnimationContext` (sprite offset, ground Y, camera origin and camera-relative
turn limits), and `CombatState::update_victory_animation`.

The update preserves the audited statement order for C01-C15. C02 consumes exactly
one shared `Random(2)` draw per victory update. C12 moves by 12, applies camera-relative
facing thresholds, conditionally consumes its `Random(2)` draw at action-frame
multiples of eight, then consumes the ten ordered particle-coordinate draws
(`Random(40)`, `Random(10)` repeated five times). Particle spawn requests are returned
as ordered data so the effect renderer can consume them without hiding or reordering
RNG calls. C08 and C13 now receive explicit ground context; C13 uses ties-to-even
Pascal rounding for its cosine displacement.

C06, C09 and C14 branch on the previous sprite frame. Tests must seed both 33 and 34
and assert the opposite frame on each eligible action-frame boundary. This is an
animation update API, not a full renderer/effect-system integration claim.


Validation note: GitHub Actions passed formatting and `cargo test --all-targets` on commit `9f335d9cd23665277081bc17a685b2adac60213d`. The update API preserves the shared RNG cursor; it does not claim renderer integration for C12's returned particle events.


## 🌈 Pose animation runtime slice

`CombatState::update_pose_animation(fighter_index, sprite_offset)` now executes the identical C01-C12 `[Quadro]` pose branch found in the pinned Pascal ROT files. It writes `sprite_offset + 12` when `QuadroAcao = 0`, then independently writes `sprite_offset + 30` when `QuadroAcao = 2`; all other action frames retain the previous `P.Quadro`. It rejects non-pose actions, invalid fighter indices, and C13-C15. Tests exercise both writes and the no-write interval for every supported fighter.

## 🌈 Live effect-instance foundation

The Rust core now owns a fixed-capacity `EffectArena` with 80 ordered slots. Allocation uses the first free slot, reuses the first freed slot, and returns no effect when full. The full-pool path exits before type configuration and therefore consumes no RNG, matching the Pascal allocator's order.

The first source-backed configurations are the C01-used effect types 2, 7, 8 and 16 from the pinned `Dados/Efeitos.ini`:

- Type 2: 15-frame dust/cloud effect with the source bottom-anchor flag and ordered sprite changes; its animation also applies the one-pixel upward motion when not frozen.
- Type 7: weak/strong C01 Esp1 projectile, with owner-facing, hitbox, damage, guard damage and the `1 em 5` variant roll. That roll uses the shared Pascal LongInt RNG cursor only after a free slot has been found.
- Type 8: C01 final-special hit effect with the source damage, guard damage, terminating/variant flags and frame-dependent hitbox phases.
- Type 16: short-lived C01 Esp2 trail with owner-facing and source sprite timing.

C12 victory no longer returns detached particle requests. It allocates five live type-2 instances in the arena and returns the allocated slot indices. The new C01 frame executor is separate from command entry: it applies the [Quadro] branches for specials 1-4 and the final special, creates the source effect types, writes pending sounds, moves the fighter, updates opponent invulnerability, and exposes the strong Esp2 [Acerto] recoil hook.

`tick_effects` advances supported live instances in slot order and returns immutable render snapshots before lifetime/culling cleanup, preserving the source's same-tick draw behavior. This is a simulation-side foundation, not a finished renderer: SDL sprite drawing, effect-vs-effect collision, effect-vs-fighter collision consequences, and the remaining effect-type catalog are still outstanding.

## 🌈 Pascal capture and nested RNG evidence boundary

The existing Pascal runtime fixtures capture AI tick/script records and draw segments, not a victory `[Quadro]` execution with `P.Quadro`, movement, `CriaEfeito` instances, or a real parent action invoking a child action script. The Rust child-script seam already passes the same mutable RNG cursor into the child, but a synthetic child test is not a Pascal capture. Real capture and differential checks remain pending until the Pascal harness records those events from an actual execution.
