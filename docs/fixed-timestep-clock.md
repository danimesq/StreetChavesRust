# Fixed 28 ms simulation clock

## Status

Implemented in the platform-neutral Rust core as a scheduling primitive. There is not yet an SDL3 host loop, browser host loop, or complete Game::tick orchestrator. This module establishes the timing contract those hosts will share.

## Logical interval

The researched game loop uses a 28 ms logical interval. Rendering cadence is independent: the host may present at 60 Hz, 144 Hz, a variable refresh rate, or another cadence without changing the number of simulation updates.

FixedTimestepClock accumulates elapsed time as integer nanoseconds. A call to advance(elapsed) returns:

- ticks_due: every complete 28 ms simulation interval represented by the elapsed time;
- interpolation_alpha: the remaining fractional interval, in the range [0, 1), for optional presentation interpolation.

The clock also tracks the total number of ticks scheduled since construction. It has no SDL, browser, WASM, operating-system clock, sleep, input, or renderer dependency.

## Host-loop order

The SDL3 and browser adapters should follow the same order:

1. Read elapsed time from the adapter's monotonic clock.
2. Drain platform events and normalize them into logical input events.
3. Apply those events to the shared InputSampler.
4. Call FixedTimestepClock::advance(elapsed).
5. For each returned tick, call InputSampler::sample_tick() exactly once and advance the simulation exactly once.
6. Obtain a read-only presentation snapshot and render once at the host's available presentation cadence.

The intended integration shape is:

~~~rust
let batch = clock.advance(elapsed);

for _ in 0..batch.ticks_due {
    let input = input_sampler.sample_tick();
    simulation.tick(input);
}

renderer.render(simulation.snapshot(), batch.interpolation_alpha);
~~~

Here simulation.tick, simulation.snapshot, and renderer.render describe the future host/core boundary; they are not claimed to exist in the current crate yet.

Applying the collected input events before the tick loop means pending pressed and released edges are delivered on the first due logical tick. Held state remains visible on subsequent catch-up ticks, and hold counters advance once per tick. OS key-repeat and render FPS do not advance those counters.

## Catch-up and pause policy

The clock does not impose a maximum catch-up count and does not discard complete intervals. For example, 113 ms of elapsed time schedules four ticks and retains 1 ms. The host must execute the returned ticks before rendering the newest state.

A host must not feed time spent in an intentional pause or a suspended/background browser tab into the simulation clock. On pause/resume or page-visibility transitions, the adapter should rebase its own previous-timestamp measurement so that suspended wall time is not mistaken for active simulation time. Any future catch-up cap must be an explicit, documented host policy; it must not silently alter deterministic replay or input sampling.

## Rendering and interpolation

Rendering may happen once per host callback, even when no simulation tick is due. The interpolation fraction is derived only from the sub-tick remainder. It may be used to interpolate presentation between snapshots, but it must never mutate authoritative simulation state, fighter timers, collision geometry, RNG, or input hold counters.

The clock schedules simulation work; it does not itself run the simulation or render.

## Validation

The integration tests cover:

- no update before the exact 28 ms boundary;
- carrying fractional time across host callbacks;
- identical one-second tick counts at 60 Hz and 144 Hz presentation rates;
- reporting all due ticks after a delayed callback without dropping elapsed time;
- zero elapsed time.

The 60 Hz and 144 Hz tests each feed exactly one second of elapsed time and must schedule exactly 35 logical ticks, leaving exactly 20 ms in the accumulator.
