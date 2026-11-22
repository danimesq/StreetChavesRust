# 🌈 RNG call-site inventory: shared stream and tick phase map

## 🌈 Purpose and confidence

This is a source-call-site inventory of the pinned C17/SDL3 reference, not yet an expanded dynamic draw trace. It identifies every subsystem that can advance the single `g->fight.rng` state, then places those call groups into the C fight-tick order.

Pinned C source: `danimesq/open-street-chaves` at `e76ad2188af6842ff6618ecdb8f425bba61c9183`.

The inventory separates:
- **[source]**: the call expression exists at the cited line.
- **[conditional]**: whether it executes depends on guards or runtime state.
- **[loop]**: one source expression may execute many times.
- **[variable draws]**: a single high-level operation may consume a different number of draws between runs.

Do not treat the number of source expressions as the number of runtime draws.

## 🌈 One state, many subsystems

All listed C calls go through `osc_random(g, n)` in `src/fight/fight.c:15-20`, which mutates the same `g->fight.rng` field. The helper `chance(g, c, k)` in `src/fight/cpu.c:10-15` also calls that function. No independent gameplay, presentation, stage, or effect RNG stream is established by this implementation.

| Subsystem | Pinned source lines | Random behavior |
|---|---|---|
| Fight setup | `fight.c:51-69` | Random stage for a two-human matchup; random music when no character/stage-specific music is selected |
| Camera and combo HUD state | `fight.c:145-148,178` | Two draws for camera shake; one conditional draw for combo jitter per eligible fighter |
| CPU strategy | `cpu.c:10-15,41,53-62,77-78` | Preference reroll, nested chance checks, per-character rule chances, optional random move choice |
| Stage objects | `stage.c:14-40` | Mover spawn chance/kind, item drop chance, and two draws per debris fragment |
| Fighter frame selector | `selector.c:48-53,66,79,96,100,118,131,133,172-173,217` | Throw frames/positions, flash, throw hit-stop, random animation frames, run-turn chance, chance-gated frame table rows |
| Fighter motion/physics | `fighter.c:84-85,144,176` | Dust coordinates, landing dust, and optional blood |
| CPU defense / melee | `combat.c:41-74,102-171` | Guard/jump decisions, dust, guard/hit hit-stop, flash and blood |
| Effect creation/update | `effects.c:33-61,102,122-123,207,248-249` | Type-specific lifetime/knock/speed, debris properties, probabilistic sounds, spawn offsets, effect motion and hit visuals |
| Round-end / victory | `fight.c:265-280` | Transition, quote, and victory voice selection |

This table is deliberately grouped by subsystem, not sorted into one global chronological list. Runtime order comes from the callers in the fight loop.

## 🌈 C fight setup and startup seed

- `src/game.c:17`: `osc_game_init` seeds `g->fight.rng` from `SDL_GetTicksNS()`, truncated to 32 bits.
- `src/main.c:143`: the deterministic `--frames` harness overrides that state with `0x5EED1234`.
- `src/fight/fight.c:51-69`: `osc_fight_start` may consume RNG for stage and music selection before the first fight tick.
- `osc_fight_start` does not reinitialize `fight.rng`; the game-state RNG persists between fights.

The deterministic harness seed is a test convenience of the C port. It is not evidence of the original executable's startup `RandSeed`.

## 🌈 C fight-tick phase order

The following is the call order in the pinned `osc_fight_tick` implementation, reduced to phases that can consume the shared RNG:

1. **Tick prelude:** camera shake in `fight.c:145-148`, then combo jitter in `fight.c:178`.
2. **Input and CPU:** human input and sequence processing, then `cpu_think` for eligible CPU fighters. CPU preference, chance and move-range draws happen here.
3. **Stage objects:** `stage_objects` runs before fighter frame selection. Mover/drop decisions and debris can consume draws here.
4. **Fighter frames:** `fighter_frame` is called for both fighters in the current on-top order. When not paused, `select_frame` runs:
   `sel_frames -> sel_stores/sel_code -> special_motion -> sel_run_turn -> fx_spawns_for -> fx_sounds_for -> sel_boxes`.
   Frame-table chance checks, throw logic, effect spawn offsets, effect-specific spawn initialization and probabilistic frame sounds can all advance the stream.
5. **Defense:** `defend` runs for both fighters when the fight is not frozen/blocked. CPU guard and jump decisions consume draws here.
6. **Effect update:** `update_fx` iterates effect slots in order. Effect collision/flash/blood branches can draw, and spawning a secondary effect can trigger more draws inside `spawn_fx`.
7. **Round-end branch:** transition/quote selection at `fight.c:265-267` and victory voice choice at `fight.c:280` can consume draws when their round-end predicates fire.
8. **Melee pass:** `melee_step` runs for fighter 1 and then fighter 2. Guard/hit-stop, flash and blood branches can consume draws here.
9. **Physics:** move clocks, landing cleanup and physics run after melee. Dust/blood draws occur where the relevant fighter/physics path requests them.

The exact phase sequence is essential. A random call in a fighter's `[Quadro]` equivalent can be preceded by stage, CPU, camera, or effect calls in the same tick. The same frame/action can therefore produce different RNG indices when unrelated earlier predicates take different branches.

## 🌈 Variable draw-count traps

### Retry loop in the throw selector

`selector.c:79` repeatedly draws a frame until it differs from the current frame:

```c
do fr = 41 + osc_random(g, 3); while (fr == f->frame);
```

That is one source call expression but can consume one or more draws. Replacing it with one draw plus a deterministic fallback changes the following global RNG state.

### Short-circuit branches

Examples:
- `combat.c:137`: blood RNG is evaluated only when the blood option is enabled and the hit strength predicate passes, unless the unlock condition short-circuits the expression.
- `effects.c:248-249`: flash and blood checks are conditional; blood may be bypassed by an unlock condition.
- `selector.c:133`: a second random draw occurs only if the first draw returns zero.
- `cpu.c:41`: the preference-array draw happens only when the preceding 1-in-13 draw succeeds.
- `effects.c:122-123`: position draws depend on nonzero per-row random offsets; each matching spawn row may also repeat for its configured count.

### Nested spawn randomness

A frame-selector spawn can draw for X/Y offsets in `fx_spawns_for`, then call `spawn_fx`, whose effect-type initializer may draw again. For example, debris type 21 consumes four additional random values inside `spawn_fx` for vertical speed, direction, speed and sprite selection, in addition to any caller-provided position draws.

### Repeated stage debris

`stage.c:40` draws X and Y for each of 30 fragments. Each fragment invokes `spawn_fx`; the effect type may add its own draws. The source-level stage event is not a two-draw event.

## 🌈 Rust consequences

- Use one injected, traceable RNG stream for all simulation phases until original-executable observations establish a safe split.
- Log at least: global draw index, phase, call-site ID, range, returned value, and caller entity (fighter/effect/stage slot).
- Preserve short-circuit evaluation order. Do not eagerly precompute chances.
- Expand retry loops and per-count spawn loops into ordered draw events.
- Keep setup-time draws, fight-tick draws, round-end draws, and presentation draws in the same history unless evidence proves the original used separate state.
- A deterministic seed is necessary but not sufficient. The same call graph and the same branch outcomes are also required.
- Do not derive a Delphi-compatible startup seed from the C port's SDL tick seed or from Pascal/FPC `Randomize`.

## 🌈 Still open

- [x] Identify the C startup seed and deterministic harness override.
- [x] Inventory all direct `osc_random` and `chance` call sites in the pinned fight sources.
- [x] Place the call groups into the C fight-tick phase order.
- [x] Identify retry loops, short-circuit paths and nested effect-spawn draws.
- [ ] Extract every Pascal `Random` call site, including rendering/stage/HUD calls, and place it into the Pascal `Estado7_Luta` order.
- [ ] Produce a normalized per-tick RNG trace for each representative move, then all 15 fighters.
- [ ] Capture the original executable's startup seed and compare the global draw stream.
- [ ] Add original-executable golden RNG traces after the playable Rust baseline.

## 🌈 Pinned source links

- [C fight tick and Delphi RNG](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/fight/fight.c)
- [C game initialization seed](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/game.c)
- [C deterministic harness seed](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/main.c)
- [C CPU chance helper and decision logic](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/fight/cpu.c)
- [C frame selector and special throws](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/fight/selector.c)
- [C fighter movement and physics](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/fight/fighter.c)
- [C melee and defense](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/fight/combat.c)
- [C effects and nested spawn randomness](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6618ecdb8f425bba61c9183/src/fight/effects.c)
- [C stage objects](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6618ecdb8f425bba61c9183/src/fight/stage.c)
