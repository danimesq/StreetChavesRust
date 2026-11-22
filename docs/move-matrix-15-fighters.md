# Move matrix: 15-fighter research

## Purpose and evidence rules

This document is the working move-by-move specification for the original 15-fighter roster. It is intentionally incomplete and will grow fighter by fighter. It does **not** claim that either SDL reconstruction is the original game's authoritative behavior.

Evidence labels:
- **[Pascal source]**: directly read from the pinned Pascal INI/ROT source.
- **[C reference]**: directly read from pinned C source/spec/table documentation. No C per-fighter comparison is claimed in this initial entry.
- **[Inference]**: interpretation of source behavior; needs tests.
- **[Original validation pending]**: not confirmed against the original executable.

Pinned Pascal reference: `OpenStreetChaves`, commit `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`.
Pinned C reference from roster research: `open-street-chaves`, commit `e76ad2188af6844f2a5ca5464aafd23c2107d3d1`.

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
