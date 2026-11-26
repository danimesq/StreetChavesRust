# Character-specific combat parity (PR #33)

## Scope

This PR implements the C17-specific per-character body-hit overrides previously recorded as gaps in the PR #30 audit. It does not replace the Pascal/ROT reference for the original game. C17 is treated as an independent differential implementation.

Reference: `danimesq/open-street-chaves/src/fight/combat.c`, commit `a417c0621fca8438fe81ab46a39b3865dd95b49a`, function `melee_step`.

## Runtime state

- `FighterState.hitstun` stores C17's body-hit slide distance, defaulting to 4 on every successful hit.
- `FighterState.rotation_step` stores C17's signed `rot_step` in degrees per tick.
- Existing `recoil` remains the Pascal reaction/recoil field.
- Existing `angular_velocity` remains separate from C17's `rot_step`.
- Existing `vertical_velocity` receives only the specific launch writes listed below.

## Implemented override matrix

| Fighter | Action | Result |
|---|---|---|
| C01 | Special 2, strong | Hitstun 20 |
| C02 | Special 2, strong | Hitstun 13 |
| C04 | Special 2 / Special 3 | Combo window 46 / 47 |
| C04 | Final special | Hitstun 20, spin ±20, launch -30 |
| C05 | Final special, frame >= 15 | Spin ±20, launch -30 |
| C06 | Final special | Combo window 50 |
| C06 | Final special, frame >= 22 | Spin ±20 |
| C06 | Special 3, strong | Hitstun 25 |
| C07 | Special 3, strong | Hitstun 18 |
| C09 | Final special, frame >= 14 | Spin ±20 |
| C09 | Final special, frame >= 15 | Hitstun 25 |
| C10 | Final special, frame >= 15 | Spin ±20 |
| C10 | Special 4, strong | Hitstun 13 |
| C11 | Final special, frame >= 20 | Spin ±20 |
| C12 | Special 4, frame >= 22 | Spin ±20 |
| C12 | Special 2, strong | Hitstun 25 |
| C15 | Special 3 | Combo window 50, launch -26 |

Spin sign follows the defender's facing after the common hit resolver updates it: facing 1 writes -20, facing 2 writes +20.

## Validation added

- Exact combo timer overrides for C04, C06 and C15.
- All eight non-default hitstun assignments plus the ordinary default of 4.
- Every spinning character's frame threshold, including one tick before each threshold.
- Both signs of rotation.
- Each of the three vertical launch assignments and the C05 pre-threshold control.

## Boundaries

- The C17 table does not imply new Pascal `[Acerto]` sections.
- It does not equate `hitstun`, `rot_step`, `recoil`, or `angular_velocity`.
- This PR records combat state on contact. A renderer/physics host is still responsible for consuming slide distance, rotational steps and vertical velocity over subsequent ticks.
