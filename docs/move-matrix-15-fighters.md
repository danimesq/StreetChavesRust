# Move matrix: 15-fighter research

## Purpose and evidence rules

This document is the working move-by-move specification for the original 15-fighter roster. It is intentionally incomplete and will grow fighter by fighter. It does **not** claim that either SDL reconstruction is the original game's authoritative behavior.

Evidence labels:
- **[Pascal source]**: directly read from the pinned Pascal INI/ROT source.
- **[C reference]**: directly read from pinned C source/spec/table documentation. No C per-fighter comparison is claimed in this initial entry.
- **[Inference]**: interpretation of source behavior; needs tests.
- **[Original validation pending]**: not confirmed against the original executable.

Pinned Pascal reference: `OpenStreetChaves`, commit `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`.
Pinned C reference from roster research: `open-street-chaves`, commit `e76ad2188af6842ff6618ecdb8f425bba61c9183`.

## Matrix format

Each move record should capture: fighter ID/name; move name and source ID; input/command recognition; command-state progression and reset; eligible starting states; transition action; action timer/duration; energy/meter effects and conditions; sprite frames and timing; hitbox coordinates, activation, strength and damage; guard damage; effects; sounds; CPU decision conditions; source location and exact evidence; cross-port differences; original-executable validation status; unresolved questions.

Do not treat sprite-frame number, action timer, state ID, or command-sequence progress as interchangeable. Coordinates and values remain in the source representation until their semantics are independently confirmed.

## Fighter 01 — Chaves

### Fighter metadata

| Field | Value | Evidence |
|---|---|---|
| ID | C01 | [Pascal source] filename |
| Sprite height | 156 | [Pascal source] `[Geral]` |
| Walk speed | 8.0 | [Pascal source] `[Geral]` |
| Campaign boss | ID 2 | [Pascal source] `[Geral]` |
| Excluded opponents | IDs 3, 9 | [Pascal source] `[Geral]` |
| Stage | 1; no alternate/rare stage configured | [Pascal source] `[Geral]` |
| CPU music | 4 | [Pascal source] `[Geral]` |

### Special-command recognizer

**[Pascal source]** Commands are processed in ordered rules, per frame. The sequence state resets after nine frames without advancing. Direction labels are relative to opponent-facing where documented.

| Step | Condition | Transition |
|---:|---|---|
| 1 | Down; not Forward; not Back | 0 → 1 |
| 2 | Down; not Forward; not Back | 1 → 1 |
| 3 | Down; not Forward; not Back | 2 → 1 |
| 4 | Forward; not Down | 1 → 2 |
| 5 | Back; not Down | 1 → 3 |
| 6 | Up; not Down; Down-hold ≥ 10; grounded | 0 → 4 |
| 7 | Up; not Down; Down-hold ≥ 10; grounded | 1 → 4 |
| 8 | Back; not Down | 1 → 3 |
| 9 | Down; not Forward; not Back | 3 → 5 |
| 10 | Forward; not Down; not Back | 5 → 6 |
| 11 | Down; Forward; not Back | 3 → 6 |
| 12 | Down; Forward; not Back | 5 → 6 |

The source defines four special families through button rules. Matching rules are processed in order; source comments say all matching rules execute.

| Family | Command progress | Button | Strength | Evidence |
|---|---:|---|---|---|
| Special 1 | 2 | Punch | Weak/strong | [Pascal source] |
| Special 2 | 3 | Punch | Weak/strong | [Pascal source] |
| Special 3 | 4 | Punch | Weak/strong | [Pascal source] |
| Special 4 | 3 | Kick | Weak/strong | [Pascal source] |
| Final special | 6 | Punch | Weak/strong | [Pascal source] |

### Special move behavior

These are source-observed script rules, not original-game verified timings.

| Move | Allowed start states (Pascal) | Duration | Meter change | Strength | Source observation |
|---|---|---:|---|---|---|
| Special 1 | idle, walking, guard, low guard | 28 | +5 × charge gain if below max | weak/strong | Rejected while effect type 7 for this fighter exists |
| Special 2 | idle, walking, guard, low guard | 16 weak / 27 strong | +5 × charge gain if below max | weak/strong | Distinct duration by strength |
| Special 3 | idle, walking, jump, low guard, Special 2; additionally ground proximity check | 10 weak / 16 strong | +6 × charge gain if below max | weak/strong | Snaps to ground; clears airborne/jump/sequence state and vertical velocity |
| Special 4 | idle, walking, guard, low guard | 22 | +5 × charge gain if below max | weak/strong | Same duration for both strengths |
| Final special | idle, walking, strong punch, crouch, guard, low guard; requires life ≤ 15 OR power > 0 | 38 | decrements power; if power was 3, resets charge | special | Sets frame offset +36; screen flash; pending final-special sound |

Meter values are source expressions, not guaranteed absolute meter amounts; the charge-gain constant and cap are defined elsewhere.

### Frame/tick script observations

**Special 1**
- Action starts at frame 0, selects sprite offsets +36, +37, +38 at action timers 0, 4, 7; timer 10 selects +39.
- At timer 10, creates effect type 7 with variant A for weak or B for strong, positioned ±45 relative to fighter X based on facing.
- Pending sound ID 14 at timer 10.

**Special 2**
- Timer 0: pending sound 15 and sprite offset +40.
- Timer 6 weak / timer 10 strong: offset +41.
- Timer 12 weak / timer 19 strong: offset +42.
- When current sprite frame is +41, creates three type-2 effects at randomized positions. Weak pushes opponent 19; strong pushes 21, direction adjusted by facing.
- From timer ≥9, creates effect type 16 at ±60 X and Y−85.
- Marks the move as a kick in the script.

**Special 3**
- Pending sound 16 at timer 0.
- Sprite toggles +43/+44 every third action timer.
- Moves Y upward by 10 per script update and pushes horizontally 5 toward facing.
- At timer 20, sets airborne flag.
- Opponent invulnerability toggling depends on hit count and timer: source conditions include hit count ≥1 clearing invulnerability, hit count ≥4 setting it, and timer >7 setting it. Ordering matters and should be tested.
- Marks the move as a kick.

**Special 4**
- Pending sound 17 at timer 0; sprite offsets +45, +46, +47, +48, +49, then +12 at timers 0, 4, 8, 12, 15, 18.
- At timers 4–13, pushes opponent 17 weak / 21 strong in facing direction.
- Marks kick only from timer ≥12.

**Final special**
- Sprite offsets +36, +37, +38 at timers 0, 3, 6; timer 9 selects +39.
- At timer 9, pending sound 18, creates effect type 8 variant A at ±110 X, and screen shake 20.

### Hitbox data (Pascal `[Ataque]`)

The INI key is `action.frame`, optionally suffixed `.f1` or `.f2` for weak/strong. Values are `x1,y1,x2,y2,active,strength,damage,guard_damage`. These are source coordinates; coordinate orientation, frame/state mapping, and original-game correspondence require separate validation.

| Key | Box | Active | Strength field | Damage | Guard damage |
|---|---|---:|---:|---:|---:|
| 3.2 | 15,47,63,57 | 1 | 1 | 7 | 0 |
| 4.2 | 25,84,84,124 | 1 | 1 | 7 | 0 |
| 5.5 | 25,46,65,58 | 1 | 2 | 14 | 0 |
| 6.5 | 25,82,83,107 | 1 | 2 | 14 | 0 |
| 8.2 | 15,67,75,82 | 1 | 1 | 7 | 0 |
| 9.2 | 10,115,70,155 | 1 | 1 | 7 | 0 |
| 10.5 | 15,67,70,82 | 1 | 2 | 14 | 0 |
| 11.5 | 5,95,98,113 | 1 | 2 | 14 | 0 |
| 13.2 | 15,60,80,94 | 1 | 1 | 7 | 0 |
| 14.2 | 10,75,75,97 | 1 | 1 | 7 | 0 |
| 15.5 | 15,60,80,94 | 1 | 2 | 14 | 0 |
| 16.5 | 10,75,75,97 | 1 | 2 | 14 | 0 |
| 26.6.f1 | 40,73,100,86 | 1 | 0 | 16 | 2 |
| 26.10.f2 | 40,73,100,86 | 1 | 0 | 20 | 3 |
| 27.0 | -32,5,32,125 | 1 | 0 | 10 | 2 |
| 27.2 | -32,5,32,125 | 1 | 0 | 6 | 2 |
| 27.4 | -32,5,32,125 | 1 | 0 | 4 | 2 |
| 27.7 | -32,5,32,125 | 1 | 0 | 2 | 2 |
| 28.6 | -30,10,35,70 | 1 | 0 | 10 | 3 |
| 28.9 | -30,10,35,70 | 1 | 0 | 7 | 2 |
| 28.12 | -10,50,45,100 | 1 | 0 | 5 | 1 |

**Unknowns:** action IDs 3–28 have not been mapped completely to named moves/normal attacks here; exact hitbox activation against the original executable is pending. The data entries are recorded as-is without assuming all rows are special attacks.

### CPU decision rules (Pascal `[IA]`)

Script runs only when not defending. Random tests are conditioned on distance/threat and scaled by `Fator`. `Random(n)=0` uses source's integer random semantics; probability should not be simplified without confirming the RNG range.

| Condition | Decision | Random gate |
|---|---|---|
| Threat=1 and projectile distance >140 | Special 1 weak/strong random | 1 in Round(7×Fator) |
| Threat=0 and distance >120 | Special 1 random strength | 1 in Round(30×Fator) |
| Threat=0 and distance <200 | Special 2 random strength | 1 in Round(40×Fator) |
| Distance <150 and opponent above ground | Special 3 random strength | 1 in Round(50×Fator) |
| Threat=0 and distance <200 | Special 4 random strength | 1 in Round(40×Fator) |
| Distance <150 and opponent above ground | Final special | 1 in Round(50×Fator) |

### Hit reaction

Pascal `[Acerto]`: when Chaves's Special 2 strong hits, sets opponent recoil to 20. The script comment states hit reaction executes after default damage/reaction/push, so ordering is material.

### Validation backlog for Chaves

- [ ] Inspect corresponding C fighter/move tables and hand-coded logic; record exact commit and file/symbol refs.
- [ ] Map Pascal action IDs to normal attacks and named moves.
- [ ] Verify command progress, reset timer, and ordered button matching with focused tests.
- [ ] Verify action durations, sprite frames, hitbox activation, damage, and guard damage against original executable.
- [ ] Verify energy/power conditions and resulting values, including cap/rounding.
- [ ] Capture CPU choices with RNG state/seed and input conditions.
- [ ] Check sound/effect IDs against resource tables.
- [ ] Record any Pascal/C disagreement without treating either port as canonical.

## Remaining base roster

The remaining 14 fighters are not yet move-matrix researched. No behavior is extrapolated from Chaves.

| ID | Fighter | Status |
|---:|---|---|
| 2 | Seu Madruga | Planned |
| 3 | Chiquinha | Planned |
| 4 | Quico | Planned |
| 5 | Dona Florinda | Planned |
| 6 | Prof. Girafales | Planned |
| 7 | Sr. Barriga | Planned |
| 8 | Bruxa do 71 | Planned |
| 9 | Paty | Planned |
| 10 | Nhonho | Planned |
| 11 | Godines | Planned |
| 12 | Sr. Furtado | Planned |
| 13 | Jaiminho | Planned |
| 14 | Dona Neves | Planned |
| 15 | Glória | Planned |


---

## Cross-roster pass — all 15 fighters

This pass expands coverage beyond Chaves. It is a **source comparison**, not a claim of original-game validation. The Pascal files are data-driven INI/ROT; the C project has a decompiled command/move setup specification and generated static tables. The two models are not interchangeable.

### Pinned C evidence

C reference: `open-street-chaves`, commit `e76ad2188af6842ff6618ecdb8f425bba61c9183`.

- `docs/spec/04-moves.md`: move IDs, notation, command parser, sequence state, CPU overview.
- `docs/spec/04a-move-tables.txt`: per-character direction transitions and button/sequence-state mappings.
- `docs/spec/05b-move-setup.txt`: accepted starting states, resulting states, durations, meter multipliers, sound IDs and special conditions.
- `docs/spec/05d-cpu-table.txt`: per-character CPU decisions and probability coefficients.
- `docs/spec/05a-anim-tables.txt`: animation/hit/hurt table evidence.

The C specification labels decompiled behavior separately from static bytes and inference. Its move-start routine `0x4775fc` is identified as the entry point, but the detailed per-move frame data is still marked unknown in that reference. Do not fill this gap from the Pascal implementation.

### Base roster move-name and notation inventory

The following inventory is recorded in both Pascal `[Golpes]` and C `04-moves.md`. The notation assumes facing right in the displayed list; the runtime parser resolves forward/back relative to fighter facing.

| ID | Fighter | Special 1 | Special 2 | Special 3 | Special 4 | Super |
|---:|---|---|---|---|---|---|
| 1 | Chaves | BDP Power Wave | BEP Burn Knuckle | BCP Rising Tackle | BEK Crack Shoot | BEBDP Power Geyser |
| 2 | Seu Madruga | BDP Ryuu Geki Ken | DBDP RyuuGa | DEDK Genei Kyaku | CBEK Hien Ryuu Jin Kyaku | BDBEP Ryuko Ranbu |
| 3 | Chiquinha | BDP Ko Ou Ken | DBDP Kuu Ga | BEP Hien Senpuuken | BEK Hien Senpuukyaku | BDBEK Hien Honou Kyaku |
| 4 | Quico | BEP Hi Sho Ken | YDP Zan Ei Ken | EBDK Ku Ha Dan | BDP Sho Ryu Dan | BEBDK Cho Reppa Da |
| 5 | Dona Florinda | BDP Souhakkei | BEK Sen en Shuu | BCK Spinning Bird Kick | KR Hyakuretsu Kyaku | BDBDK Senretsu Kyaku |
| 6 | Prof. Girafales | BDP Tiger Shot | BDK Low Tiger Shot | DBDP Tiger Blow | DBDK Tiger Crush | BDBDK Tiger Cannon |
| 7 | Sr. Barriga | PR Tekkyu Dai Kaiten | EDP Tekkyu FunsaiGeki | BCK Tekkyu Hi En Zan | BCP Ricchi Futoi | BDBEP Tekkyu Dai Bouson |
| 8 | Bruxa do 71 | BDP Chaos Flare | CBDP Chaos Flare (AR) | DBDP Demon Cradle | BEK Bat Spin | BDBDK Demon Horde |
| 9 | Paty | BDK Cannon Drill | DBDK Cannon Spike | BDP Accel S. Knuckle | CBEK Cannon Strike (AR) | BDBDK Spin D. Smasher |
| 10 | Nhonho | PR Hyakuretsu Harite | EDP Super Zutsuki | BCK Super Hyakkan Otoshi | BDK Biggu Hara | EDEDP Onimussou |
| 11 | Godines | BDP Bad Stone | BEP Criminal Upper | BDK Raffian Kick | BEK Bushin Hurricane | BDBDK Dead End Irony |
| 12 | Sr. Furtado | BDP Gadouken | DBDP Kouryuken | BEK Dankuu Kyaku | CBEK Dankuu Kyaku (AR) | BDBDK Kouryuu Rekka |
| 13 | Jaiminho | BDP Psycho Shot | BDK Double Knee Press | BEP Psycho Field | DBD? Jaiminho Warp | EDEDP Psycho Crusher |
| 14 | Dona Neves | BEP Psycho Ball Attack | BEK Psycho Reflector | BEK Phoenix Arrow (AR) | DBDP Psycho Sword | BEBEP Shining Crystal Bit |
| 15 | Glória | BDP KaChoSen | BEP RyuEnBu | DBDK Hisho RyuEnJin | BDK Hissatsu Shinobibachi | BEBDK Cho Hissatsu S. |

**Notation key:** C=up, B=down, D=forward/right, E=back/left, Y/Z=diagonals, P=punch, K=kick, R=rapid mash, ?=punch or kick, (AR)=airborne. The C analysis infers Y as down-back from Zan Ei Ken; keep that as inference until independently checked.

### Pascal special-action duration inventory

Values below are `P.DuracaoAcao` assignments in each character's `[Acao]` script, in action-timer units. Where the source uses separate weak/strong branches, both values are shown. These are not necessarily equal to C's durations or original executable timing.

| ID | Fighter | Esp. 1 | Esp. 2 | Esp. 3 | Esp. 4 | Final / special |
|---:|---|---:|---:|---:|---:|---:|
| 1 | Chaves | 28 | 16 / 27 | 10 / 16 | 22 | 38 |
| 2 | Seu Madruga | 28 | 30 | 20 | 45 | 30; follow-up actions 13 and 40 |
| 3 | Chiquinha | 28 | 15 | 14 / 19 | 15 | 15; follow-up 30 |
| 4 | Quico | 28 | 13 / 16 | 15 | 15 | 13 |
| 5 | Dona Florinda | 28 | 23 | 38 / 40 | 20 | 30 |
| 6 | Prof. Girafales | 28 | 28 | 25 | 30 | 50 |
| 7 | Sr. Barriga | 17 / 20 | 15 | 15 | 15 | 12; follow-up 15 |
| 8 | Bruxa do 71 | 28 | 15 | 20 | 25 | 20 |
| 9 | Paty | 14 / 23 | 20 | 15 | 15 | 40 |
| 10 | Nhonho | 20 | 11 / 20 | 20 | 13 | 40 |
| 11 | Godines | 28 | 30 | 9 / 15 | 12 | 45 |
| 12 | Sr. Furtado | 28 | 20 | 28 | 32 | Special 4 is final-like and consumes power |
| 13 | Jaiminho | 28 | 16 | 30 | 30 | 30; Extra 2 enters a separate final action |
| 14 | Dona Neves | 28 | 35 | 35 | 35 | 70 |
| 15 | Glória | 28 | 22 | 25 | 18 | 30 |

**Important:** Jaiminho's Pascal script groups `Esp4Fraco`, `Esp4Forte`, `EspFinal` and `EspExtra1` into `acEspecial4`, while `EspExtra2` enters `acEspecial5`. Sr. Furtado's script maps `Esp4Fraco` to a power-consuming final-like action; do not force the generic five-slot model onto these exceptions.

### C move setup comparison

The C `05b-move-setup.txt` uses move IDs 16/17, 18/19, 20/21, 22/23 and 24 for the usual special slots/super, with character-specific exceptions. Durations below are decimal conversions of the values in that file. They are C reference values, not Pascal values.

| ID | Fighter | C S1 16/17 | C S2 18/19 | C S3 20/21 | C S4 22/23 | C super |
|---:|---|---:|---:|---:|---:|---:|
| 1 | Chaves | 28 | 16 | 10 | 22 | 38 |
| 2 | Seu Madruga | 28 | 30 | 20 | 45 (22 only) | 13 (23 follow-up) |
| 3 | Chiquinha | 28 | 15 | 14 | 15 | 15 |
| 4 | Quico | 28 | 13 | 15 | 15 | 13 |
| 5 | Dona Florinda | 28 | 23 | 38 | 20 | 30 |
| 6 | Prof. Girafales | 28 | 28 | 25 | 30 | 50 |
| 7 | Sr. Barriga | 17 | 15 | 15 | 15 | 12 |
| 8 | Bruxa do 71 | 28 | 15 | 20 | 25 | 20 |
| 9 | Paty | 14 | 20 | 15 | 15 | 40 |
| 10 | Nhonho | 20 | 11 | 20 | 13 | 40 |
| 11 | Godines | 28 | 30 | 9 | 12 | 45 |
| 12 | Sr. Furtado | 28 | 20 | 28 | 32 | Not represented as ordinary ID 24 in this table |
| 13 | Jaiminho | 28 | 16 | 30 | 30 | 30 (ID 26) |
| 14 | Dona Neves | 28 | 35 | 35 | 35 | 70 |
| 15 | Glória | 28 | 22 | 25 | 18 | 30 |

C table columns are a convenient alignment for the common roster, not a guarantee that all character-specific branches share the same semantic role. For Seu Madruga, Sr. Furtado and Jaiminho, the IDs and chained states require their own explicit treatment. C's move-setup table also records accepted source states, meter multipliers and sound IDs; consult the pinned file rather than inferring those from the duration summary.

### Cross-roster findings and divergences

1. **The five displayed special-move names are not enough to describe runtime behavior.** Command transitions and button maps differ per fighter. Some characters reuse a sequence state for different buttons or moves; some have extra command states.
2. **Charge and mash mechanics are character-specific.** Chaves and several fighters use directional sequences; Sr. Barriga and Nhonho have charge/mash conditions; Dona Florinda's fourth special uses repeated kick charge in Pascal; Jaiminho uses a back-charge sequence for an extra move.
3. **Air-only and state-gated moves exist.** Bruxa's second special has explicit `NoAr` button conditions and requires `acPulo` in Pascal. Dona Neves's third special is only accepted from `acPulo`. Paty's fourth special is also jump-only in Pascal.
4. **Move eligibility differs even where the displayed move name and slot align.** Pascal scripts permit some specials from guard, crouch, jump, or even a preceding special; others require a narrow state. C's accepted-state sets are separate evidence and can differ.
5. **Meter use is not uniform.** Most Pascal ordinary specials add `5 * GanhoCarga` or `6 * GanhoCarga` when below `CargaMax`; final-like actions decrement `P.Poder` and may clear `CargaPoder` when power was 3. C records multipliers such as x5, x6, x7 and x2; these must not be silently equated to Pascal's named variables.
6. **Hitbox table action IDs vary.** Pascal `[Ataque]` uses action/timer keys, including special actions such as 25–30 and character-specific rows. C generated hit tables use their own schema and IDs. Matching a row by numeric coincidence is not proof of equivalent move timing.
7. **CPU logic is character-specific.** The C CPU table records per-character distance/projectile/airborne conditions and coefficients. Pascal `[IA]` scripts are similar in broad shape but not numerically identical in every case. For example, C Chaves special 1 projectile threshold coefficient is 7, while Pascal's expression is `Round(7 * Fator)`; the C project derives its difficulty factor from game level and fight number. These are different representations and require behavioral comparison.
8. **Sound IDs and effect IDs are implementation-local.** Pascal ROT sound/effect numbers cannot be equated to C sound/effect identifiers without resource mapping.

### Initial duration discrepancy check

The duration tables reveal several exact matches and several meaningful divergences between the two reconstructions. Examples:

- Chaves: special durations align across the common slots (28, 16, 10, 22, 38).
- Seu Madruga: Pascal has special 3 duration 20; C has 20, but Pascal's special 4 is 45 and C splits move 22 (45) and move 23 (13) as a chained/follow-up path.
- Chiquinha: Pascal special 3 has weak/strong durations 14/19, while C's move 20/21 setup is 14.
- Quico: Pascal special 2 is 13/16 while C lists 13 for the paired move ID; this may hide strength-specific handling.
- Dona Florinda: Pascal special 3 is 38/40, while C lists 38.
- Sr. Barriga: Pascal special 1 is 17/20, C lists 17.
- The remaining rows align at the displayed duration level in this first pass, but frame timing, accepted states, meter, and effects still need detailed comparison.

Do not label a mismatch as an original-game bug. It is a difference between two implementations until the original executable is observed.

### Cross-roster validation backlog

- [x] Inspect all 15 Pascal `[Quadro]` blocks and publish a first-pass frame/timer/effect/sound event index in [frame, AI and hitbox comparison](frame-ai-hitbox-comparison.md).\n- [ ] Expand the index into full ordered per-move event timelines with branch conditions and RNG order.
- [x] Inspect all 15 Pascal `[IA]` blocks and compare character-specific conditions/move IDs against C `05d-cpu-table.txt`; shared CPU prelude and difficulty-scale equivalence remain open.
- [x] Normalize all 379 Pascal `[Ataque]` rows against all 379 C hit rows by action/timer key and rectangle coordinates.\n- [ ] Decode the remaining hit metadata and map action IDs to named moves.
- [ ] Map sound/effect IDs to resource definitions for both projects.
- [x] Resolve Seu Madruga move 22/23 and Sr. Furtado move 22 as character-specific setup paths; identify Jaiminho move 26 as extra2/final-like.\n- [ ] Resolve Jaiminho move 24/25, which appear in the command map but not the cited per-character setup table.
- [ ] Confirm whether C duration and Pascal action timer advance at the same simulation cadence.
- [ ] Validate representative and exceptional moves against the original executable.
- [ ] Add deterministic tests for parser order, sequence timeout, button matching, eligibility, and chained actions.

## Remaining base-roster status

| ID | Fighter | Inventory | Pascal action rules | C setup/AI | Original executable |
|---:|---|---|---|---|---|
| 1 | Chaves | Documented | Frame/event index; AI compared; hitboxes normalized | Setup/AI cross-reference | Pending |
| 2 | Seu Madruga | Documented | Frame/event index; AI compared; hitboxes normalized | Move 22/23 exception mapped | Pending |
| 3 | Chiquinha | Documented | Frame/event index; AI compared; hitboxes normalized | Setup/AI compared | Pending |
| 4 | Quico | Documented | Frame/event index; AI compared; hitboxes normalized | Setup/AI compared | Pending |
| 5 | Dona Florinda | Documented | Frame/event index; AI compared; hitboxes normalized | Setup/AI compared | Pending |
| 6 | Prof. Girafales | Documented | Duration inventory | Setup/AI table available | Pending |
| 7 | Sr. Barriga | Documented | Duration inventory | Setup/AI table available | Pending |
| 8 | Bruxa do 71 | Documented | Air-only special noted | Setup/AI table available | Pending |
| 9 | Paty | Documented | Jump-only special noted | Setup/AI table available | Pending |
| 10 | Nhonho | Documented | Charge/mash behavior noted | Setup/AI table available | Pending |
| 11 | Godines | Documented | Duration inventory | Setup/AI table available | Pending |
| 12 | Sr. Furtado | Documented | Final-like special 4 exception | Setup ID exception noted | Pending |
| 13 | Jaiminho | Documented | Extra 1/2 and grouped final exception | Move ID 26 exception noted | Pending |
| 14 | Dona Neves | Documented | Jump-only special noted | Setup/AI table available | Pending |
| 15 | Glória | Documented | Duration inventory | Setup/AI table available | Pending |


See also: [Frame, AI and hitbox comparison](frame-ai-hitbox-comparison.md) for the complete 15-fighter first-pass index, the 379/379 rectangle match, CPU comparisons, and exceptional-ID findings.
