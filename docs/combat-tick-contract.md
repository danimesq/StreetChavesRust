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
| DelphiRng | Implements the Delphi 5 32-bit LCG transition and multiply-high range reduction for an injected seed. |
| PascalAttackState | Preserves INI row order and separates strength selector conditions from row force assignments. |
| HitboxRecord | Retains selector key (Pascal action / C state), timer, raw coordinates, active, strength, damage, and guard damage. |
| HurtboxRecord | Retains frame and raw coordinates. |
| BoxUpdate | Makes replace, clear, and retain explicit operations. |
| Aabb | Implements strict AABB overlap; edge contact alone is not a collision. |

FixtureRng remains a synthetic deterministic test source. DelphiRng now implements the Delphi 5 32-bit recurrence and multiply-high range reduction, with the initial RandSeed supplied by the caller. The recurrence is implemented; original startup seeding and the full original call sequence remain separate research tasks.

## 🌈 Hitbox and hurtbox lifecycle: source paths now traced

Both ports retain prior box values on a lookup miss, but the implementation details differ. Pascal does so in AplicarCaixas, where an undefined body row or an unmatched attack row causes no write. C does so in sel_boxes, where no matching hurt/hit row also causes no write. See [the lifecycle deep dive](box-lifecycle-deep-dive.md) for the exact selector keys, update order, independent clear paths, and the fighter 11 duplicate-key discrepancy.

A matched row with active=false is not a clear. It stores the row's geometry and metadata while disabling the attack. A miss is different: it leaves all prior fields untouched. Rust now represents these separately and includes a Pascal ordered-row adapter.

The C melee path uses strict AABB comparisons (left < other.right, right > other.left, top < other.bottom, bottom > other.top). The inspected Pascal fight/object/effect paths use the same strict edge-overlap pattern. Rust exposes that geometry as a pure contract; hit consequences are not implemented yet.

Still unverified against the original executable:

- whether the original preserves the Pascal strength-specific selection behavior for fighter 11 action 27 timer 1;
- exact clear sites and whether facing changes on a selector miss retain previously mirrored coordinates;
- activation behavior across state transitions, hit-stop, and same-tick victim re-selection;
- whether the original uses the same strict AABB comparisons in every collision path.

The Rust model therefore exposes Replace, Clear, and Retain as explicit operations. An adapter must map source behavior to one of them only after the relevant update and collision flow is established. Missing data must not silently mean Retain.

## 🌈 Tick timeline normalization

For each fighter/action, the next research pass must produce one record per logical tick:

1. Resolve the action's entry conditions and timer origin.
2. Expand nested guards in source order.
3. Expand loops into individual operations, preserving repeated calls.
4. Number RNG draws in actual execution order, not source-line order alone.
5. Record frame writes, sounds, effects, and simulation mutations in one ordered stream.
6. Record box-table lookup and lifecycle operations at the exact point they occur in the fight loop.
7. Attach source path, pinned revision, and line range to every event.

The existing 1,056-statement ledger is the input index, not a substitute for this normalized trace.

## 🌈 Fixture policy

The current Rust tests are **contract fixtures**, not golden gameplay fixtures. They validate ordering and model behavior using deliberately small synthetic examples. A golden fixture may be added only when its expected values are backed by a pinned source path or a controlled original-executable observation.

Recommended fixture groups:

- sequential RNG call-order fixtures;
- action entry and timer-boundary fixtures;
- same-tick multi-event ordering fixtures;
- hitbox replace/clear/retain lifecycle fixtures once source semantics are known;
- facing/mirroring and collision-boundary fixtures after the original flow is resolved;
- one representative action per fighter, then exceptional move chains for Seu Madruga, Sr. Furtado, and Jaiminho.

## 🌈 Validation status

- [x] Add dependency-free Rust contract crate.
- [x] Add ordered tick/action event types.
- [x] Add deterministic sequential fixture RNG.
- [x] Preserve full hitbox metadata and raw hurtbox coordinates.
- [x] Represent box lifecycle operations explicitly without choosing an unknown cross-port default.
- [x] Add strict AABB overlap geometry from the inspected Pascal/C collision paths.
- [x] Record the C selector's observed stale-box retention as port-specific evidence.
- [ ] Run cargo fmt --check and cargo test in a Rust toolchain environment.
- [ ] Expand the 15 fighter ledgers into fully evaluated action/tick traces.
- [ ] Resolve box retention and activation from the combat update path.
- [ ] Implement and verify the Delphi RNG adapter.
- [ ] Run the PE preflight against the canonical Chaves.exe and hand off its bundle to REA/Ghidra.

## 🌈 Provenance

- Pascal source snapshot: [OpenStreetChaves @ e1b025c35ecbc2b14d8635564aafd23c2107d3d1](https://github.com/danimesq/OpenStreetChaves/tree/e1b025c35ecbc2b14d8635564aafd23c2107d3d1)
- C source snapshot: [open-street-chaves @ e76ad2188af6842ff6618ecdb8f425bba61c9183](https://github.com/danimesq/open-street-chaves/tree/e76ad2188af6842ff6618ecdb8f425bba61c9183)
- Research input: [ordered frame event ledger](ordered-frame-event-ledger.md)
- Collision comparison: [frame, AI and hitbox comparison](frame-ai-hitbox-comparison.md)
