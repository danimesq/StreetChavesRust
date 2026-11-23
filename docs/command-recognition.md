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
for both fighter input streams. The current executable character action dispatcher
only implements C01 special commands 16..24. Commands for other characters,
C13's extra commands 25..26, and common basic actions 3..6 are preserved in
`FightTickReport.unhandled_commands`; they are not silently dropped or simulated
with guessed behavior. The Pascal source documents `CargaSoco` and
`CargaChute` but this repository slice does not yet port their producer logic,
so those byte fields remain explicit runtime state inputs rather than guessed
counters.

## Tests

The tests load all 15 embedded tables, validate charge/air/extra-command entries,
exercise C01 sequence-to-button dispatch, verify ordered transitions and timeout
pause/reset, and ensure unimplemented commands remain visible in the tick report.
