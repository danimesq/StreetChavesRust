# Mercedes action-dispatch slice

## Implemented in this branch

- Common [Acao] command IDs 3..6:
  - weak/strong punch and kick;
  - standing, crouching and airborne variants;
  - Pascal action IDs and durations;
  - charge-power increments guarded by the pre-increment value;
  - outgoing weak/strong attack sounds.
- Directional human input in `FightSimulation`: horizontal movement, crouch,
  crouch release, jump start, jump direction and source-ordered common action
  finalization.
- C13 `cmEspExtra1` (25): enters `acEspecial4`, duration 30, charge gain 5,
  and literal `ForcaGolpe = 4`.
- C13 `cmEspExtra2` (26): enters `acEspecial5`, duration 30, clears air/jump
  state, consumes `Poder`, applies the low-health/power guard, sets the sprite
  offset, screen flash and final-special sound.

## Still pending

- C02..C12 and C14..C15 [Acao] sections. These scripts depend on the Pascal
  ROT interpreter's expression and statement semantics. Treating their command
  IDs as a generic special would erase character-specific guards, side effects,
  and source order.
- `CargaSoco` and `CargaChute` production/decrement. In the pinned
  `OpenStreetChaves` source revision, `ChavesTipos.pas` documents them as
  decreasing button counters and `Estado7_Golpes.inc` reads them in
  `RegraVale`. C13/C07/C10 ROT scripts reset them. No assignment/decrement
  producer was found in the audited fight/input includes. This branch therefore
  does not invent a countdown seed, decay rate, or repeat-press policy.

## Source provenance

- Repository: `danimesq/OpenStreetChaves`
- Revision: `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`
- Relevant sources: `Dados/Lutadores/Comum.rot`,
  `Dados/Lutadores/C13.rot`, `source/game/Estado7_Luta.inc`,
  `source/game/Estado7_Golpes.inc`, `source/game/ChavesTipos.pas`.

## Tests

Integration tests cover a basic weak punch, C13 extra 1 and C13 extra 2.
The full suite remains the CI gate for formatting, compilation and regression.


## Source-order corrections

The shared `Comum.rot [Acao]` prelude clears `P.Defendendo` before testing a command. The Rust dispatcher now preserves that write for C13 extra commands and for character commands that remain explicitly unimplemented; those paths also run the shared `[AcaoFim]` epilogue. Directional left/right/down calls clear the flag before evaluating their common movement guard.

The common crouched strong-kick branch has one source-specific exception: `P.Personagem = pnJaiminho` sets `P.DuracaoAcao := 18`, while the normal duration is 11. The Rust common-action slice now preserves that C13/Jaiminho exception and has a dedicated regression test.
