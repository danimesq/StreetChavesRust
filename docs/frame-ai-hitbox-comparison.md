# 🌈 Frame, AI and hitbox comparison — 15 fighters

## 🌈 Scope and evidence boundary

This document records a first complete cross-roster source pass over the 15 base fighters. It expands the earlier move-name/duration inventory with Pascal `[Quadro]` event indexes, Pascal `[IA]` versus C CPU comparisons, and a normalized comparison of every Pascal `[Ataque]` row against the C hit table.

This is **not** original-game validation. A difference between Pascal and C is a difference between two reconstructions until the original executable is observed. The Rust first-playable milestone must not wait for the final executable-parity stage.

Pinned references:

- Pascal: [OpenStreetChaves, `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`](https://github.com/danimesq/OpenStreetChaves/tree/e1b025c35ecbc2b14d8635564aafd23c2107d3d1)
- C: [open-street-chaves, `e76ad2188af6842ff6618ecdb8f425bba61c9183`](https://github.com/danimesq/open-street-chaves/tree/e76ad2188af6842ff6618ecdb8f425bba61c9183)

Primary files:
- Pascal: `Dados/Lutadores/C01.rot` through `C15.rot`; `Dados/Lutadores/C01.ini` through `C15.ini`.
- C: `docs/spec/05a-anim-tables.txt`, `docs/spec/05b-move-setup.txt`, `docs/spec/05d-cpu-table.txt`, `docs/spec/04a-move-tables.txt`.

## 🌈 Pascal [Quadro] sweep

The table inventories the character-specific `[Quadro]` block. “Frame writes” counts syntactic `P.Quadro :=` assignments, not unique sprite frames. Timer values are the literal action-timer predicates found in the block, including modulo predicates. Effect and sound IDs are Pascal-local identifiers, not yet mapped to resource filenames or C IDs.

| ID | Fighter | Lines | Frame writes | Timer predicates | Effect IDs | Numeric sound IDs |
|---:|---|---:|---:|---|---|---|
| 1 | Chaves | 100 | 28 | 0,2,3,4,6,7,8,9,10,12,13,14,15,18,19,20,21,28,mod 3 | 2,7,8,16 | 14,15,16,17,18 |
| 2 | Seu Madruga | 165 | 17 | 0,2,3,5,8,10,11,15 | 1,2,9 | 20,22 |
| 3 | Chiquinha | 121 | 28 | 0,1,2,3,4,5,6,8,10,15,20,25,30,35 | 1,2,9 | symbolic hit sounds |
| 4 | Quico | 81 | 22 | 0,1,2,4,5,6,7,9,10,13 | 2,11,12 | 28 |
| 5 | Dona Florinda | 89 | 36 | 0,1,2,3,4,5,6,8,9,10,13,15,17,20,21,25,26,28,29,33,35,37,38,41,mod 2 | 13 | 32 |
| 6 | Prof. Girafales | 117 | 22 | 0,2,3,5,6,7,8,9,10,12,13,25,26,mod 4 | 15 | 34 |
| 7 | Sr. Barriga | 107 | 28 | 0,2,3,4,5,6,8,10,12,14,16,mod 2 | 1,2 | symbolic hit sounds |
| 8 | Bruxa do 71 | 119 | 21 | 0,1,2,4,5,6,8,9,10,11,13,15,17,mod 2 | 17,18,19,20,24 | 42 |
| 9 | Paty | 123 | 31 | 0,1,2,3,4,5,6,7,8,9,10,11,12,18,22,mod 2,mod 3 | none in this block | 46 |
| 10 | Nhonho | 100 | 19 | 0,2,3,4,5,6,10,12,14,15,16,20 | 2 | 47 |
| 11 | Godines | 106 | 34 | 0,1,2,4,5,6,8,10,11,12,14,16,18,20,22,24,26,27,28 | 2,22,23 | 50,51,52 |
| 12 | Sr. Furtado | 123 | 25 | 0,2,3,4,5,6,8,10,12,13,14,16,25,mod 2,mod 8 | 2,9 | 53,54,55,56 |
| 13 | Jaiminho | 78 | 14 | 0,1,2,3,5,8,10,11,15,mod 2 | 17,26,27 | 41,57,58,59 |
| 14 | Dona Neves | 86 | 19 | 0,1,2,3,4,5,7,9,10,15,17,32,mod 2,mod 3 | 29,30,31,32 | 60,61,62 |
| 15 | Glória | 116 | 19 | 0,1,2,3,4,5,7,8,9,10,12,13,14,15,16,20 | 33,34,35,36 | 63,64,65,66 |

### 🌈 Event observations

- **Frame selection is not a pure lookup.** The scripts also mutate Y/velocity, facing-relative push, airborne flags, invulnerability, hit counters, opponent state, and sometimes life/damage directly. These operations must become ordered simulation events/state transitions, not renderer side effects.
- **Effect IDs are local to each reconstruction's resource namespace.** For example, Pascal Chaves uses 7/2/16/8, while Seu Madruga uses 9/1/2. Numeric coincidence across characters or projects does not establish shared art or behavior.
- **Sound assignments include both numeric IDs and symbolic constants** such as `sndGolpeFraco`, `sndAcertoForte`, and `sndEspFinal`. A resource catalog is still needed before naming or cross-mapping them.
- **Paty has no `CriaEfeito` call in her character `[Quadro]` block** in this source snapshot. This does not prove she has no effects: common scripts, other actions, or the effect system may still create them.
- **Seu Madruga and Godines contain substantial chained-hit logic** in their frame scripts. Seu Madruga can advance through additional action states and directly stage opponent reactions; those chains need dedicated Rust tests.
- **Jaiminho's frame script uses screen/camera-relative effect spawn coordinates** in addition to fighter-relative coordinates. Preserve the coordinate expressions as evidence; do not normalize them prematurely.

The timer sets above are an index, not a full event trace. Exact per-branch ordering, repeat behavior, RNG calls, and same-tick interactions remain to be translated into explicit event tables and tests.

## 🌈 Pascal [IA] versus C CPU table

The character-specific Pascal `[IA]` blocks were compared with C `05d-cpu-table.txt`. The broad condition/move structure aligns across the roster, including projectile checks, distance gates, self-air/anti-air checks, and exceptional move IDs. The comparison below records the notable per-character interpretation points.

| ID | Fighter | Pascal-specific decision shape | C table correspondence / open point |
|---:|---|---|---|
| 1 | Chaves | Projectile S1; distance S1/S2/S3/S4; airborne-opponent final | Same six decision rows and thresholds. Probability scale still differs in representation: Pascal `Fator` versus C `k`. |
| 2 | Seu Madruga | Projectile S1; close S2/S4; self-air S3; special4-strong final-like branch | C uses move 22 for the 45-tick S4 path and move 23 for the 13-tick power-gated branch. Do not call move 23 generic super 24. |
| 3 | Chiquinha | Projectile S1; distance S1–S4; final under 250 | C rows align with the same distance/projectile pattern; probability operands need common difficulty semantics before numeric equivalence is claimed. |
| 4 | Quico | Projectile S1; S2/S3 under 200; S4 under 100; final under 250 | C table has the same broad gates. Pascal S4 has no explicit `Ameaca=0` guard, matching the C ANY row rather than NOPROJ. |
| 5 | Dona Florinda | Projectile S1; S2 under 150; S3 under 300; S4 under 100; final under 200 | C rows align structurally, including charge/mash move recognition elsewhere in the command table. |
| 6 | Prof. Girafales | Projectile branch selects S2; distance branch uses `Random(4)` from S1 base; S3/S4/final at close/mid range | C uses projectile move 18 and a four-ID range beginning at move 16. This is a deliberate multi-slot range, not a simple S1-only choice. |
| 7 | Sr. Barriga | All character-specific tests require no threat; five distance gates | C table contains the corresponding NOPROJ rows. Shared AI behavior still needs to be compared separately. |
| 8 | Bruxa do 71 | Projectile S1; self-air S2; distance S3/S4/final | C marks S2 as `ANY+SELFAIR100`; preserve the self-air condition rather than interpreting it as opponent anti-air. |
| 9 | Paty | S1 under 200; S2 under 100; S3 under 150; S4 under 100; final under 200 | C has the same shape, with ANY rows for S2/S3 and NOPROJ rows where Pascal checks `Ameaca=0`. |
| 10 | Nhonho | S1 under 200; S2 under 300; S3 under 100; S4 under 200; final under 200 | C rows align with the ANY/NOPROJ distinctions. Charge/mash input counters remain a separate command-recognition concern. |
| 11 | Godines | S1 under 200; S2 under 300; S3 under 300; S4/final under 200 | C rows align with those ranges; final retains the no-threat condition. |
| 12 | Sr. Furtado | Projectile S1; S2 under 80; S3 under 150 plus a second self-air S3 branch; S4 is power-gated final-like | C has two move-20 decision rows for the S3 cases and move 22 as the power-gated final-like action. There is no ordinary move 24 row in this character's setup table. |
| 13 | Jaiminho | Projectile S1; S2 under 90; S3 over 130; S4 over 120 with a four-ID random range; extra2 under 270 | C maps the extra action to move 26. The move-24/25 command-table entries do not have corresponding rows in the cited C move-setup table; their exact runtime meaning remains unresolved. |
| 14 | Dona Neves | Projectile S1; projectile-near S2; S3 under 120; S4 under 80; final under 120 | C has matching PROJ/NOPROJ/ANY row types and thresholds. |
| 15 | Glória | Projectile S1; S2/S3/S4/final distance gates; extra S3 anti-air check | C expresses the extra S3 gate as `NOPROJ+OPPAIR50`; preserve it as a separate branch. |

### 🌈 AI comparison limits

- C's `cpu()` has a shared prelude: preferred-distance selection, walking, and close-range basic actions. This document compares the character-specific decision blocks, not the complete CPU behavior against Pascal `Comum.rot`.
- C's difficulty coefficient is documented as `k = (8 - level) * 0.6 + 1.3 - 0.1 * fightNo`. Pascal scripts use `Round(n * Fator)`; do not assume `Fator == k` without tracing Pascal's shared difficulty calculation and RNG semantics.
- The order of sequential random checks matters. A table of probabilities does not capture RNG consumption order or interactions when more than one branch can fire in the same tick.
- C move IDs are mapped through per-character button/sequence tables. The move ID alone does not prove a named special slot for exceptions.

## 🌈 Hitbox and hurtbox normalization: 379 + 305 exact matches

**Result: all 379 Pascal `[Ataque]` records match all 379 C `hit` rows, including every non-coordinate metadata field. All 305 Pascal `[Corpo]` hurtbox records also match all 305 C `hurt frame` rows after coordinate normalization.**

### 🌈 Hitbox comparison

Normalization used:

1. Pascal keys are `action.timer`, optionally suffixed with `.f1` or `.f2`. The suffix is retained as strength-variant metadata; the shared join key is `action.timer`.
2. Pascal `[Ataque]` values are `x1,y1,x2,y2,active,strength,damage,guardDamage`.
3. C `hit` rows encode `state,timer,guardDamage,damage,strength,active,bottom,right,top,left`. The last four coordinates are reordered to compare with Pascal's `x1,y1,x2,y2`.
4. The metadata mapping is exact: Pascal `active,strength,damage,guardDamage` equals C `active,strength,damage,guardDamage` after selecting C columns 6, 5, 4, 3 respectively (1-based columns after the `hit` label).
5. Compare as a multiset, not a map: repeated action/timer rows are valid and must not overwrite one another. Strength suffixes must not be discarded before validating variant metadata.

| ID | Fighter | Pascal rows | C rows | Exact geometry + metadata |
|---:|---|---:|---:|---:|
| 1 | Chaves | 21 | 21 | 21 |
| 2 | Seu Madruga | 21 | 21 | 21 |
| 3 | Chiquinha | 21 | 21 | 21 |
| 4 | Quico | 19 | 19 | 19 |
| 5 | Dona Florinda | 36 | 36 | 36 |
| 6 | Prof. Girafales | 30 | 30 | 30 |
| 7 | Sr. Barriga | 25 | 25 | 25 |
| 8 | Bruxa do 71 | 15 | 15 | 15 |
| 9 | Paty | 33 | 33 | 33 |
| 10 | Nhonho | 33 | 33 | 33 |
| 11 | Godines | 34 | 34 | 34 |
| 12 | Sr. Furtado | 29 | 29 | 29 |
| 13 | Jaiminho | 23 | 23 | 23 |
| 14 | Dona Neves | 18 | 18 | 18 |
| 15 | Glória | 21 | 21 | 21 |
| | **Total** | **379** | **379** | **379** |

### 🌈 Hurtbox comparison

Pascal `[Corpo]` records use `frame=x1,y1,x2,y2`. C `hurt frame` rows use `left,bottom,right,top`. Reordering C to `left,top,right,bottom` yields an exact match with Pascal's `x1,y1,x2,y2`.

| ID | Fighter | Pascal [Corpo] | C hurt rows | Exact matches |
|---:|---|---:|---:|---:|
| 1 | Chaves | 23 | 23 | 23 |
| 2 | Seu Madruga | 21 | 21 | 21 |
| 3 | Chiquinha | 23 | 23 | 23 |
| 4 | Quico | 21 | 21 | 21 |
| 5 | Dona Florinda | 21 | 21 | 21 |
| 6 | Prof. Girafales | 22 | 22 | 22 |
| 7 | Sr. Barriga | 18 | 18 | 18 |
| 8 | Bruxa do 71 | 18 | 18 | 18 |
| 9 | Paty | 22 | 22 | 22 |
| 10 | Nhonho | 18 | 18 | 18 |
| 11 | Godines | 16 | 16 | 16 |
| 12 | Sr. Furtado | 20 | 20 | 20 |
| 13 | Jaiminho | 22 | 22 | 22 |
| 14 | Dona Neves | 18 | 18 | 18 |
| 15 | Glória | 22 | 22 | 22 |
| | **Total** | **305** | **305** | **305** |

### 🌈 Interpretation and limits

### 🌈 Important selector discrepancy: character 11, action/state 27, timer 1

The 379/379 result is a multiset comparison: both sources contain the same two strength-specific rectangles and metadata. It is not proof that the runtime selectors choose the same rectangle.

Pascal C11.ini has two rows for 27.1, one conditioned on f1 and one on f2. AplicarCaixas checks the current force condition before applying a row.

The C table contains two rows for character 11, state 27, timer 1. The C sel_boxes loop matches both rows using only character/state/timer, does not filter by hit level, and keeps scanning. Both rows have level=0, so the second row overwrites the first geometry regardless of the current force.

This is a concrete behavior difference in the pinned ports. Preserve the row-multiset match as a data finding, but track the selector discrepancy separately. The original executable must decide which behavior Rust ultimately adopts.

This is unusually strong cross-port evidence: in the pinned snapshots, both projects agree on every listed hitbox coordinate and metadata field, and on every listed hurtbox coordinate. The comparison includes active state, strength/level, damage, and guard/chip damage, not just rectangles.

Code-path tracing now establishes that both pinned ports retain old box fields on a lookup miss, and both mirror coordinates only when a matching row is applied. The C selector's no-strength-filter behavior for character 11/action 27/timer 1 is a separate port divergence. These are [port-specific] execution findings, not original-executable proof. Original activation timing, facing/mirroring edge cases, cross-system call order, and the disputed character-11 variant still require controlled executable observations.

Remaining:
- Correlate action IDs with named normal/special moves and frame selection.
- Verify coordinate mirroring and world-space conversion in the Rust core.
- [x] Trace hit activation/deactivation and lookup misses in both selectors.
- [x] Add deterministic Rust contracts for explicit retain, inactive records, and Pascal strength-specific row filtering.
- [ ] Verify the character-11 state-27/timer-1 variant against the original executable.
- [ ] Add original-backed fixtures for representative rows and absent-frame hurtbox retention.
- Reserve original-executable parity checks for the final compatibility stage after the first playable Rust baseline.

## 🌈 Exceptional move-ID resolution

### 🌈 Seu Madruga

- Pascal `cmEsp4Fraco` starts `acEspecial4`, duration 45, with charge gain; on hit, the character script chains to `acEspecial5`, duration 30.
- Pascal `cmEsp4Forte` starts `acEspecial6`, duration 13, consumes power, and is health/power gated. Its hit reaction chains to `acEspecial7`, duration 40.
- C `04a-move-tables.txt` maps the relevant command/button states to move 22 and move 23. C `05b-move-setup.txt` gives move 22 the 45-tick path and move 23 the 13-tick health/meter-gated path.
- **Resolution:** C move 22 corresponds to the Pascal weak special-4 path; C move 23 corresponds to the Pascal strong/final-like path. Neither should be renamed to generic move 24.

### 🌈 Sr. Furtado

- Pascal `cmEsp4Fraco` starts `acEspecial4`, duration 32, consumes power, sets the final-special sound/flash, and has no generic `cmEspFinal` branch in this character's action script.
- C `05b-move-setup.txt` has character 12 move 22 as the 32-tick health/meter-gated, sound-0x13 action; it has no ordinary move 24 entry for this fighter.
- **Resolution:** C move 22 is Sr. Furtado's final-like fourth-slot action. Keep it character-specific in Rust data; do not manufacture a move 24 alias.

### 🌈 Jaiminho

- Pascal groups `cmEsp4Fraco`, `cmEsp4Forte`, `cmEspFinal`, and `cmEspExtra1` into `acEspecial4` (duration 30), with strength/extra flags distinguishing the script path.
- Pascal `cmEspExtra2` starts `acEspecial5` (duration 30), consumes power, and is final-like.
- C `04a-move-tables.txt` exposes command mappings for move IDs 22–25 and maps the extra back-charge command to move 26. C `05b-move-setup.txt` documents move 22/23 and move 26, but not 24/25 in Jaiminho's per-character setup block.
- **Partial resolution:** move 26 is the extra2/final-like path. Move 22/23 occupy the documented fourth-slot setup path. **Move 24/25 remain unresolved** between the C command map and move-setup table; do not invent their state/duration or silently collapse them into 22/23.

## 🌈 Rust implementation implications

- Store hitboxes as validated data keyed by fighter, action, timer, and optional strength variant. Preserve duplicate rows and source provenance.
- Treat frame scripts as ordered simulation logic or explicit event timelines. Do not move opponent mutation, damage, RNG, or collision effects into rendering.
- Keep Pascal and C event IDs namespaced until resource tables prove a mapping.
- Model character-specific move IDs as explicit data, not a universal `special_slot * 2` assumption.
- Preserve the original 15-fighter roster. Chapolin remains the separately documented optional bonus.
- The first playable Rust baseline is its own milestone. Original-executable differential testing is the final compatibility stage after that baseline exists.

## 🌈 Remaining work

- [x] Expand all 15 character `[Quadro]` blocks into a source-ordered event ledger; full guard-path and RNG expansion remains pending.
- [ ] Map every Pascal sound/effect ID to resource definitions and compare with C resource tables.
- [x] Compare all 379 C hit metadata tuples (active/strength/damage/guard) with Pascal: exact multiset match.
- [x] Normalize all 305 Pascal `[Corpo]` hurtboxes against C hurt rows: exact match.
- [ ] Map all action IDs to normal attacks and named moves without numeric-coincidence assumptions.
- [ ] Resolve Jaiminho C move IDs 24/25 from the hand-coded C path or mark them unknown with a source-backed explanation.
- [ ] Compare Pascal `Comum.rot` CPU behavior with C's shared CPU prelude.
- [ ] Build deterministic Rust fixtures for representative and exceptional move/event chains.
- [ ] After the first playable Rust baseline, run differential captures against the original executable.
