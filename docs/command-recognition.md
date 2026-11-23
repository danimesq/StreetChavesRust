# Pascal character command recognition

## Source data

The loader consumes `data/fighter-command-rules.ini`, a compile-time extraction of
`[Sequencias]` and `[Botoes]` from the 15 original files
`Dados/Lutadores/C01.ini` through `C15.ini` in
`danimesq/OpenStreetChaves`, pinned at
`e1b025c35ecbc2b14d8635564aafd23c2107d3d1` (GPL-3.0). Section names are prefixed
with the fighter ID solely to combine the original two sections into one resource.
Values and per-section numeric row order are retained.

## Runtime behavior

- Portuguese INI terms are parsed: `Baixo`, `Frente`, `Tras`, `Cima`,
  `TempoBaixo>=N`, `TempoTras>=N`, `NoChao`, `NoAr`, `Passo=N`,
  `CargaSoco>=N`, and `CargaChute>=N`, including `!` negation.
- `[Sequencias]` rows are evaluated every simulation tick in source order.
  A successful transition writes `SeqTempo=9`; the timer is decremented after
  the rule loop and resets the sequence state at zero. Hit-stop and screen flash
  pause this timeout, matching Pascal.
- `[Botoes]` rows are checked on the four attack-button press edges, in button
  order and then INI row order. All matching rows are preserved and dispatched.
- The original Pascal rule loader ignores a complete row with an unknown term;
  the Rust loader retains such rows in `ignored_rules` instead of reordering them.
- Grounded is exactly `P.Y = ChaoY`; airborne is exactly `P.VelY <> 0`.
  Punch and kick charge predicates use separate `CargaSoco`/`CargaChute` state.

## Dispatcher boundary

`FightSimulation` loads command rules for C01..C15 and now uses the real tables
for both fighter input streams. The common input path now executes the Pascal basic attack commands 3..6,
including ground, crouch and jump variants, action durations, charge-power gain
and outgoing hit sounds. C01 special commands 16..24 retain their audited
dispatcher. C13's character-specific extra commands 25/26 are also implemented
from C13.rot, including the literal force value 4 and the final-special power,
flash and sound writes.

C02..C12 and C14..C15 character-specific [Acao] scripts are still not executed
by Rust. Their command IDs remain visible in `FightTickReport.unhandled_commands`.
The Pascal source's generic ROT interpreter is a separate subsystem and must be
ported before those scripts can be run faithfully.

The Pascal record describes `CargaSoco` and `CargaChute` as decreasing counters
for punch and kick buttons. In the pinned source revision, their only runtime
reads are in `RegraVale`; C13/C07/C10 scripts reset them, but no producer or
decrement routine appears in the audited fight/input source. The new runtime
does not invent a duration or repeat-press rule. The byte fields remain
explicit state inputs until that missing producer is located or independently
validated.

## Tests

The tests load all 15 embedded tables, validate charge/air/extra-command entries,
exercise C01 sequence-to-button dispatch, verify ordered transitions and timeout
pause/reset, and ensure unimplemented commands remain visible in the tick report.
