# Fixed 28 ms simulation clock

## Status

The platform-neutral Rust core contains both:

- `FixedTimestepClock`: an integer-nanosecond accumulator for the researched 28 ms logical interval.
- `FixedStepRunner`: a headless execution bridge that connects that clock to the shared `InputSampler` and a platform-neutral simulation trait.

There is not yet an SDL3 host loop, browser host loop, concrete game-wide `Game::tick` implementation, or renderer. The shared execution boundary is ready for those adapters to implement without changing timing or input semantics.

## Logical interval

The researched game loop uses a 28 ms logical interval. Rendering cadence is independent: the host may present at 60 Hz, 144 Hz, a variable refresh rate, or another cadence without changing the number of simulation updates.

`FixedTimestepClock` accumulates elapsed time as integer nanoseconds. A call to `advance(elapsed)` returns:

- `ticks_due`: every complete 28 ms simulation interval represented by the elapsed time;
- `interpolation_alpha`: the remaining fractional interval, in the range [0, 1), for optional presentation interpolation.

The clock also tracks the total number of ticks scheduled since construction. It has no SDL, browser, WASM, operating-system clock, sleep, input, or renderer dependency.

## Shared headless execution interface

Native SDL3, standalone browser Canvas and any other host use the same `FixedStepRunner`:

- `handle_input_event(InputEvent)` accepts normalized logical events. The host owns translation from physical keys, scancodes, controller events or browser events.
- `advance(elapsed, simulation)` asks the fixed clock how many ticks are due, samples the input sampler once per due tick, and invokes the simulation once for each sample.
- `SimulationTick` is the interface implemented by the authoritative game-state/orchestrator. Each call represents exactly one logical simulation update.

The runner does not own or invent a game state. A concrete simulation implementation is intentionally a separate milestone; tests use a recording headless simulation to verify the boundary.

A host follows this order:

1. Measure elapsed time from its monotonic clock.
2. Drain platform events and normalize each event into a logical `InputEvent`.
3. Pass those events to `FixedStepRunner::handle_input_event`.
4. Call `FixedStepRunner::advance(elapsed, &mut simulation)`.
5. Render the latest read-only snapshot once at the host's presentation cadence, using the returned `interpolation_alpha` only for visual interpolation if snapshots eventually support it.

The integration shape is:

~~~rust
let mut runner = FixedStepRunner::new();

// For each drained platform event, map it to a logical InputEvent.
runner.handle_input_event(InputEvent::ButtonChanged {
    button: Button(0),
    down: true,
});

let batch = runner.advance(elapsed, &mut simulation);

// Presentation happens independently, including when batch.ticks_due is zero.
renderer.render(simulation.snapshot(), batch.interpolation_alpha);
~~~

Here `simulation` implements `SimulationTick`; `snapshot` and `renderer` describe the later read-only presentation boundary and are not claimed to exist in the current crate.

## Input sampling semantics

Input events must be applied before `advance`. If no 28 ms tick is due, pending pressed/released edges remain queued in `InputSampler`; render callbacks do not consume them.

When multiple ticks are due in one callback, `FixedStepRunner` samples once for every tick:

- The first due tick receives pending `pressed` and `released` edges.
- Later catch-up ticks do not repeat those edges.
- Held buttons remain held and their counters advance once per tick.
- A focus-loss/device-reset event clears held state and pending edges before the next sample.

This preserves the input sampler's logical-tick semantics under both high refresh rates and catch-up updates.

## Catch-up and pause policy

The clock does not impose a maximum catch-up count and does not discard complete intervals. For example, 113 ms of elapsed time schedules four ticks and retains 1 ms. The runner executes all four due simulation calls before returning to the host.

A host must not feed time spent in an intentional pause or a suspended/background browser tab into the simulation clock. On pause/resume or page-visibility transitions, the adapter should rebase its own previous-timestamp measurement so that suspended wall time is not mistaken for active simulation time. Any future catch-up cap must be an explicit, documented host policy; it must not silently alter deterministic replay or input sampling.

## Rendering and interpolation

Rendering may happen once per host callback, even when no simulation tick is due. The interpolation fraction is derived only from the sub-tick remainder. It may be used to interpolate presentation between snapshots, but it must never mutate authoritative simulation state, fighter timers, collision geometry, RNG, or input hold counters.

Neither `FixedTimestepClock` nor `FixedStepRunner` renders, sleeps, reads a host clock, or accesses platform APIs. The host controls presentation cadence; the runner controls logical simulation cadence.

## Validation

The existing clock tests cover:

- no update before the exact 28 ms boundary;
- carrying fractional time across host callbacks;
- identical one-second tick counts at 60 Hz and 144 Hz presentation rates;
- reporting all due ticks after a delayed callback without dropping elapsed time;
- zero elapsed time.

The runner integration tests cover:

- pending input edges surviving callbacks where no tick is due;
- exactly one input sample per due simulation tick;
- one-shot pressed edges and per-tick held counters during catch-up;
- focus reset before the next simulation sample;
- exactly 35 simulation calls per second at both 60 Hz and 144 Hz;
- presentation callbacks with no due simulation tick.

The 60 Hz and 144 Hz tests each feed exactly one second of elapsed time and must execute exactly 35 logical ticks, leaving exactly 20 ms in the accumulator.
