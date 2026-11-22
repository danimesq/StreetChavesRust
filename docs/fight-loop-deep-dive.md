# Fight-loop deep dive: update order, frames, collisions, and effects

## Scope

This is a source-level comparison of the fight path in the two SDL projects. For the Rust reconstruction, Pascal/OpenStreetChaves is the normative source; C17/SDL3 is comparative only. Any conflict is recorded and left for later original-executable verification rather than resolved by silently preferring C.

Primary sources inspected:

- Pascal/SDL2: `source/ChavesSDL.pas`, `source/ChavesJogo.pas`, `source/game/Estado7_Luta.inc`, `source/game/Estado7_Golpes.inc`, `source/game/Comum_Quadro.inc`, and `source/game/Comum_Sprites.inc`.
- C17/SDL3: `src/fight/fight.c`, `fighter.c`, `combat.c`, `selector.c`, `effects.c`, and `render/fight_draw.c`.

This is not yet a byte-for-byte differential validation against the original executable. Pascal is normative for the Rust project by explicit policy, and the pinned project build requirement identifies FPC 3.2.2. C's decompiler-derived annotations remain useful comparative evidence, but they do not override Pascal. The original executable may justify a later, explicitly documented correction.

---

## 1. SDL host loop versus game tick

### Pascal + SDL2

`ChavesSDL.pas` owns the platform loop and keeps the game object separate:

- SDL events update physical scancode state and are forwarded to `TJogo.TeclaSDL`.
- Gamepads and generic joysticks are normalized into the game's compact input bitmask.
- Losing window focus clears the host key array and calls `SoltarTeclas`, preventing stuck inputs.
- The host schedules `TJogo.PassoSDL` at the configured interval, documented as the original 28 ms.
- The game renders to its own logical 400×300 surface; the host uploads/presents the pixel buffer through an SDL texture.

This is a useful boundary, but not a fully pure simulation boundary: the game object still owns drawing and game state together.

### C17 + SDL3

The C implementation's fight logic is driven by `osc_fight_tick(g, in1, in2)`. The platform loop supplies normalized input bytes. The fight tick is an explicit function and can be called headlessly.

The SDL3 implementation therefore makes the simulation boundary easier to test, even though some rendering-related data is still prepared during simulation.

### Rust decision

Keep the host/tick split from Pascal and the explicit headless tick from C:

- `chaves-sdl3` receives events and samples devices.
- It translates them to logical actions.
- A scheduler calls `Game::tick(actions)`.
- The simulation returns a render snapshot and semantic audio events.
- Rendering and audio playback consume those outputs but do not mutate game state.

Do not let SDL event repeat rates or presentation FPS determine fighter animation time.

---

## 2. Fight tick order

The order of operations is observable gameplay. Reordering two apparently independent systems can change hit timing, random-number consumption, effect creation, or which fighter is drawn on top.

### Pascal order in `TJogo.Estado7_Luta`

The inspected procedure runs the following sequence:

1. `L7_Prepara`: establishes the ground coordinate and round-one entrance jump; initializes the boss dialogue when applicable.
2. `L7_CenarioECamera`: handles pending music/sounds, special flash presentation, camera tracking/shake/clamps, and stage drawing.
3. `L7_Relogio` and both upper HUD passes.
4. Human input through `L7_Entrada`, gated by flash, hit-stop, player-entry notice, and paused clock.
5. `sequenciagolpe(1/2)`: updates directional charge/sequence state and evaluates character command rules.
6. CPU decisions for inactive players, under their own pause/clock gates.
7. Fly-by and bonus-item processing, then background stage-object collision checks.
8. Fighter drawing in `DesenhaPorCima` order. Importantly, `desenhalutador` calls `AtualizaQuadro` when neither flash nor hit-stop is active. The apparent drawing step therefore also advances animation/game state.
9. Human/CPU defense checks through `defendergolpe`.
10. Foreground stage objects, then `desenhaefeitos`. This routine animates, moves, draws, collides, and ages effects in one ordered pass.
11. Lower HUD.
12. Time-over and round-intro/round-end logic.
13. Body hit resolution through `L7_Acertos(1/2)`.
14. Hit-stop countdown, then `L7_Fisica` for action completion, jump/knockdown motion, landing, and screen bounds.
15. Boss dialogue, blink counter, demo handling, flash overlay, and optional step-by-step move display.

The code's current organization is historically faithful to the game, but it couples simulation to drawing. The Rust implementation should preserve the order while separating responsibilities.

### C17 + SDL3 order in `osc_fight_tick`

The inspected C function is more explicit:

1. Increment fight tick; handle demo/challenge early-return cases and set current input.
2. Initialize the round-one entrance jump and boss dialogue.
3. Play the sound requested by the previous tick.
4. Update super-pause presentation state and camera/shake/clamps.
5. Update meter, stocks, health-bar trail, and game timer.
6. Dispatch human input, evaluate command sequences, then run CPU decisions.
7. Update input/sequence/combo counters.
8. Update stage objects.
9. Run `fighter_frame` for both fighters in `on_top` order.
10. Run defense checks.
11. Run `update_fx`.
12. Update round intro, time-over, and round-end/match transitions.
13. Resolve body hits with `melee_step` for both fighters.
14. Decrement hit-stop; advance move clocks.
15. Clamp/resolve landing, then run fighter physics.
16. Update dialogue skip, previous input, blink/flash counters, and audio backend.

### Important comparison

The broad phases are similar: input and commands precede fighter animation selection; defense and effects precede body-hit resolution; physics follows hit resolution.

However, the implementations are not interchangeable:

- Pascal's `desenhalutador` invokes animation advancement from the rendering path. C calls `fighter_frame` explicitly inside the tick and leaves `osc_fight_render` to consume the result.
- Pascal's `desenhaefeitos` draws each effect before resolving its collisions and lifetime. C's `update_fx` creates a per-slot draw snapshot at the equivalent point, then performs later collision/cleanup work; `osc_fx_render` draws the snapshot.
- Both process effects before the later body-hit pass, but the exact state and pause gates differ in individual branches.
- C's order is explicit in one tick function. Pascal's order is distributed across `Estado7_Luta`, `Comum_Sprites`, and the state-specific helpers.

**Rust requirement:** define a named, documented phase sequence and tests for it. Do not make the renderer call simulation functions.

---

## 3. Fighter state, move selection, and frame selection

### Pascal: command rules and scripts

The Pascal path is data-driven at the character level:

- Each fighter's INI data supplies move names, command sequences, button rules, and other character properties.
- `sequenciagolpe` maintains directional hold counters, evaluates `[Sequencias]` rules in order, advances a sequence state, and expires it with a short timer.
- `golpe` evaluates `[Botoes]` rules in order and invokes the configured `AcaoJogador` command.
- `L7_Acertos` calls `ExecutarAcerto(Jog)` after a hit, allowing character-specific `.rot` scripts to alter combo time, spin, recoil, launch, or chain into another action.
- `desenhalutador` sets the current action/frame context and calls `AtualizaQuadro` before selecting the sprite to draw.

The important idea is not that Rust should implement Pascal's script language. It is that fighter-specific behavior is content-driven and that command evaluation order matters.

### C: explicit move and frame pipeline

The C code uses generated move/animation tables plus explicit routines:

- `set_move` first clears blocking, handles several hard-coded special states (including knockdown/lie-down and guard), then searches character-specific rows before common rows.
- A move row can constrain source states, require the fighter to be grounded/not airborne, consume super stocks, set a sound, add meter, or require an existing effect condition.
- `fighter_frame` updates facing for selected neutral states, calls `select_frame` when not paused, and updates rotation.
- `select_frame` calls the following helpers in this exact order: `sel_frames`, `sel_stores`, `special_motion`, `sel_run_turn`, `fx_spawns_for`, `fx_sounds_for`, `sel_boxes`.

This is a major discovery for the Rust design: frame selection is not just choosing a sprite index. It also applies special movement, triggers effect/sound events, and refreshes collision boxes.

### Architectural implication

Do not create a Rust API where animation is a passive `Vec<SpriteFrame>` and the renderer simply increments an index.

The compatibility model needs a simulation-side frame-selection operation that can produce:

- selected sprite/frame ID;
- current hitbox and hurtbox values;
- movement or velocity changes associated with that animation tick;
- semantic effect-spawn events;
- semantic sound events;
- any state transition caused by frame/timer conditions.

Keep the renderer read-only. The render snapshot can contain the selected frame and pose, but the simulation owns the decision.

### A subtle compatibility hazard: draw order and RNG

The Pascal drawing path uses `Random` for jitter, stage animation placement, and other visual details. Its source comments explicitly note that the order of stage-animation draws and random calls follows the data file. The C project also has visual/effect-related random calls in simulation code.

If the original shares one RNG stream between gameplay and presentation, moving cosmetic randomness to a separate renderer RNG may change later gameplay randomness. Conversely, keeping all cosmetic RNG in the renderer would make headless simulation nondeterministic relative to the original.

**Important RNG distinction:** this Pascal project is built with Free Pascal/Lazarus. Its ChavesRoteiro.SortearValor wrapper calls FPC System.Random, whose RTL documentation describes a Mersenne Twister implementation. The original Delphi 5 executable uses a different 32-bit generator; the C project implements the Delphi 5 LCG recurrence. Therefore the Pascal port's emitted random values are not a valid sequence oracle for the original.

**Open compatibility task:** map every RNG call in the fight path, classify it as gameplay-affecting or presentation-only, and determine whether the original uses one shared stream. Do not split RNG streams until this is established. See [RNG compatibility](rng-compatibility.md) for the recurrence, seed caveats, and references.

---

## 4. Collision and hit resolution

### Shared geometric model

Both SDL implementations use axis-aligned rectangles and strict overlap comparisons.

For body attacks, the attack rectangle is positioned relative to the fighter's feet/anchor and sprite height; the target rectangle is the opponent's hurt/body rectangle. The overlap condition is effectively:

```text
attack.left < target.right
attack.right > target.left
attack.top < target.bottom
attack.bottom > target.top
```

Touching edges alone is not a hit. This strictness matters for frame-exact tests.

Both implementations also distinguish:
- body/hurt boxes;
- active attack boxes;
- effect/projectile boxes;
- invulnerability;
- standing and crouching guard;
- chip damage;
- hit-stop;
- knockdown and special hit reactions.

### Pascal body-hit path

`L7_Acertos`:

1. Applies recoil/dust behavior to a fighter already in a hit reaction.
2. Builds the attack and opponent body rectangles using rounded fighter coordinates.
3. Requires `GolpeAtivo`, a non-invulnerable opponent, strict rectangle overlap, and no flash/hit-stop.
4. Opens the wrong guard for low or airborne attacks.
5. If the opponent is guarding, emits a guard sound/effect, pushes the defender, applies chip damage when the clock is live, clears the attack, and may cause a guard KO.
6. If the hit lands, updates facing, score, meter, combo counters, damage, health-bar delay, optional flash/blood, first-hit bonus, draw order, and reaction state.
7. Calls the character-specific `ExecutarAcerto` script.
8. Applies hit-stop according to hit strength and updates the victim's frame.
9. Pushes the attacker away from a cornered opponent.

The reaction path is notably script-extensible. A generic hit reaction can be overridden or chained by character-specific rules.

### C body-hit path

`melee_step` uses the same strict AABB overlap model and checks attacking, invulnerability, and hit-stop/super-pause. It then handles:

- guard mismatch for low and jump attacks;
- guard sound/effect, pushback, chip damage, attack cancellation, and guard hit-stop;
- score, meter, combo and damage;
- character/state/timer-specific throw rotation, recoil distance, vertical launch, and hit-stop;
- first-hit bonus, on-top order, and multi-part special chains;
- victim move selection through `set_move`.

The C code makes many exceptions explicit in C branches keyed by character, state, and animation timer. The Pascal implementation delegates a portion of that customization to the per-character `.rot` script. These are different representations of behavior and may encode different levels of recovered detail.

### Projectile/effect collision is a separate system

Do not route every attack through the melee routine. The effects loop independently checks effect boxes against:
- other active effects, for clashes/reflection/cancellation;
- fighter body boxes, for guard, hit, knockdown, and pickups.

This matters for multi-hit projectiles, persistent effects, reflecting effects, and effects whose collision box is re-armed at specific phases.

### Rust requirement

Implement collision geometry as pure functions first, but keep hit resolution in an ordered combat system. Geometry can be shared; consequences cannot be collapsed into one generic `apply_damage()` without losing guard, chip, hit-stop, score, combo, launch, and character-specific behavior.

Add boundary tests for:
- one-pixel overlap;
- exact edge contact (must not overlap under strict comparisons);
- mirrored/facing-left coordinates;
- airborne versus grounded hurt boxes;
- guard mismatch;
- invulnerability;
- collision during hit-stop.

---

## 5. Effects: animation, movement, collision, and lifetime

### Pascal: one ordered draw/update/collision pass

`desenhaefeitos` iterates effect slots in ascending order. For each active slot it:

1. Calls `AnimarEfeito` to select the sprite/stage.
2. Updates special hitbox phases for specific effect types.
3. Repositions orbiting effects (types 31/32) relative to their owner.
4. Applies type-specific conditions and gravity.
5. Moves the effect when flash/hit-stop gates allow.
6. Draws debug boxes and the sprite/text.
7. Checks effect-vs-effect collisions, including ownership transfer/reflection and cancellation.
8. Checks effect-vs-fighter collisions, including guard, damage, knockdown, and pickup healing.
9. Advances lifetime/countdown and removes expired or out-of-bounds effects.

Because drawing occurs before later collision/lifetime cleanup in the same slot pass, an effect can be visible in the tick in which it is consumed or removed. Slot order also matters because an earlier effect may modify a later slot before that later slot is processed.

### C: simulation-side snapshot

`update_fx` also iterates slots in order, but creates an `fx_draw` snapshot at the point where the original routine would draw. It then processes effect clashes, fighter collisions, lifetime, countdowns, and culling.

`osc_fx_render` later draws only the snapshot. This preserves the visual fact that an effect may still be rendered for the current tick even if the live effect slot was cleared later in that tick.

The C implementation additionally makes the type table explicit: spawn defaults include speed, damage, life, hitbox flags, gravity, persistence, and per-type special cases. Some effect types re-arm hitboxes at selected phases, orbit a fighter, reflect other projectiles, or spawn secondary effects.

### Differences to investigate

- Pascal's `E.Tipo`, `PodeAcertar`, `Terminando`, `Flag50`, and `Flag52` map conceptually to C's `type`, `has_hitbox`, `persist`, `reflect`, and `gravity`, but this is not proof of a one-to-one field mapping.
- Pascal's effect animation uses `Quadro` and `Duracao`; C uses `phase`, `life`, and generated sprite rows. Verify off-by-one behavior per effect type.
- Both use a fixed slot array and first-free-slot spawning. A Rust `Vec` that freely compacts/reorders effects would alter processing order and potentially collision results.
- The C code documents a stale sprite-byte quirk on slot reuse. Do not reproduce it blindly; determine whether it is visually observable and required for compatibility.
- The Pascal and C code differ in some visible per-type details (for example, type 5 text motion is gated differently around flash/hit-stop). These should be logged as discrepancies, not silently normalized.

### Rust requirement

Start with a fixed-capacity ordered effect arena, not an unordered collection. Model each tick as an ordered pass and generate render snapshots before any later removal that should not erase the current tick's visual.

Keep effect data tables versioned and validate references. Do not invent an effect type or simplify per-type timing without evidence.

---

## 6. Physics and movement

Both projects use a logical ground coordinate of 300 in the inspected fight path and a gravity increment of 4 in their documented/current implementation.

### Pascal

`L7_Fisica`:
- advances finite action duration unless flash/hit-stop is active;
- resets completed actions and clears attack/guard/invulnerability flags;
- resolves below-ground correction;
- advances airborne vertical motion and gravity;
- applies horizontal jump direction through `Empurra`;
- handles knockdown travel and ground impact;
- handles bounce/recoil and landing;
- clamps living fighters to the camera's horizontal play bounds.

### C

`physics`:
- advances airborne y and vertical velocity;
- applies gravity 4;
- applies horizontal jump direction through `move_x`;
- handles knocked-up state 19 and bounce state 20 separately;
- emits dust and impact sounds, including stronger screen shake for selected fighters;
- resolves landing and clears air/attack state;
- clamps living fighters to the camera's horizontal bounds.

The implementations share a recognizable model, but their state IDs, action timers, and character-specific branches must be translated through explicit mapping tables. Do not assume numeric state IDs have the same meaning in every internal representation.

---

## 7. Consequences for the Rust design

The source inspection changes the initial architecture in several concrete ways:

1. **Simulation phases must be explicit.** The tick order is part of compatibility, not an implementation detail.
2. **Frame selection belongs to the simulation.** It can change hitboxes, motion, effects, sounds, and state.
3. **Rendering must be read-only.** Build a snapshot after simulation decisions; never advance animation from a draw call.
4. **Effects need stable slot order.** Preserve first-free-slot allocation and in-order processing until differential tests prove a different representation safe.
5. **Separate melee from effect collision.** They share geometry but not consequence rules.
6. **RNG ownership needs investigation.** Visual calls may consume the same original RNG stream as combat.
7. **Data-driven content is essential.** Use typed tables/configuration for fighter moves, frame selection, effect defaults, and per-character exceptions, while keeping any hand-coded compatibility rules traceable.
8. **Do not port either project's bugs by assumption.** First establish whether a quirk is original behavior, a reverse-engineering interpretation, or a port artifact.

## 8. Immediate test plan

- [ ] Record one complete neutral tick and verify the order of input, frame selection, defense, effects, melee, and physics.
- [ ] Capture frame/timer/hitbox values around one basic attack's active window.
- [ ] Verify the strict AABB edge behavior against the original executable.
- [ ] Compare a blocked hit and an unblocked hit, including chip damage and hit-stop duration.
- [ ] Compare one projectile clash, one reflection, and one persistent multi-hit effect.
- [ ] Verify effect slot reuse and whether the consumed effect is still drawn for the current tick.
- [ ] Trace RNG calls in a deterministic scenario and test whether cosmetic effects alter later combat outcomes.
- [ ] Compare the two SDL projects against the same original-game observations before deciding which differences to reproduce.

---

## Source links

### Pascal + SDL2

- [SDL host and input](https://github.com/danimesq/OpenStreetChaves/blob/main/source/ChavesSDL.pas)
- [Game orchestration and state includes](https://github.com/danimesq/OpenStreetChaves/blob/main/source/ChavesJogo.pas)
- [Fight state and tick flow](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado7_Luta.inc)
- [Move commands, sequences, and CPU](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado7_Golpes.inc)
- [Fighter/effect rendering and effect processing](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Comum_Sprites.inc)
- [State dispatch and key memory](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Comum_Quadro.inc)

### C17 + SDL3

- [Fight tick and round flow](https://github.com/danimesq/open-street-chaves/blob/main/src/fight/fight.c)
- [Move setup, clocks, and physics](https://github.com/danimesq/open-street-chaves/blob/main/src/fight/fighter.c)
- [Melee and guard resolution](https://github.com/danimesq/open-street-chaves/blob/main/src/fight/combat.c)
- [Frame selection pipeline and special motion](https://github.com/danimesq/open-street-chaves/blob/main/src/fight/selector.c)
- [Effect spawning, update, collision, and snapshots](https://github.com/danimesq/open-street-chaves/blob/main/src/fight/effects.c)
- [Read-only fight/effect renderer](https://github.com/danimesq/open-street-chaves/blob/main/src/render/fight_draw.c)
