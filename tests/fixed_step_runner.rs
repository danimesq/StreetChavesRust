use std::time::Duration;

use street_chaves::fixed_timestep::TickBatch;
use street_chaves::input::{Button, Direction, InputEvent, InputFrame};
use street_chaves::runtime::{FixedStepRunner, SimulationTick};

#[derive(Default)]
struct RecordingSimulation {
    inputs: Vec<InputFrame>,
}

impl SimulationTick for RecordingSimulation {
    fn tick(&mut self, input: InputFrame) {
        self.inputs.push(input);
    }
}

#[test]
fn input_edges_wait_for_a_due_tick_and_are_sampled_once() {
    let mut runner = FixedStepRunner::new();
    let mut simulation = RecordingSimulation::default();
    let attack = Button(2);

    runner.handle_input_event(InputEvent::DirectionChanged(Direction::Right));
    runner.handle_input_event(InputEvent::ButtonChanged {
        button: attack,
        down: true,
    });

    // Host/render callbacks may happen before the first logical tick.
    assert_eq!(
        runner.advance(Duration::from_millis(10), &mut simulation).ticks_due,
        0
    );
    assert_eq!(
        runner.advance(Duration::from_millis(17), &mut simulation).ticks_due,
        0
    );
    assert!(simulation.inputs.is_empty());

    let due = runner.advance(Duration::from_millis(1), &mut simulation);
    assert_eq!(due.ticks_due, 1);
    assert_eq!(simulation.inputs.len(), 1);
    assert_eq!(simulation.inputs[0].direction, Direction::Right);
    assert!(simulation.inputs[0].was_pressed(attack));
    assert!(simulation.inputs[0].is_held(attack));
    assert_eq!(simulation.inputs[0].held_for(attack), 1);

    // The pressed edge was consumed by the first simulation tick.
    runner.advance(Duration::from_millis(28), &mut simulation);
    assert_eq!(simulation.inputs.len(), 2);
    assert!(!simulation.inputs[1].was_pressed(attack));
    assert!(simulation.inputs[1].is_held(attack));
    assert_eq!(simulation.inputs[1].held_for(attack), 2);
}

#[test]
fn catch_up_samples_input_once_per_tick_and_delivers_edges_only_once() {
    let mut runner = FixedStepRunner::new();
    let mut simulation = RecordingSimulation::default();
    let attack = Button(4);

    runner.handle_input_event(InputEvent::ButtonChanged {
        button: attack,
        down: true,
    });

    let batch = runner.advance(Duration::from_millis(56), &mut simulation);

    assert_eq!(batch.ticks_due, 2);
    assert_eq!(simulation.inputs.len(), 2);
    assert!(simulation.inputs[0].was_pressed(attack));
    assert_eq!(simulation.inputs[0].held_for(attack), 1);
    assert!(!simulation.inputs[1].was_pressed(attack));
    assert_eq!(simulation.inputs[1].held_for(attack), 2);
    assert_eq!(runner.scheduled_ticks(), 2);
}

#[test]
fn focus_reset_clears_queued_edges_before_the_next_simulation_tick() {
    let mut runner = FixedStepRunner::new();
    let mut simulation = RecordingSimulation::default();
    let attack = Button(7);

    runner.handle_input_event(InputEvent::DirectionChanged(Direction::DownLeft));
    runner.handle_input_event(InputEvent::ButtonChanged {
        button: attack,
        down: true,
    });
    runner.handle_input_event(InputEvent::Reset);

    runner.advance(Duration::from_millis(28), &mut simulation);

    assert_eq!(simulation.inputs.len(), 1);
    assert_eq!(simulation.inputs[0].direction, Direction::Neutral);
    assert!(!simulation.inputs[0].is_held(attack));
    assert!(!simulation.inputs[0].was_pressed(attack));
    assert_eq!(simulation.inputs[0].held_for(attack), 0);
}

#[test]
fn runner_executes_35_simulation_ticks_per_second_at_60_and_144_hz() {
    let mut at_60_hz = FixedStepRunner::new();
    let mut simulation_60 = RecordingSimulation::default();

    for frame in 0..60 {
        // These 60 presentation intervals total exactly one second.
        let nanos = 16_666_666 + (if frame < 40 { 1 } else { 0 });
        at_60_hz.advance(Duration::from_nanos(nanos), &mut simulation_60);
    }

    let mut at_144_hz = FixedStepRunner::new();
    let mut simulation_144 = RecordingSimulation::default();

    for frame in 0..144 {
        // These 144 presentation intervals also total exactly one second.
        let nanos = 6_944_444 + (if frame < 64 { 1 } else { 0 });
        at_144_hz.advance(Duration::from_nanos(nanos), &mut simulation_144);
    }

    assert_eq!(simulation_60.inputs.len(), 35);
    assert_eq!(simulation_144.inputs.len(), 35);
    assert_eq!(at_60_hz.scheduled_ticks(), 35);
    assert_eq!(at_144_hz.scheduled_ticks(), 35);
    assert_eq!(at_60_hz.remainder(), Duration::from_millis(20));
    assert_eq!(at_144_hz.remainder(), Duration::from_millis(20));
}

#[test]
fn renderer_cadence_can_advance_without_running_simulation() {
    let mut runner = FixedStepRunner::new();
    let mut simulation = RecordingSimulation::default();

    let batch = runner.advance(Duration::from_millis(16), &mut simulation);

    assert_eq!(
        batch,
        TickBatch {
            ticks_due: 0,
            interpolation_alpha: 16.0 / 28.0,
        }
    );
    assert!(simulation.inputs.is_empty());
}
