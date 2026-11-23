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

## Current implementation snapshot (PR #21 branch)

The migration now routes C01 commands 16–24, C02 commands 16–23, C03 commands 16–24 and C13 commands 25/26 through `execute_character_rot_action`, alongside the existing C04/C09 routes. C06 commands 14–24 now also route through its actual script after adding source-backed effect types 13/15 to the live effect arena. C05 commands 14–24 have now been routed through the real interpreter; focused tests are authored, including victory/pose, and CI passed on f463ef7482cf3b94bd6b8abc88c62903b554f395 (87 library tests plus all integration targets). The C05 Esp1 live-effect rejection guard now has a real effect-type-13 fixture. C06 tests cover victory/pose, weak/strong specials, final-special airborne writes and active-effect rejection for its first two specials. CI passed on code commit `f8bfdb1b03a4250082a1b24a4b2fad0216e0530b`: formatting and all test targets are green (91 library tests plus integration targets 12, 2, 29, 7, 13, 5 and 5). Focused tests cover C01 special 3, C01 final special, C13 extra 1 and C13 extra 2, including comparisons against the retained C01/C13 hand-port oracles. C02 special 1 has weak/strong entry tests and a rejected-action guard test; specials 2–4 and the final special have focused tests for their live writes, air-state changes, power, flash and sound. C03 has weak/strong tests for specials 1–4 and a final-special test covering its source-specific action, sprite, power, flash and sound writes. The Contract tests workflow passed after the C03 migration, including formatting and all test targets (80 library tests plus integration targets of 12, 2, 29, 6, 13 and 5 tests). Broader differential capture against the pinned Pascal runtime remains pending. The old C01 compatibility API remains in the crate as a differential oracle, but `FightSimulation` no longer dispatches C01 specials through it. The former C13 hand-port is retained as a test-only differential oracle and is no longer used by production dispatch.

## Delivery ledger

- [x] C01 commands 16–24 are routed through ROT with focused entry-state regression tests; differential parity validation remains pending.
- [x] C01 special-command hand-port is no longer used by FightSimulation; the compatibility API remains available until differential parity is proven.
- [x] C13 commands 25/26 are routed through ROT with focused entry-state regression tests; differential parity validation remains pending.
- [x] C13 extra-command hand-port is no longer used by production dispatch; it remains available as a differential test oracle until parity evidence is sufficient for deletion.
- [x] C02 commands 16–23 execute through the real ROT [Acao] section, with weak/strong coverage for specials 1–3, special 4, final special and rejected-guard coverage for special 1.
- [x] C03 commands 16–24 execute through the real ROT [Acao] section, with weak/strong coverage for specials 1–4 and final-special state/effect assertions.
- [x] C05 commands 14–24 are routed through the real C05.rot [Acao] section; victory/pose, weak/strong, landing-height, charge-reset and final-special tests pass CI.
- [x] C05 Esp1's `ExisteEfeito(13, Jog)` rejection guard is covered with a real effect-type-13 instance created by the source-backed effect arena.
- [x] Run CI and confirm the new C01/C13 ROT-vs-hand-port differential tests pass. (The selected C01/C13 cases pass; broader Pascal capture remains pending.)
- [x] C02 special-one weak/strong entry and rejected-guard tests pass in CI.
- [x] Runtime contract and documentation reflect the exact implemented boundary.
- [x] Formatting and all test targets pass on code commit `f8bfdb1b03a4250082a1b24a4b2fad0216e0530b`.
