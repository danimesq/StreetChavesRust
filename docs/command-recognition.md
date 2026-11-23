# Pascal command recognition primitives

## Scope

This slice implements the mechanics needed by the Pascal `RegraVale` and `sequenciagolpe` routines: facing-relative directions, AND-condition evaluation, attack-button edge triggers, ordered multi-rule evaluation, `TempoBaixo`/`TempoTras` counters capped at 18 with one-tick decay, and sequence-state transitions.

## API

- `parse_condition`: parses supported condition atoms (`B/F/T/C/H/A`, numeric `b/t/P/s/k`, and negation `!`).
- `rule_matches`: AND-evaluates terms in the supplied order.
- `CommandRecognizer::update_counters`: updates direction-hold counters once per simulation tick.
- `commands_for_button`: returns all matching command codes in source order, only on a press edge.
- `apply_sequences`: applies configured sequence transitions in order when the current state matches the rule's source state.

## Explicit integration boundary

Character-specific rules still must be loaded from each fighter's `[Botoes]` and `[Sequencias]` INI sections. This PR does not invent button mappings, populate a character table from guesses, or connect command results to `FightSimulation`'s action dispatcher. That integration follows after the rule loader and shared action pipeline are source-audited.

Ground-Y is passed explicitly to condition evaluation, avoiding a hard-coded stage coordinate.

## Validation

Five tests cover condition parsing, facing-relative direction reversal, counter cap/decay, multiple matching commands preserving rule order, and sequence transitions requiring the matching current state.
