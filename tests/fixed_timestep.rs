use std::time::Duration;

use street_chaves::fixed_timestep::FixedTimestepClock;

#[test]
fn no_tick_is_scheduled_before_the_full_28ms_interval() {
    let mut clock = FixedTimestepClock::new();

    let almost = clock.advance(Duration::from_nanos(27_999_999));
    assert_eq!(almost.ticks_due, 0);
    assert_eq!(clock.scheduled_ticks(), 0);
    assert_eq!(clock.remainder(), Duration::from_nanos(27_999_999));

    let boundary = clock.advance(Duration::from_nanos(1));
    assert_eq!(boundary.ticks_due, 1);
    assert_eq!(boundary.interpolation_alpha, 0.0);
    assert_eq!(clock.scheduled_ticks(), 1);
    assert_eq!(clock.remainder(), Duration::ZERO);
}

#[test]
fn fractional_time_is_carried_between_render_callbacks() {
    let mut clock = FixedTimestepClock::new();

    assert_eq!(clock.advance(Duration::from_millis(10)).ticks_due, 0);
    assert_eq!(clock.advance(Duration::from_millis(10)).ticks_due, 0);

    let final_batch = clock.advance(Duration::from_millis(8));
    assert_eq!(final_batch.ticks_due, 1);
    assert_eq!(final_batch.interpolation_alpha, 0.0);
    assert_eq!(clock.scheduled_ticks(), 1);
}

#[test]
fn one_second_produces_the_same_ticks_at_60_and_144_render_callbacks() {
    let mut at_60_hz = FixedTimestepClock::new();
    for frame in 0..60 {
        // 60 intervals sum to exactly one second.
        let nanos = 16_666_666 + (if frame < 40 { 1 } else { 0 });
        at_60_hz.advance(Duration::from_nanos(nanos));
    }

    let mut at_144_hz = FixedTimestepClock::new();
    for frame in 0..144 {
        // 144 intervals also sum to exactly one second.
        let nanos = 6_944_444 + (if frame < 64 { 1 } else { 0 });
        at_144_hz.advance(Duration::from_nanos(nanos));
    }

    assert_eq!(at_60_hz.scheduled_ticks(), 35);
    assert_eq!(at_144_hz.scheduled_ticks(), 35);
    assert_eq!(at_60_hz.remainder(), Duration::from_millis(20));
    assert_eq!(at_144_hz.remainder(), Duration::from_millis(20));
    assert_eq!(
        at_60_hz.interpolation_alpha(),
        at_144_hz.interpolation_alpha()
    );
}

#[test]
fn a_stall_reports_all_due_ticks_without_silently_dropping_time() {
    let mut clock = FixedTimestepClock::new();

    let batch = clock.advance(Duration::from_millis(113));

    assert_eq!(batch.ticks_due, 4);
    assert_eq!(clock.scheduled_ticks(), 4);
    assert_eq!(clock.remainder(), Duration::from_millis(1));
}

#[test]
fn zero_elapsed_time_does_not_advance_the_simulation_clock() {
    let mut clock = FixedTimestepClock::new();

    let batch = clock.advance(Duration::ZERO);

    assert_eq!(batch.ticks_due, 0);
    assert_eq!(batch.interpolation_alpha, 0.0);
    assert_eq!(clock.scheduled_ticks(), 0);
}
