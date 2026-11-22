# 🌈 Pascal RNG compatibility: FPC 3.2.2 is normative

## 🌈 Source-of-truth policy

For this Rust reconstruction, **the pinned Pascal project is the normative implementation**. The Rust runtime must reproduce the semantics of its interpreter and its compiler runtime first.

The C17/SDL3 implementation is a comparative source only. Its Delphi-style LCG is retained as a separately named candidate adapter, not the default RNG. If C and Pascal disagree, record the discrepancy and keep Pascal behavior until reverse engineering of the original `Chaves.exe` supplies evidence that justifies a documented change.

This is a project precedence rule. It does not claim that the Pascal port is itself the original game.

## 🌈 The actual compiler version is pinned

The pinned OpenStreetChaves README requires:

- Lazarus 4.8
- Free Pascal 3.2.2

This lets us inspect the matching RTL revision rather than relying on current FPC documentation. The reference implementation is `fpc/FPCSource` at tag `release_3_2_2`, especially `rtl/inc/system.inc`.

## 🌈 How the Pascal interpreter dispatches Random

In `ChavesRoteiro.pas`, all script random calls go through `SortearValor(N, Largo)`:

- `Largo = false`: call `System.Random(LongInt(N))`.
- `Largo = true`: call `System.Random(N)` using the Int64 overload.

The value-width bit is preserved by the expression evaluator. `Round` and `Trunc` produce Int64 values in this interpreter, so expressions such as `Random(Round(8 * Fator))` select the Int64 overload. An explicit `ShortInt(...)` cast converts back to the narrow integer path.

The existing replay recorder in `ChavesRoteiro.pas` stores each request as the tuple `(N, Largo, R)`. In replay mode it checks both the requested range and the width before returning the recorded result. A mismatch sets `SorteioDivergiu` and returns to normal RNG operation. This is an existing, valuable differential-testing mechanism.

## 🌈 FPC 3.2.2 System.Random: MTwist and two different consumption costs

The pinned RTL contains a Pascal translation of libmtwist, with a 624-word state and the 397 offset used by MT19937.

| Overload | Raw MT words per API call | Range reduction |
|---|---:|---|
| `Random(LongInt)` | 1 | Multiply-high: high 32 bits of word × range |
| `Random(Int64)` | 2 | Compose a non-negative 63-bit value, then modulo range |

The Int64 implementation does not merely widen the LongInt result. It consumes two independent 32-bit MT words:

1. The first word becomes the low 32 bits.
2. The second contributes 31 high bits.
3. The resulting non-negative 63-bit value is reduced with modulo.

This distinction matters to the global stream. One `Random(Round(...))` call consumes two MT words, while `Random(8)` consumes one.

A zero range still advances the generator:
- `Random(LongInt(0))` consumes one MT word and returns zero.
- `Random(Int64(0))` consumes two MT words and returns zero.

The Rust implementation `Fpc322Rng` exposes both overloads, counts API calls separately from raw MT words, preserves the FPC seed-reset marker behavior, and accepts an injected `RandSeed`.

## 🌈 Deterministic Pascal reference seed

The Pascal SDL harness has a deterministic logic-recording mode:

- `ChavesSDL.pas` assigns `RandSeed := 20261026`.
- It runs a scripted sequence of inputs without waiting for real time.
- It writes the logical state trace to `logica.txt` at configured intervals.

That fixed seed is a useful reproducible test seed. It is not the normal production startup seed and must not be confused with the original executable's startup state.

Normal initialization calls `Randomize` in `Comum_Inicio.inc`. The exact seed source is platform-specific. The pinned FPC Linux RTL implementation assigns `RandSeed := LongInt(Fptime(nil))`; the deterministic harness bypasses that startup variability by setting the seed explicitly. Do not infer the Windows build's exact startup value from the Linux RTL path.

## 🌈 Rust implementation status

`src/lib.rs` now contains:

- `Fpc322Rng`: normative Pascal-compatible MTwist stream for Rust.
- `random_longint`: one-word LongInt overload.
- `random_int64`: two-word Int64 overload.
- Separate counters for System.Random calls and raw MT words.
- `PascalRandomSample`: range, result, width, first raw-word index and consumption count.
- `DelphiRng`: experimental comparison adapter based on the Delphi 5 LCG evidence from the C project. It is not the default Rust source.

Tests use the reference MT19937 seed 5489 and assert:
- raw words 3499211612, 581869302 and 3890346734;
- `Random(LongInt(8)) = 6` using one word;
- `Random(Int64(1000)) = 4` using two words;
- zero-range calls still consume the width-specific number of words.

## 🌈 Inventory from the pinned Pascal project

The normalizer's first complete run found:

- **255 unique static Random sites** across `Comum.rot` and `C01.rot` through `C15.rot`.
- **465 appearances** after composing the common sections into all 15 fighter timelines.
- **244 LongInt-width** and **221 Int64-width** occurrences in those composed timelines.
- **71 static RNG-related references/calls** in Pascal `.pas` and `.inc` files, including `Randomize`, `RandSeed`, direct `Random`, and the interpreter's `Sortear` wrappers.

The 465 count intentionally includes repeated use of `Comum.rot`; the 255 count deduplicates by source path, section and source coordinates. Neither is a dynamic draw count. Branches and loops decide which sites actually execute and how many raw MT words are consumed.

See:
- [Pascal RNG call-site inventory](pascal-rng-inventory.md)
- [Symbolic ROT normalizer](rot-normalizer.md)
- [C-port shared RNG call-site comparison](rng-callsite-inventory.md)

## 🌈 Original executable verification remains separate

The original program is Delphi 5 + DelphiX. The C project uses a Delphi-style 32-bit LCG, but that is not a reason to replace the Pascal-first behavior in Rust. The optional `DelphiRng` remains available for differential experiments when REA/Ghidra reveals the original binary's RNG path.

Still open:
- Capture the original executable's initial random state and seed source.
- Determine whether it uses the same MTwist or another Delphi 5 implementation.
- Compare a controlled original draw trace with Pascal/FPC and Rust.
- Preserve all call sites across fighter AI, frames, stage, camera, effects, HUD and victory presentation.

Until that comparison exists, the Rust default follows Pascal/FPC 3.2.2.

## 🌈 Pinned references

- [OpenStreetChaves README at pinned commit](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/README.md): Lazarus 4.8 and FPC 3.2.2 build requirement.
- [Pascal interpreter](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/game/ChavesRoteiro.pas): Random width dispatch and request record/replay.
- [Deterministic harness](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/ChavesSDL.pas): fixed `RandSeed := 20261026`.
- [Pascal initialization](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/source/game/Comum_Inicio.inc): `Randomize`.
- [FPC 3.2.2 MTwist and Random overloads](https://github.com/fpc/FPCSource/blob/release_3_2_2/rtl/inc/system.inc): exact generator, word consumption, reduction.
- [FPC 3.2.2 Linux Randomize implementation](https://github.com/fpc/FPCSource/blob/release_3_2_2/rtl/linux/system.pp): Linux-specific `Fptime(nil)` seed source.
- [C fight RNG](https://github.com/danimesq/open-street-chaves/blob/e76ad2188af6842ff6618ecdb8f425bba61c9183/src/fight/fight.c): comparison-port Delphi-style LCG, not the normative Rust default.
