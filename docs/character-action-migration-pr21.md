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

The migration now routes C01 commands 16–24, C02 commands 16–23, C03 commands 16–24 and C13 commands 14–26 through `execute_character_rot_action`, alongside the existing C04/C09 routes. C06 commands 14–24 now also route through its actual script after adding source-backed effect types 13/15 to the live effect arena. C07 commands 14–24 are now routed through its actual script with focused action-entry tests. CI passed on code commit `324094b475926bbc19085606c7f2c7ecb159c101`. C08 commands 14–24 are now routed through its actual script, with source-backed effect types 18/19 and focused guard/action tests. CI passed on code commit `fbb02c6b9ad7b319b621b5e76ecb2a56f8761089`. C10 commands 14–24 now route through the real C10.rot [Acao] section; focused tests cover every command, weak/strong force, charge writes, special 3 height guard and final-special eligibility. CI passed on code commit `13f13a9af41ab2bce947499d7fbb2e6df535d637` (100 library tests plus integration targets 12, 2, 29, 7, 13, 5 and 5). C11 commands 14–24 now route through the real C11.rot [Acao] section with tests for all entries and source-specific durations, force/charge and final-special writes. C12 commands 14–17 and 20–22 route through the real C12.rot [Acao] section with focused tests; commands 18/19 remain deliberately un-routed because their source guard reads `P.GolpeAtivo`, which is not currently represented in the live `FighterState`. This is an explicit per-command boundary, not a fabricated default. C05 commands 14–24 have now been routed through the real interpreter; focused tests are authored, including victory/pose, and CI passed on f463ef7482cf3b94bd6b8abc88c62903b554f395 (87 library tests plus all integration targets). The C05 Esp1 live-effect rejection guard now has a real effect-type-13 fixture. C06 tests cover victory/pose, weak/strong specials, final-special airborne writes and active-effect rejection for its first two specials. CI passed on code commit `f8bfdb1b03a4250082a1b24a4b2fad0216e0530b`: formatting and all test targets are green (91 library tests plus integration targets 12, 2, 29, 7, 13, 5 and 5). Focused tests cover C01 special 3, C01 final special, C13 extra 1 and C13 extra 2, including comparisons against the retained C01/C13 hand-port oracles. C02 special 1 has weak/strong entry tests and a rejected-action guard test; specials 2–4 and the final special have focused tests for their live writes, air-state changes, power, flash and sound. C03 has weak/strong tests for specials 1–4 and a final-special test covering its source-specific action, sprite, power, flash and sound writes. The Contract tests workflow passed after the C03 migration, including formatting and all test targets (80 library tests plus integration targets of 12, 2, 29, 6, 13 and 5 tests). Broader differential capture against the pinned Pascal runtime remains pending. The old C01 compatibility API remains in the crate as a differential oracle, but `FightSimulation` no longer dispatches C01 specials through it. The former C13 hand-port is retained as a test-only differential oracle and is no longer used by production dispatch.

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
- [x] Formatting and all test targets pass on code commit `13f13a9af41ab2bce947499d7fbb2e6df535d637` (100 library tests plus integration targets 12, 2, 29, 7, 13, 5 and 5).
- [x] C10 commands 14–24 execute the pinned C10.rot [Acao] section, with accepted/rejected branch tests and green CI.


## C11 / C12 source audit (2026-11-23)

- Pinned source revision: OpenStreetChaves commit `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`.
- `research/pascal-rot/C11.rot` and `research/pascal-rot/C12.rot` were compared with their pinned upstream files; both are byte-identical.
- C11 [Acao] commands 14–24 use mapped live fighter fields, mapped constants/globals, and the existing `ExisteEfeito` helper. The effect-type-22 rejection path is preserved by the actual script; no fake effect instance or guessed effect configuration was added.
- C12 [Acao] commands 14–17 and 20–22 use mapped live fields and existing helpers. Commands 18/19 are not dispatched to this script: the source's alternative eligibility branch reads `P.GolpeAtivo` when `P.Acao = acEspecial6`, and that Pascal field has no authoritative live `FighterState` binding yet. We do not default it to false.
- This migration does not add [Quadro], [IA], or [Acerto] execution for either character.


## C13–C15 action-entry slice (2026-11-23)

- Pinned source revision: OpenStreetChaves commit `e1b025c35ecbc2b14d8635564aafd23c2107d3d1`.
- `research/pascal-rot/C13.rot`, `C14.rot` and `C15.rot` were byte-compared with their respective pinned upstream files; all three are byte-identical.
- C13 / Jaiminho now routes commands 14 and 16–26. The source's command 24 maps to `acEspecial4` with `fgEspecial`; Extra1 (25) maps to `acEspecial4` with force 4; Extra2 (26) is the 30-frame final and performs the source's airborne/power/sprite/flash/sound writes.
- C14 / Dona Neves now routes commands 14 and 16–24. Special 3 is accepted only from `acPulo`; the final is 70 frames and uses the source's power, sprite, flash and sound writes.
- C15 / Glória now routes commands 14 and 16–24. Special 3 retains the source's standing/strong-kick-frame cancel guard; the final is 30 frames and uses sprite offset +12.
- Command 15 is not routed because these character scripts have no character-specific pose branch. Focused tests exercise every routed command entry, assert action/frame/duration and the source-specific state writes, and verify that each command is reported as a real ROT execution without runtime errors.
- These tests do not claim execution of [Quadro], [IA] or [Acerto], nor do they claim Pascal differential parity. The active-effect guard paths are executed by the real `ExisteEfeito` host helper, but the source effect types 26/28/29/33 are not yet configured in the effect arena, so no synthetic active-effect fixtures are claimed.
- Structural interpreter limitations remain deferred as agreed.

- CI validation: run [45286642440](https://github.com/danimesq/StreetChavesRust/actions/runs/45286642440) passed on code commit `ffabd727fffb860a1e9534d9d00d87e04872ecc9`: `cargo fmt --all -- --check` and `cargo test --all-targets`; 105 library tests plus integration targets 12, 2, 29, 7, 13, 5 and 5 (178 total, 0 failures).
