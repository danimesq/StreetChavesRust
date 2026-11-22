//! Contract-level fixtures. These validate data ordering and deterministic
//! sequencing; they are not original-game golden traces yet.

use street_chaves::{
    ActionId, ActionTimeline, CombatEvent, FighterId, FixtureRng, RandomSource,
    Tick, TickEvents,
};

#[test]
fn two_tick_fixture_keeps_each_ticks_source_order() {
    let mut timeline = ActionTimeline::new(FighterId(1), ActionId(3));

    let mut first = TickEvents::new(Tick(0));
    first.push(CombatEvent::FrameChanged { frame: 33 }).unwrap();
    first.push(CombatEvent::SoundQueued { sound_id: 14 }).unwrap();
    timeline.push_tick(first).unwrap();

    let mut second = TickEvents::new(Tick(1));
    second.push(CombatEvent::RandomDraw {
        upper_exclusive: 40, value: 12, draw_index: 0,
    }).unwrap();
    second.push(CombatEvent::EffectSpawned {
        effect_id: 2, variant: None,
    }).unwrap();
    timeline.push_tick(second).unwrap();

    assert_eq!(timeline.ticks().len(), 2);
    assert_eq!(timeline.ticks()[0].events()[0].event,
        CombatEvent::FrameChanged { frame: 33 });
    assert_eq!(timeline.ticks()[0].events()[1].event,
        CombatEvent::SoundQueued { sound_id: 14 });
    assert_eq!(timeline.ticks()[1].events()[0].ordinal, 0);
    assert_eq!(timeline.ticks()[1].events()[1].ordinal, 1);
}

#[test]
fn random_draw_trace_is_sensitive_to_call_order() {
    let mut rng = FixtureRng::new(vec![1, 7, 2]);
    let first = rng.draw(3).unwrap();
    let second = rng.draw(10).unwrap();
    let third = rng.draw(4).unwrap();

    assert_eq!((first.value, second.value, third.value), (1, 7, 2));
    assert_eq!((first.draw_index, second.draw_index, third.draw_index), (0, 1, 2));
    assert_eq!(rng.remaining(), 0);
}
