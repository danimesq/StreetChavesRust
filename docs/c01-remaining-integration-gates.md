# C01 remaining integration gates

This document tracks the follow-up work after C01 special-command and AI dispatch. It is intentionally a gate checklist, not a claim that the integration exists.

## 1. Human input / SDL host

**Status: blocked at host integration; simulation entry exists.**

The Rust crate currently has no SDL event loop, input-state module, or game host adapter. The reference Pascal SDL host is documented in `docs/source-inventory.md` and `docs/fight-loop-deep-dive.md`, but that is upstream reference code, not Rust code in this repository.

Available simulation entry:
- `CombatState::dispatch_chaves_special_code(fighter_index, command_code, context)`
- Numeric C01 command IDs 16–24 are defined and tested.

Required host-side steps, once an SDL host exists:
1. Normalize keyboard/gamepad events to logical actions. Do not bind raw SDL scancodes in combat simulation.
2. Preserve held/pressed/released distinctions; special commands should use the source's command recognition rules, not invented button combinations.
3. Gate input according to the fight tick's flash, hit-stop, intro/notice, pause, and clock state.
4. Dispatch the accepted logical special command through the numeric C01 entry.
5. Clear sampled input on focus loss and test controller disconnect/reconnect.
6. Keep event repeat and render FPS from advancing combat action frames.

No SDL scancode-to-special mapping is declared here because the Rust host/input layer does not yet exist and the mapping/recognition needs a source audit.

## 2. Melee hit-resolution integration

**Status: hitbox/hurtbox contracts and C01 reaction hook exist; no Rust melee resolution loop yet.**

The project already models box geometry/lifecycle contracts and has `CombatState::apply_c01_hit_reaction(attacker_index)`. The hook is tested for C01 Special 2 strong, but no runtime melee resolver currently detects an active attack/hurtbox overlap and calls the hook.

The future resolver must follow the Pascal order recorded in `docs/box-lifecycle-deep-dive.md`:
- resolve guard vs successful hit;
- deactivate the attack on the appropriate branch;
- apply guard/chip/hit-stop and victim action/box refresh in source order;
- invoke character `[Acerto]` only after a successful hit and after the preceding combat-state writes;
- call the C01 reaction hook only for a confirmed C01 Special 2 strong hit;
- preserve box retain/inactive/clear distinctions and table order.

Do not call the C01 hook from frame animation, command entry, or collision-candidate-only code. It must be invoked by the eventual confirmed-hit path.

## 3. Effect collisions

**Status: live effects and render snapshots exist; collision consequence integration is pending.**

`EffectArena::advance` owns effect movement, snapshots, lifetime and culling. Its source comments explicitly leave effect-vs-effect and effect-vs-fighter consequences outside the current implementation.

Next implementation must audit the Pascal effect loop and hit-resolution source at the pinned revision, then add source-ordered candidate detection and consequence writes. Avoid treating a render snapshot as authoritative simulation state, avoid duplicate ticking, and do not invent damage/guard/owner rules for effects whose source has not been inspected.

## 4. Pascal C01 checkpoints

**Status: none of the checkpoint files below are captured yet. No golden values are fabricated.**

Capture source-derived checkpoints from the Pascal runtime with its existing trace facility, then separately capture original-executable evidence when available. Each checkpoint should include:
- reference repo/commit and runtime build identity;
- tick and fighter index, character/action/frame/duration, sprite frame, facing, force;
- position, body/hitbox state and active flags;
- health, power, charge and jump/air state;
- effect arena slots before/after, sound/flash/shake requests;
- ordered RNG draws including width, range, value and parent/child call site;
- command input, eligibility/rejection and ordered state writes.

Suggested fixture naming: `tests/fixtures/c01-entry-<case>-pascal.jsonl`. Cases: accepted/rejected Special 1 with type-7 effect ownership permutations; each special weak/strong entry; final-special power/health gates and Word wrap; walking `[AcaoFim]`; each of the six independent C01 AI conditions, both selection branches, and overlapping conditions to prove source order/RNG consumption.

### Checkpoint placeholder schema

This is a shape guide only. It is not a capture and has no expected values:

```json
{
  "schema": "street-chaves-c01-checkpoint-v1",
  "evidence": "not-captured",
  "source": {
    "repository": "OpenStreetChaves",
    "commit": "e1b025c35ecbc2b14d8635564aafd23c2107d3d1"
  },
  "case": "replace-with-case-name",
  "tick": null,
  "input": null,
  "state_before": null,
  "ordered_writes": [],
  "rng_draws": [],
  "state_after": null
}
```

## Acceptance gate

- [ ] Rust SDL host/input adapter exists and dispatches human C01 specials.
- [ ] SDL focus loss/disconnect and input gating tests pass.
- [ ] Pascal C01 command/AI checkpoint fixtures are captured and replayable.
- [ ] Melee resolver calls `[Acerto]` and C01 reaction only after confirmed hit.
- [ ] Effect-vs-fighter and effect-vs-effect consequences are source-audited and implemented.
- [ ] Differential tests compare state writes and RNG sequence; discrepancies are explained rather than waived.
