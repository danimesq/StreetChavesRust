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


## 1a. Shared logical input sampler (Rust foundation)

**Status: implemented in `src/input.rs`; physical host adapters and source command recognition remain pending.**

The shared core now exposes host-neutral `Direction`, 32 logical button slots, normalized `InputEvent`, immutable per-tick `InputFrame`, and `InputSampler`. It tracks held state, one-shot pressed/released edges and saturating hold ticks. Duplicate host down events are ignored, and a reset event clears state for focus loss/controller disconnect.

The module deliberately has no SDL, browser, scancode, key-name, device ID, or fighter-command mapping. SDL/native and browser hosts must map their physical controls to the same logical slots. Command recognition, command sequence counters and gating still require the source audit and simulation integration. This foundation advances compatibility without adding SDL or browser dependencies to the native core.

Tests cover edge-once behavior, per-logical-tick hold counters, reset, and invalid slot rejection. Host adapters must additionally test key/controller mapping, simultaneous direction composition, focus/device lifecycle, and tick-boundary sampling.

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

**Status: partial; full original effect parity is pending.**

`EffectArena` currently supports spawning and advancing only eight effect IDs: 2, 7, 8, 13, 15, 16, 18 and 19. `Dados/Efeitos.ini` defines 40 effect IDs: 1–36 for the base game, 37–38 for the Chapolin C16 extension, and 39–40 for breakable-stage wood/glass debris.

The source audit in `source/game/Comum_Sprites.inc` shows the ordered pass: evaluate each `AnimN` rule, perform type-specific writes (including type 8/28 hit windows, 31/32 owner orbit, type 30/34 termination and type 24 drift), apply gravity/movement, draw using sprite/facing/anchor/flash rules, resolve effect-vs-effect collisions/reflection through `Flag50`, resolve fighter guard/hit/knockback/item-healing consequences, then advance lifetime counters and cull by camera/ground bounds.

The Rust runtime does not yet reproduce all 40 definitions, the full ordered animation rules, visual sprite drawing, effect-vs-effect reflection/destruction, effect-vs-fighter damage/guard/pickup behavior, or the remaining per-type physics. The next implementation must port the INI definitions and source loop in phases, preserving effect-slot order, RNG draw widths and mutation order. Render snapshots must stay read-only; collision outcomes belong to authoritative live effect/fighter state.

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
