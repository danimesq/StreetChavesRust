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
| HitboxRecord | Retains frame, timer, raw coordinates, active, strength, damage, and guard damage. |
| HurtboxRecord | Retains frame and raw coordinates. |
| BoxUpdate | Makes replace, clear, and retain explicit operations. |
| Aabb | Implements strict AABB overlap; edge contact alone is not a collision. |

The fixture RNG is **not** a Delphi RNG implementation. It is a deterministic injection point for tests. The exact original RNG algorithm, seed initialization, and call sequence remain separate research tasks.

## 🌈 Hitbox and hurtbox lifecycle: unresolved on purpose

The current cross-port table comparison proves row agreement, not a shared lifecycle implementation. The pinned C selector gives one concrete port-specific rule: `sel_boxes` scans rows for the selected frame/state/timer and writes matching fields, but does not clear the previous box fields before scanning. Its fighter initialization supplies a fallback hurtbox. Therefore, in this C snapshot, a lookup with no matching row leaves the previous box values in place. This is **[port-specific]**, not yet promoted to original-game behavior or assumed for the Pascal port.

The Pascal `desenhalutador` calls `AtualizaQuadro` from its draw path, and the fight loop later checks `GolpeAtivo` and the current attack/body coordinates. The exact per-frame assignment/clear behavior inside the script/data update path still needs its own source trace.

The C melee path also uses strict AABB comparisons (`left < other.right`, `right > other.left`, `top < other.bottom`, `bottom > other.top`). The Pascal fight/object/effect paths inspected use the same strict edge-overlap pattern. The Rust crate now exposes this as a pure geometry contract; it does not yet implement hit resolution.

It does not prove:

- whether the Pascal port clears or retains a previous box when a frame has no row;
- whether the C port's observed stale-box retention matches the original executable;
- whether active=false removes a hitbox or leaves a disabled row in place;
- when attack boxes are sampled relative to action timers and collision detection;
- whether facing changes mirror coordinates before or after collision checks;
- whether collision edges are inclusive or exclusive;
- whether a hitbox can remain active across a state transition or hit-stop.

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
