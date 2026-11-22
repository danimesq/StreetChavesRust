# 🌈 Combat tick and collision contracts

## 🌈 Purpose

This document defines the first Rust-facing contracts derived from the research ledger. It makes ordering, random draws, and box-state operations explicit without inventing unresolved original-game behavior.

## 🌈 Evidence already established

- The pinned Pascal and C snapshots contain **379/379 matching hitbox rows**, including action/timer, raw geometry, active flag, strength, damage, and guard damage.
- They contain **305/305 matching hurtbox rows** after coordinate-order normalization.
- The Pascal [Quadro] ledgers contain source-ordered frame, timer/branch, effect, sound, RNG, and simulation-mutation statements for all 15 fighters.
- A source-ordered statement ledger is not yet a fully evaluated tick trace. Nested guards, loops, and cross-calls still need expansion.

## 🌈 Rust contracts introduced

| Contract | Guarantee |
|---|---|
| ActionTimeline | One fighter/action timeline has strictly increasing logical ticks. |
| TickEvents | Events retain insertion/source order; they are not sorted by event category. |
| FixtureRng | Tests consume a supplied sequence in exact call order and fail on exhaustion or invalid range. |
| Fpc322Rng | Normative Pascal/FPC 3.2.2 MTwist; LongInt calls consume one word and Int64 calls consume two words before modulo reduction. |
| DelphiRng | Experimental C-port/Delphi candidate only; it is not the default Rust RNG while Pascal is normative. |
| PascalAttackState | Preserves INI row order and separates strength selector conditions from row force assignments. |
| HitboxRecord | Retains selector key (Pascal action / C state), timer, raw coordinates, active, strength, damage, and guard damage. |
| HurtboxRecord | Retains frame and raw coordinates. |
| BoxUpdate | Makes replace, clear, and retain explicit operations. |
| AttackDeactivated | Sets the attack-active flag false while preserving stored geometry/metadata. |
| Aabb | Implements strict AABB overlap; edge contact alone is not a collision. |

FixtureRng remains a synthetic deterministic test source. Fpc322Rng now models the pinned Pascal project's FPC 3.2.2 MTwist behavior, preserving the different raw-word costs of Random(LongInt) and Random(Int64). DelphiRng remains an isolated candidate for later original-executable comparison, not a default override.

## 🌈 Hitbox and hurtbox lifecycle: source paths now traced

Both ports retain prior box values on a lookup miss, but the implementation details differ. Pascal does so in AplicarCaixas, where an undefined body row or an unmatched attack row causes no write. C does so in sel_boxes, where no matching hurt/hit row also causes no write. See [the lifecycle deep dive](box-lifecycle-deep-dive.md) for the exact selector keys, update order, independent clear paths, and the fighter 11 duplicate-key discrepancy.

A matched row with active=false is not a clear. It stores the row's geometry and metadata while disabling the attack. A lookup miss is different: it leaves all prior fields, including the old active bit, untouched. An explicit state-machine deactivation is a third case: it flips the active bit while retaining geometry and metadata. Rust now represents all three separately and includes a Pascal ordered-row adapter.

The C melee path uses strict AABB comparisons (left < other.right, right > other.left, top < other.bottom, bottom > other.top). The inspected Pascal fight/object/effect paths use the same strict edge-overlap pattern. Rust exposes that geometry as a pure contract; hit consequences are not implemented yet.

Still unverified against the original executable:

- whether the original preserves the Pascal strength-specific selection behavior for fighter 11 action 27 timer 1;
- exact clear sites and whether facing changes on a selector miss retain previously mirrored coordinates;
- activation behavior across state transitions, hit-stop, and same-tick victim re-selection;
- whether the original uses the same strict AABB comparisons in every collision path.

The Rust model exposes Replace, Clear, and Retain as explicit operations, plus AttackDeactivated for a state-machine active-flag write. The current source adapters map a C/Pascal lookup miss to Retain, a matched inactive row to Replace with active=false, and writes such as GolpeAtivo := False / attacking = false to AttackDeactivated. Clear is reserved for actual record removal. These are confirmed port semantics; whether the original executable shares every detail remains an open compatibility check.

## 🌈 Tick timeline normalization

For each fighter/action, the next research pass must produce one record per logical tick:

1. Resolve the action's entry conditions and timer origin.
2. Expand nested guards in source order.
3. Expand loops into individual operations, preserving repeated calls.
4. Number RNG draws in actual execution order, not source-line order alone.
5. Record frame writes, sounds, effects, and simulation mutations in one ordered stream.
6. Record box-table lookup and lifecycle operations at the exact point they occur in the fight loop.
7. Attach source path, pinned revision, and line range to every event.

The static normalizer now covers all 15 fighters and the common script, producing 5,091 symbolic events. It preserves guards, loop regions, static Random sites and width-specific MT word costs. The output does not assign concrete tick numbers or runtime RNG word indices. The first complete pass found 255 unique .rot RNG sites (465 composed timeline occurrences) and 71 RNG-related references in Pascal .pas/.inc source. The normalized JSON is uploaded by the `pascal-rot-symbolic-timelines` CI artifact. The existing 1,056-statement frame ledger remains a complementary index, not a substitute for runtime traces.

## 🌈 Fixture policy

The current Rust tests are **contract fixtures**, not golden gameplay fixtures. They validate ordering and model behavior using deliberately small synthetic examples. A golden fixture may be added only when its expected values are backed by a pinned source path or a controlled original-executable observation.

Recommended fixture groups:

- sequential RNG call-order fixtures;
- action entry and timer-boundary fixtures;
- same-tick multi-event ordering fixtures;
- golden hitbox lifecycle fixtures once original-executable semantics are observed;
- original-backed facing/mirroring and collision-boundary fixtures after the original flow is resolved;
- one representative action per fighter, then exceptional move chains for Seu Madruga, Sr. Furtado, and Jaiminho.

## 🌈 Validation status

- [x] Add dependency-free Rust contract crate.
- [x] Add ordered tick/action event types.
- [x] Add deterministic sequential fixture RNG.
- [x] Implement the normative Pascal/FPC 3.2.2 MTwist RNG with width-aware one-word/two-word calls.
- [x] Add a static parser/normalizer for all 15 Pascal .rot scripts and the common script.
- [x] Preserve full hitbox metadata and raw hurtbox coordinates.
- [x] Model explicit replace/clear/retain operations.
- [x] Add strict AABB overlap geometry from the inspected Pascal/C collision paths.
- [x] Trace retain-on-miss in both ports, including the distinct Pascal row-order and C last-match behavior.
- [x] Model explicit `GolpeAtivo := False` / `attacking = false` separately from a full box clear.
- [x] Add the character 11 state 27 timer 1 selector divergence as a documented open original-compatibility question.
- [x] Run cargo fmt --all -- --check and cargo test --all-targets in GitHub Actions (run #38: 14 unit + 2 integration tests passed).
- [x] Implement DelphiRng's 32-bit LCG transition and multiply-high range reduction with an injected seed.
- [x] Trace C and Pascal lookup misses as retain operations and matched inactive rows as explicit inactive writes.
- [x] Distinguish explicit attack deactivation from clearing the stored box.
- [ ] Use Pascal `--gravar` / `ModoDeSorteio` to capture deterministic dynamic call traces and compare them with Rust.
- [ ] Capture original Chaves.exe RNG state/algorithm in the later compatibility phase.
- [ ] Expand the 15 fighter ledgers into fully evaluated action/tick traces.
- [ ] Confirm the fighter 11 selector divergence against the original executable.
- [ ] Run the PE preflight against the canonical Chaves.exe and hand off its bundle to REA/Ghidra.

## 🌈 Provenance

- Pascal source snapshot: [OpenStreetChaves @ e1b025c35ecbc2b14d8635564aafd23c2107d3d1](https://github.com/danimesq/OpenStreetChaves/tree/e1b025c35ecbc2b14d8635564aafd23c2107d3d1)
- C source snapshot: [open-street-chaves @ e76ad2188af6842ff6618ecdb8f425bba61c9183](https://github.com/danimesq/open-street-chaves/tree/e76ad2188af6842ff6618ecdb8f425bba61c9183)
- Research input: [ordered frame event ledger](ordered-frame-event-ledger.md)
- Collision comparison: [frame, AI and hitbox comparison](frame-ai-hitbox-comparison.md)
