#🌈 CPU AI source parity: Common, fifteen fighters and projectile defense

##🌈 Scope and source authority

The CPU path follows the Pascal/Lazarus source in [OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves), not a newly invented behavior tree. The source-controlled files are:

- [Common AI and shared actions](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/Comum.rot)
- [CPU difficulty and projectile defense](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado7_Golpes.inc)
- [Fight-state update order](https://github.com/danimesq/OpenStreetChaves/blob/main/source/game/Estado7_Luta.inc)
- Character scripts: [C01](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C01.rot), [C02](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C02.rot), [C03](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C03.rot), [C04](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C04.rot), [C05](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C05.rot), [C06](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C06.rot), [C07](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C07.rot), [C08](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C08.rot), [C09](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C09.rot), [C10](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C10.rot), [C11](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C11.rot), [C12](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C12.rot), [C13](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C13.rot), [C14](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C14.rot), [C15](https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C15.rot).

StreetChavesRust embeds the source-pinned ROT snapshots under `research/pascal-rot/` and parses them into the existing typed ROT AST. The CPU does not contain a separate hand-maintained list of each fighter's attack probabilities.

##🌈 Tick order

For an unjoined second-player slot, the source-driven fight tick follows the relevant order from `Estado7_Luta`:

1. Process human input.
2. Update the two fighters' special-command sequence counters.
3. Run `CpuNovo`: `Comum.rot [IA]`, followed by the selected `Cnn.rot [IA]`.
4. Advance the fighter frame scripts.
5. Run the CPU branch of `defendergolpe`.
6. Advance the live effect arena.

An incoming P2 credit disables CPU control and transfers that slot to the human player. The CPU slot's physical P2 keyboard sample is ignored while the CPU owns it.

##🌈 Common and character-specific AI

Every CPU invocation supplies the original five input values to the ROT execution frame:

| Pascal input | StreetChavesRust source |
|---|---|
| `Dist` | Absolute distance between rounded fighter X coordinates |
| `DistProj` | Absolute distance from the CPU fighter to the last matching enemy projectile in effect-slot order |
| `OppY` | Rounded opponent Y coordinate |
| `Ameaca` | 1 if at least one other-owned, hit-capable effect exists; otherwise 0 |
| `Fator` | Exact CpuNovo difficulty factor |

The common section chooses a desired spacing, moves toward or away from the opponent, and evaluates the four basic attacks plus crouch/attack probabilities. Then the selected C01-C15 section evaluates that character's actual specials and unique conditions.

The same ROT host handles `AcaoJogador`: Common `[Acao]`, character `[Acao]`, and Common `[AcaoFim]` execute in source order, including nested action calls. All nested scripts share one RNG cursor and the live fighter/effect state.

The scripts are parsed once when a fight is constructed and cached for the 28 ms simulation ticks. They are not reparsed every frame.

##🌈 Exact F1 difficulty formulas

The original uses two distinct factors. They must not be collapsed into a single tuning value.

For `CpuNovo` in `Estado7_Golpes.inc`:

`Fator = (8 - NivelJogo) × 0.6 + 1.3 - Fase × 0.1`

For the CPU branch of `defendergolpe`:

`Fator = max(0, (8 - NivelJogo) × 0.9 + 1.2 - Fase × 0.1)`

- `NivelJogo` is the F1 difficulty level, 1..8.
- `Fase` is the campaign opponent index, 1..8.
- `FightSimulation::set_cpu_campaign_phase` stores the campaign phase. StreetChavesRust currently starts at 1 because the complete `CHS_ADVERSARIOS` opponent-order/campaign state has not yet been implemented.

The regression test checks both formulas at level 4 / phase 1 and level 8 / phase 8.

##🌈 Projectile-aware defense

The CPU reads the live `EffectArena`, filtering for:

- nonzero effect type;
- `can_hit == true`;
- owner different from the CPU's player slot.

The iteration intentionally continues to the end. As in Pascal, each matching effect overwrites `DistProj` and `ProjY`, so the last matching slot supplies those values. `Terminando` remains true if any matching effect is terminating.

The CPU then follows the source decisions:

- Defend against an active opponent fighter attack inside 160 pixels, selecting standing or low defense using the original height tests and independent random checks.
- Jump when a projectile is within the original horizontal/vertical window, with the same `Random(Round(5 * Fator)) <= 3` test.
- Consider the idle jump when no projectile threatens, provided the fighter is not already airborne and no selected projectile is terminating.
- Use standing defense for a projectile above the CPU and low defense for one below it, with the original 120/100-pixel thresholds and distinct probabilities.

The fighter-frame and effect-update phases are separate; projectile distances are sampled before the effect tick moves the live effect slots, matching the original state order.

##🌈 Pascal RNG overloads

The ROT interpreter preserves the integer-width marker returned by Pascal `Round`:

- `Random(2)`, `Random(8)` and other literal integer ranges use `Random(LongInt)`, consuming one MT word.
- `Random(Round(...))` uses `Random(Int64)`, consuming two MT words.

Range zero is preserved rather than silently changing overloads or skipping the request. This matters at high difficulty/campaign combinations where a small `Round(multiplier * Fator)` can become zero.

##🌈 Fidelity boundary

Implemented in this slice:

- Common `[IA]` target-distance selection, approach and close-range decisions.
- All fifteen standard fighter `[IA]` sections, using their original source scripts.
- Nested source `AcaoJogador` action chains with shared RNG.
- CPU projectile detection and source-ordered `DistProj` / `ProjY` selection.
- CPU defense, low defense and projectile-aware jump decisions.
- Separate exact F1 difficulty factors for `CpuNovo` and `defendergolpe`.

Still separate work:

- `CHS_ADVERSARIOS` campaign opponent generation, boss selection and stage transitions. Until that exists, the CPU campaign phase remains 1.
- Human defensive input behavior inside the full `defendergolpe` path.
- The optional C16 Chapolin bonus extension, which is outside the standard fifteen-fighter roster.
- Full visual/effect and collision parity for every projectile type. The AI can reason over supported live hit-capable effects, but that does not claim every original effect has already been ported.

##🌈 Regression coverage

The test suite checks both exact difficulty formulas, source-slot projectile selection, own-projectile exclusion and execution of each standard fighter's `[IA]` section through the shared ROT host. The complete native and WASM CI matrix remains the compiler and contract gate.
