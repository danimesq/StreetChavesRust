# PR #21 — Progressive character migration to the ROT runtime

## Responsibility

PR #20 established and tested the first live ROT execution path. This PR owns the progressive migration of character action behavior from compatibility hand-ports into that runtime.

This is not a claim that the interpreter is universal. Every migrated command must remain bounded by the fields, globals, helpers, numeric behavior and call semantics that have actually been implemented and validated.

## Starting point

At the start of this PR:

- C04 / Quico final-special command (24) executes through the ROT runtime.
- C09 / Paty special commands (14, 15, 20, 21) execute through the ROT runtime.
- C01 special commands 16–24 still have a compatibility dispatcher in `chaves_specials.rs`.
- C13 extra commands 25/26 still have a compatibility implementation in the common-action path.
- Common basic actions 3–6 remain a bounded common compatibility slice.
- The ROT runtime does not yet expose every field/helper or every Pascal call behavior.

## Goals

1. Expand real character [Acao] execution through the ROT interpreter, preserving the authoritative `CombatState`, the shared fight RNG, source order and existing engine helpers.
2. Progressively replace the C01 and C13 hand-ports only after their actual ROT branches execute and pass focused regression tests.
3. Migrate further C02–C15 commands by source-audited character/command slices. Do not infer one character's behavior from another.
4. Preserve explicit unsupported-command reporting and source-located runtime errors.
5. Keep the permanent runtime contract and the temporary delivery ledger synchronized with what is actually implemented.

## Suggested order

### A. C01 hand-port migration

- Route C01 command codes 16–24 through the real `C01.rot [Acao]` section.
- Preserve `Comum.rot [Acao]` → character `[Acao]` → `Comum.rot [AcaoFim]` ordering.
- Compare the ROT execution against the existing C01 hand-port across accepted and rejected commands, all guards, ordered state writes, effects, sound/flash and RNG.
- Remove or reduce the hand-port only after parity is demonstrated. Keep any still-needed compatibility behavior explicit.

### B. C13 extra-command migration

- Route C13 commands 25/26 through the actual `C13.rot [Acao]` section.
- Audit and implement any missing host bindings/helpers required by those exact branches.
- Preserve C13's source-specific state writes and the common prelude/epilogue.
- Retain the current implementation as a regression oracle until the ROT path is proven equivalent; then remove duplicated production logic.

### C. Roster expansion

- Audit each character's own `[Acao]` source and dependencies before enabling it.
- Migrate bounded command groups, not entire characters by assumption.
- Record commands that remain unsupported and why.
- Do not synthesize missing optional sections or generic special behavior.

## Explicit runtime boundaries

The following are not solved merely by opening this PR:

- `AplicarCaixas` host integration.
- Recursive/nested `AcaoJogador` lifecycle calls.
- Int64-width Pascal RNG host calls.
- Pascal `Extended` numeric equivalence (Rust currently uses `f64` for real values).

Other unmapped fields, globals and helpers remain unsupported until audited and implemented. An error is preferable to a guessed write or silently skipped call.

## Validation requirements

For each migrated command, tests should cover as applicable:

- accepted and rejected eligibility guards;
- actual player/opponent state writes;
- exact source order, including independent sequential `if` branches;
- effects, ownership, sounds, flash/shake and movement;
- shared RNG draw width, range, order and cursor continuity;
- common prelude and epilogue even on rejected character branches;
- parity with the previous hand-port while it remains available as an oracle.

Run:

```sh
cargo fmt --all -- --check
cargo test --all-targets
```

Do not mark a command migrated based only on its final action ID. Do not claim roster-wide or universal runtime compatibility until the relevant source branches and evidence exist.

## Current implementation snapshot (PR #21 branch)\n\nThe first migration slice now routes C01 commands 16–24 and C13 commands 25/26 through `execute_character_rot_action`, alongside the existing C04/C09 routes. Focused tests were added for C01 special 3, C01 final special, C13 extra 1 and C13 extra 2, including comparisons against the retained C01/C13 hand-port oracles. The Contract tests workflow passed on the current branch head, including formatting and all test targets. Broader differential capture against the pinned Pascal runtime remains pending. The old C01 compatibility API remains in the crate as a differential oracle, but `FightSimulation` no longer dispatches C01 specials through it. The former C13 hand-port is retained as a test-only differential oracle and is no longer used by production dispatch.\n\n## Delivery ledger

- [x] C01 commands 16–24 are routed through ROT with focused entry-state regression tests; differential parity validation remains pending.
- [x] C01 special-command hand-port is no longer used by FightSimulation; the compatibility API remains available until differential parity is proven.
- [x] C13 commands 25/26 are routed through ROT with focused entry-state regression tests; differential parity validation remains pending.
- [x] C13 extra-command hand-port is no longer used by production dispatch; it remains available as a differential test oracle until parity evidence is sufficient for deletion.
- [ ] At least one additional source-audited character command is migrated.\n- [x] Run CI and confirm the new C01/C13 ROT-vs-hand-port differential tests pass. (The selected C01/C13 cases pass; broader Pascal capture remains pending.)
- [ ] Runtime contract and documentation reflect the exact implemented boundary.
- [ ] Formatting and all test targets pass.
