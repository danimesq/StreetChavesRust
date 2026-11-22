# 🌈 Delphi 5 RNG compatibility

## 🌈 The 32-bit transition

The C reference implements the Delphi 5 integer random transition in src/fight/fight.c:

    next_seed = (seed * 0x08088405 + 1) modulo 2^32
    result = high_32_bits(next_seed * range)

This is a 32-bit state recurrence followed by a multiply-high range reduction. The implementation uses an unsigned 32-bit state and a 64-bit product, which preserves the low 32-bit state transition and extracts the upper 32 bits of the product.

The Delphi 5 assembly form reproduced in the historical Borland discussion uses IMUL against RandSeed, increments the result, stores RandSeed, then uses MUL with the requested range. The seed update occurs before the multiply. A zero range therefore still advances the state and returns zero.

The range API is intended for non-negative integer limits and returns values in [0, range). The Rust implementation uses u32 for that range. Negative ranges are deliberately outside the new API's contract.

## 🌈 Rust implementation

DelphiRng now provides:

- A caller-supplied 32-bit RandSeed.
- Wrapping multiplication and addition, explicitly modulo 2^32.
- Multiply-high range reduction.
- A sequential draw counter.
- A RandomSource implementation, allowing the same trace interface as FixtureRng.
- State advancement even when the requested range is zero.

Tests pin a known sequence from seed zero:

| Draw | Range | New RandSeed | Result |
|---:|---:|---:|---:|
| 0 | 10 | 0x00000001 | 0 |
| 1 | 100 | 0x08088406 | 3 |
| 2 | 2 | 0xDC6DAC1F | 1 |
| 3 | 120 | 0x33DC589C | 24 |

These vectors validate the implementation of the recovered transition and range transform. They do not establish the original executable's initial seed or the full per-frame order of random calls.

## 🌈 The seed is a separate compatibility problem

The recurrence can be exact while the full sequence still diverges because the initial seed differs.

- Pascal port: Comum_Inicio.inc calls Randomize once during game initialization.
- C port: osc_game_init seeds fight.rng from SDL_GetTicksNS, truncated to 32 bits.
- C deterministic harness: --frames overrides that with 0x5EED1234.
- Original Delphi build: the exact RandSeed produced by its Randomize call at startup has not been captured.

The game does not reset the C RNG in osc_fight_start; the state persists across fights. Pascal's RandSeed is process-global and Randomize is called from TJogo.Iniciar, not once per fighter or once per round.

For differential fixtures, inject the same known seed into both the original capture harness and Rust. Do not compare runs that merely started at similar wall-clock times.

## 🌈 Important: the Pascal port's System.Random is not the Delphi 5 oracle

The OpenStreetChaves repository is a Free Pascal/Lazarus port, not a Delphi 5 build. ChavesRoteiro.pas routes script Random calls through SortearValor, which ultimately calls System.Random.

The current Free Pascal RTL documentation describes its System.Random implementation as Mersenne Twister. That is different from the Delphi 5 32-bit LCG described above. The Pascal port also dispatches wide-valued arguments to its Int64 System.Random overload when the interpreter marks the argument as wide. The pinned OpenStreetChaves revision does not record the exact FPC compiler/RTL build used to produce a particular binary, so the precise MT implementation/version is not yet pinned.

Therefore:

- Pascal .rot files remain valuable for recovering branch order, call sites, and game-state effects.
- The current FPC RTL documentation says its System.Random uses Mersenne Twister, but the reference repository does not pin the exact compiler/RTL build. Treat this as the port's documented RTL behavior, and pin the build version before making a binary-level claim.
- A random sequence observed while running the Free Pascal port is not evidence of the original Delphi 5 sequence.
- C's osc_random transition is the stronger current implementation reference for the Delphi 5 integer generator.
- The original executable and the exact RandSeed/call stream remain the final authority.

Do not retrofit the Delphi LCG into the Pascal source and then call that a recovered original behavior. Keep the original port's runtime semantics separate from the Rust compatibility implementation.

## 🌈 Shared stream and call-order sensitivity

The fight engine uses one RandSeed field, and calls to osc_random occur in more than CPU decision logic. They also appear in frame selection, special throws, camera shake, hit-stop, effects, stage events, and presentation-related choices.

The Pascal application has a process-global RandSeed and random calls outside the fighter scripts as well: stage selection, camera shake, background objects, bonus effects, HUD jitter, round transitions, and victory presentation.

A Rust implementation that uses a separate RNG for cosmetic effects can change later gameplay outcomes if the original shared stream was consumed by those operations. Keep one ordered stream until a captured original trace proves a split is behaviorally safe.

The next RNG research artifact should list every Random call in actual fight-loop execution order, including conditional calls and loops, not merely list source locations. A loop that tries again until a frame differs consumes a variable number of draws.

## 🌈 What is confirmed and what remains open

- [x] Delphi 5 integer LCG multiplier and increment captured from the cited assembly form.
- [x] 32-bit wrapping transition implemented in Rust.
- [x] Multiply-high range transform implemented in Rust.
- [x] Zero-range call consumes one state transition in Rust.
- [x] C implementation uses the same transition and range transform for positive int ranges.
- [x] Pascal port's wrapper calls Free Pascal System.Random rather than implementing the Delphi 5 transition.
- [ ] Capture the original executable's startup RandSeed or a reproducible way to set it.
- [ ] Verify the range edge cases against the original executable.
- [ ] Expand every per-fighter and common-script RNG call into exact tick order.
- [ ] Include camera, stage, effect, UI, and fight-flow calls in the same stream trace.
- [ ] Compare the C port's consumption order with an original-executable capture.
- [ ] Add original-executable golden RNG traces after the playable Rust baseline.

## 🌈 References

- [Delphi 5 integer RNG assembly discussed in the Borland language newsgroup](https://groups.google.com/g/borland.public.delphi.language.objectpascal/c/_3BBDQ0X5H0): includes the 0x08088405 multiplier, increment, RandSeed store, and multiply-high implementation.
- [Embarcadero System.Random documentation](https://docwiki.embarcadero.com/Libraries/Athens/en/System.Random): documents the non-negative range behavior and warns that implementation details may vary by compiler version.
- [Free Pascal System.Random documentation](https://www.freepascal.org/docs-html/rtl/system/random.html): documents the FPC Mersenne Twister implementation.
- [Free Pascal Randomize documentation](https://www.freepascal.org/docs-html/rtl/system/randomize.html): states that FPC seeds its generator from the system clock.
- [C fight.c at the pinned revision](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/fight/fight.c)
- [C game.c at the pinned revision](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/game.c)
- [Pascal ChavesRoteiro.pas at the pinned revision](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/game/ChavesRoteiro.pas)
- [Pascal Comum_Inicio.inc at the pinned revision](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/game/Comum_Inicio.inc)
