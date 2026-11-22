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

- [ ] Complete the source audit for all 15 original fighters, recording absent sections and per-character exceptions.
- [ ] Verify Rust action/command identifiers and mappings against the Pascal enums.
- [ ] Implement victory and pose per source-defined fighter behavior; test eligible and ineligible states.
- [ ] Implement specials incrementally, with focused state-diff and RNG tests for every supported branch.
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

The C02 and C12 branches consume RNG from the shared Pascal stream. The C12 branch also has movement, camera-boundary and facing side effects. These are specifically **not** implemented by the pose command or by the existing victory-entry helper. Victory animation runtime work remains pending until the frame-update contract can represent sprite-frame state, position/facing side effects and shared RNG without losing source ordering.

## 🌈 Verified pose enum and first runtime implementation

The numeric mapping is grounded in the original source/game/ChavesTipos.pas at source revision e1b025c35ecbc2b14d8635564aafd23c2107d3d1:

- acVitoria = 23
- acPose = 24

C01-C12 each define cmPose with the same guard (P.Acao in [acParado, acAndando]) and writes (P.Acao := acPose, P.QuadroAcao := 0, P.DuracaoAcao := 20). C13-C15 do not define cmPose. Rust now exposes ACTION_POSE = 24 and CombatState::apply_character_pose, which applies that branch only to C01-C12. Tests cover all 12 eligible character IDs, both allowed starting actions, rejected actions, C13-C15 and invalid fighter indices.

This implements the [Acao] command entry only. The per-character [Quadro] pose frame assignments (frame 12 at action frame 0 and frame 30 at action frame 2 in C01-C12) remain a separate animation-update task.
