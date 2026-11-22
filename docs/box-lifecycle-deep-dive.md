# 🌈 Box lifecycle deep dive: C selector and Pascal AplicarCaixas

## 🌈 Scope and evidence boundary

This note reconstructs the data lifecycle implemented by the two pinned SDL ports. It does not claim that every detail is confirmed against the original Chaves.exe.

- Pascal reference: OpenStreetChaves at e1b025c35ecbc2b14d8635564aafd23c2107d3d1
- C reference: open-street-chaves at e76ad2188af6842ff6618ecdb8f425bba61c9183
- Pascal implementation files: source/game/ChavesGolpes.pas, source/game/ChavesRoteiro.pas, source/game/Comum_Sprites.inc, source/game/Estado7_Luta.inc
- C implementation files: src/fight/selector.c, src/fight/fighter.c, src/fight/combat.c

The existing table audit found 379/379 hitbox rows and 305/305 hurtbox rows in the Pascal/C data sets after coordinate normalization. That is a row-multiset result. It does not prove that both selectors resolve every row identically at runtime.

## 🌈 C lifecycle: selector writes and stale values

The C path is:

1. fighter_frame runs once per fighter in the current on-top draw order.
2. Unless hit-stop or super-pause is active, fighter_frame calls select_frame.
3. select_frame runs frame selection, field stores, special motion, run-turn logic, effect spawns, effect sounds, and finally sel_boxes.
4. The later melee pass checks the resulting attack fields against the other fighter's hurt fields.
5. After a successful melee hit, melee_step may call select_frame on the victim again after changing the victim's move.

Inside sel_boxes:

- Hurt rows are matched by character ID and current sprite frame.
- A hit row is matched by character ID, state, and animation timer.
- A matching hurt row writes all four hurt coordinates and mirrors the horizontal coordinates when facing left.
- A matching hit row writes all four attack coordinates, the attacking flag, damage, and chip damage. It updates hit level only when the row's level is nonzero.
- Both loops continue after a match. If more than one row matches, later table rows overwrite earlier fields.
- Neither loop clears the existing box fields before searching.
- If no row matches, the previous values remain.

The fighter reset path begins with a zeroed fighter record and then assigns a fallback hurtbox. Thus attack fields begin inactive/zero, but after the first selector writes, a missing match is a retain operation.

## 🌈 Explicit attack deactivation is not a box clear

The two projects have state-machine paths that set the attack-active flag to false independently of the table selector. These paths normally leave the stored attack coordinates intact. Some paths also reset damage or other action fields, but that is a separate state mutation.

| Port | Source location | Deactivation point | Other writes in the same path |
|---|---|---|---|
| Pascal | `Estado7_Luta.inc:966` | A blocked attack sets `P.GolpeAtivo := False` | The guard branch can apply chip damage and hit-stop; the stored attack rectangle is not erased |
| Pascal | `Estado7_Luta.inc:1028-1030` | A successful hit sets both attacker and victim `GolpeAtivo := False` before `ExecutarAcerto` | It then runs the character [Acerto] script and calls `AtualizaQuadro` on the victim, which may immediately select a new box for the victim's new action |
| Pascal | `Estado7_Luta.inc:1065` | Action-duration completion sets `GolpeAtivo := False` | Also sets `DanoGolpe := 0` and resets the action/timer/guard/invulnerability state |
| Pascal | `Estado7_Luta.inc:1097` | A physics/landing transition sets `GolpeAtivo := False` | Also resets selected airborne/action fields; coordinates are not erased here |
| Pascal | `Estado7_Luta.inc:1174` | Landing cleanup sets `GolpeAtivo := False` | Also sets `DanoGolpe := 0` and resets action/airborne fields |
| C | `fighter.c:set_move`, move 10 | Knockdown setup sets `attacking = false` | Replaces fighter state/timer and marks invulnerable; does not erase hit coordinates |
| C | `fighter.c:set_move`, moves 8/9 | Selected reaction moves set `attacking = false` | Does not itself erase hit coordinates or damage |
| C | `fighter.c:move_clock` | Move completion sets `attacking = false` | Also zeros damage and resets action flags/timers |
| C | `fighter.c:physics` | Landing sets `attacking = false` | Also zeros damage and resets airborne/action state |
| C | `combat.c:melee_step` | Blocked attack deactivates the attacker; successful hit deactivates attacker and victim | The hit path may chain the attacker into another move and reselect the victim's frame later in the same call |
| C | `selector.c:sel_throws` and special-motion landing branch | Specific throw/landing transitions set `attacking = false` | Changes action/airborne state; the box fields remain separate |

This gives three different operations which must remain distinct in Rust:

1. **Replace with an inactive row:** a matching table row writes new geometry/damage metadata with the active bit false.
2. **Deactivate the current attack:** an explicit state-machine write changes only the active bit; stored geometry/metadata remain.
3. **Lookup miss:** the selector writes nothing at all, so both the previous active bit and metadata remain unchanged.

A complete `Clear` of the stored box record is a fourth, stronger operation. It should only be emitted when a source actually removes the box record, not as a synonym for `GolpeAtivo := False` or `attacking = false`.

## 🌈 C duplicate-key discrepancy: fighter 11, state 27, timer 1

The C animation table contains two hit rows with the same selector key for character 11:

- first row: geometry corresponding to Pascal 27.1.f1
- second row: geometry corresponding to Pascal 27.1.f2

Both rows carry level 0 and the same active/damage/chip metadata. The C sel_boxes matcher does not test the row's level as a filter. It writes both rows in sequence, so the second geometry wins every time the pair is scanned.

The Pascal C11.ini contains the corresponding 27.1.f1 and 27.1.f2 rows. AplicarCaixas checks each row's strength condition against P.ForcaGolpe before calling DefineCaixaAtaque. Therefore the Pascal path selects the matching strength-specific rectangle instead of unconditionally choosing the last row.

This is a concrete port-level behavioral divergence hidden by the 379/379 row-multiset comparison. Preserve it in the evidence register; do not copy the C last-row result into Rust as if it were proven original behavior.

## 🌈 Pascal lifecycle: AtualizaQuadro then AplicarCaixas

AtualizaQuadro performs these operations in order:

1. Execute the common [Quadro] script.
2. Execute the fighter-specific [Quadro] script.
3. Call AplicarCaixas for the fighter's current state.

AplicarCaixas then performs two independent lookups.

### Hurtbox / body box

- Read the current sprite frame.
- For player 2, subtract the 50-frame strip offset.
- If the resulting frame is in range and its Corpo entry is marked Definida, call DefineCaixaCorpo.
- If the frame has no defined Corpo entry, there is no fallback clear. The prior coordinates remain.

DefineCaixaCorpo writes the coordinates only when the requested frame still equals P.Quadro. It mirrors X by swapping and negating the horizontal endpoints when the fighter faces left.

### Hitbox / attack box

- Iterate the loaded Ataques array from index 0 through its final entry. This preserves the INI section order.
- A row is considered if its strength condition is generic (CondForca = 0) or equals the fighter's current ForcaGolpe.
- DefineCaixaAtaque then checks the current action and action timer.
- Each matching row writes the attack rectangle, active flag, damage, and guard damage.
- A nonzero row Forca updates P.ForcaGolpe; zero leaves the current force unchanged.
- The loop does not stop after the first match. A later row can overwrite an earlier row, and a force update can affect whether a later conditional row matches.
- If no row matches, the old attack geometry, active flag, force, and damage metadata remain.

The strength condition and the row's force assignment are separate concepts. Rust must not collapse them into one field.

## 🌈 active=false is not Clear

A matched inactive attack row still writes geometry and damage metadata, then sets GolpeAtivo to false. It is a disabled stored box, not an instruction to erase the box record.

This distinction also exists in C: a matched row with atk=0 writes the geometry and sets attacking=false. No match is different: it does not write anything and leaves the previous record untouched.

The Rust data model therefore needs at least three distinct cases:

- Replace with an active record.
- Replace with an inactive record.
- Retain the previous record because the lookup found nothing.

Clear should remain a separate explicit operation for state-machine paths that actually clear a box or its active flag.

## 🌈 Facing and update timing

Both ports mirror horizontal coordinates when a matching row is applied. They do not globally recompute the mirror state on every collision check.

Consequently, if facing changes on a tick where no row matches, the previously mirrored coordinates can remain in the old orientation. Rust should preserve the order of facing updates and box selection rather than normalizing boxes every time the collision routine reads them.

Pascal has an additional presentation wrinkle: desenhalutador draws optional debug boxes before calling AtualizaQuadro. The simulation box update then occurs before the later body-hit phase. A debug overlay can therefore show the previously stored box for that draw, while collision later in the same fight frame uses the newly applied box.

## 🌈 Rust contracts now implemented

The Rust library now provides:

- RawBounds::mirrored_horizontally for the source endpoint transform.
- BoxUpdate::Replace, Clear, and Retain as separate operations.
- BoxState::apply_last_match_or_retain to model ordered selectors where the final matching row wins and a miss retains the old value.
- BoxState::deactivate_attack and CombatEvent::AttackDeactivated to model an explicit active-flag clear while preserving stored geometry and damage metadata.
- PascalAttackDefinition, which keeps strength_condition separate from assigned_strength.
- PascalAttackState::apply_ordered_rows, which evaluates matching rows in original order and allows a prior row's force assignment to affect later conditions.

These are adapters/contracts, not a complete fight implementation. The source-specific selector and state-machine clear paths still need to be connected to the future Fighter runtime.

## 🌈 Required next validation

- [x] Confirm C selector key fields and that misses retain old box state.
- [x] Confirm Pascal AplicarCaixas misses retain old box state.
- [x] Confirm the Pascal attack loop is ordered and has separate condition/assignment strength fields.
- [x] Identify the character 11 state 27 timer 1 duplicate-row divergence.
- [x] Trace the principal explicit attack-flag deactivation sites in both fight paths and distinguish them from selector misses.
- [ ] Exhaustively inventory any remaining attack-flag writes outside the inspected fight/selector/physics files.
- [ ] Validate whether the original executable matches Pascal's strength-specific row selection or the C port's last-row behavior.
- [ ] Capture facing changes on a frame with no box-table row.
- [ ] Add source-backed fixtures for state transitions that clear attacking while retaining geometry.
- [ ] Confirm hit-stop timing around selector calls and the same-tick victim re-selection path.

## 🌈 Source links

- [Pascal ChavesGolpes.pas](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/game/ChavesGolpes.pas)
- [Pascal Comum_Sprites.inc](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/game/Comum_Sprites.inc)
- [Pascal Estado7_Luta.inc](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/game/Estado7_Luta.inc)
- [Pascal C11.ini](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.ini)
- [C selector.c](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/fight/selector.c)
- [C fighter.c](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/fight/fighter.c)
- [C combat.c](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/fight/combat.c)
- [C 05a animation tables](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/docs/spec/05a-anim-tables.txt)
