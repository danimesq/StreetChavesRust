//! Focused contracts for the fixed-capacity live effect arena.

use street_chaves::effect_runtime::{
    EffectArena, EffectBounds, EffectTickContext, EFFECT_SIDE_LEFT, EFFECT_SIDE_RIGHT,
    MAX_EFFECT_INSTANCES,
};
use street_chaves::pascal_replay::PascalRandomSource;

#[derive(Default)]
struct ScriptedRng {
    values: Vec<i32>,
    cursor: usize,
}

impl ScriptedRng {
    fn with_values(values: &[i32]) -> Self {
        Self {
            values: values.to_vec(),
            cursor: 0,
        }
    }
}

impl PascalRandomSource for ScriptedRng {
    type Error = String;

    fn draw_longint(&mut self, range: i32) -> Result<i32, Self::Error> {
        let value = *self
            .values
            .get(self.cursor)
            .ok_or_else(|| format!("missing scripted draw at {}", self.cursor))?;
        self.cursor += 1;
        if value < 0 || value >= range {
            return Err(format!("scripted value {value} outside [0, {range})"));
        }
        Ok(value)
    }

    fn draw_int64(&mut self, range: i64) -> Result<i64, Self::Error> {
        self.draw_longint(range as i32).map(i64::from)
    }
}

fn tick_context() -> EffectTickContext {
    EffectTickContext {
        flash_active: false,
        hit_stop_active: false,
        camera_x: 0.0,
        camera_y: 0.0,
    }
}

#[test]
fn first_free_slot_and_capacity_match_pascal_effect_array() {
    let mut arena = EffectArena::default();
    let mut rng = ScriptedRng::default();

    for slot in 0..MAX_EFFECT_INSTANCES {
        assert_eq!(
            arena
                .spawn(2, None, 0, EFFECT_SIDE_RIGHT, slot as f64, 10.0, &mut rng)
                .unwrap(),
            Some(slot)
        );
    }
    assert_eq!(arena.active_count(), MAX_EFFECT_INSTANCES);
    assert_eq!(
        arena
            .spawn(7, Some('A'), 1, EFFECT_SIDE_LEFT, 0.0, 0.0, &mut rng)
            .unwrap(),
        None
    );
    assert_eq!(rng.cursor, 0, "full arena must return before type RNG");
    assert!(arena.remove(0));
    assert_eq!(
        arena
            .spawn(2, None, 0, EFFECT_SIDE_RIGHT, 9.0, 12.0, &mut rng)
            .unwrap(),
        Some(0),
        "allocator must reuse the first free slot"
    );
}

#[test]
fn type7_uses_weak_ini_values_and_consumes_one_random_five() {
    let mut arena = EffectArena::default();
    let mut rng = ScriptedRng::with_values(&[0]);

    assert_eq!(
        arena
            .spawn(7, Some('A'), 1, EFFECT_SIDE_LEFT, 245.0, 300.0, &mut rng)
            .unwrap(),
        Some(0)
    );
    assert_eq!(rng.cursor, 1);
    let effect = arena.instance(0).unwrap();
    assert_eq!(effect.duration, 5);
    assert!(effect.can_hit);
    assert!(effect.bottom_anchor);
    assert!(effect.variant_flag);
    assert_eq!(effect.facing, EFFECT_SIDE_LEFT);
    assert_eq!(effect.horizontal_velocity, 8);
    assert_eq!(effect.damage, 12);
    assert_eq!(effect.guard_damage, 3);
    assert_eq!(
        effect.bounds,
        EffectBounds {
            left: -25,
            top: -38,
            right: 25,
            bottom: 0
        }
    );

    let mut second_rng = ScriptedRng::with_values(&[1]);
    assert_eq!(
        arena
            .spawn(
                7,
                Some('B'),
                2,
                EFFECT_SIDE_RIGHT,
                100.0,
                200.0,
                &mut second_rng
            )
            .unwrap(),
        Some(1)
    );
    assert!(!arena.instance(1).unwrap().variant_flag);
    assert_eq!(arena.instance(1).unwrap().horizontal_velocity, 11);
}

#[test]
fn types13_and15_use_their_source_hitboxes_rng_and_sprite_sequences() {
    for (effect_type, expected_bounds, first_sprite, second_sprite) in [
        (
            13,
            EffectBounds {
                left: -25,
                top: -10,
                right: 25,
                bottom: 80,
            },
            34,
            35,
        ),
        (
            15,
            EffectBounds {
                left: -25,
                top: -25,
                right: 25,
                bottom: 37,
            },
            39,
            40,
        ),
    ] {
        let mut arena = EffectArena::default();
        let mut rng = ScriptedRng::with_values(&[0]);
        assert_eq!(
            arena
                .spawn(
                    effect_type,
                    Some('A'),
                    1,
                    EFFECT_SIDE_LEFT,
                    245.0,
                    300.0,
                    &mut rng,
                )
                .unwrap(),
            Some(0)
        );
        assert_eq!(rng.cursor, 1);
        let effect = arena.instance(0).unwrap();
        assert_eq!(effect.duration, 5);
        assert!(effect.can_hit);
        assert!(effect.variant_flag);
        assert_eq!(effect.facing, EFFECT_SIDE_LEFT);
        assert_eq!(effect.horizontal_velocity, 8);
        assert_eq!(effect.damage, 12);
        assert_eq!(effect.guard_damage, 3);
        assert_eq!(effect.bounds, expected_bounds);

        let first = arena.advance(tick_context());
        assert_eq!(first.render_snapshots[0].sprite, first_sprite);
        assert_eq!(first.render_snapshots[0].x, 237.0);
        arena.advance(tick_context());
        let third = arena.advance(tick_context());
        assert_eq!(third.render_snapshots[0].sprite, second_sprite);
    }
}

#[test]
fn type2_uses_ini_sprite_boundaries_and_moves_before_snapshot() {
    let mut arena = EffectArena::default();
    let mut rng = ScriptedRng::default();
    assert_eq!(
        arena
            .spawn(2, None, 0, EFFECT_SIDE_RIGHT, 150.0, 100.0, &mut rng)
            .unwrap(),
        Some(0)
    );

    let first = arena.advance(tick_context());
    assert_eq!(first.render_snapshots[0].sprite, 4);
    assert_eq!(first.render_snapshots[0].y, 99.0);
    assert_eq!(arena.instance(0).unwrap().frame, 1);

    arena.advance(tick_context());
    arena.advance(tick_context());
    let fourth = arena.advance(tick_context());
    assert_eq!(fourth.render_snapshots[0].sprite, 5);
    assert_eq!(fourth.render_snapshots[0].y, 96.0);
}

#[test]
fn type8_arms_its_two_source_hitbox_phases_and_stage_flag() {
    let mut arena = EffectArena::default();
    let mut rng = ScriptedRng::default();
    arena
        .spawn(8, None, 1, EFFECT_SIDE_RIGHT, 220.0, 300.0, &mut rng)
        .unwrap();

    arena.advance(tick_context());
    arena.advance(tick_context());
    let at_two = arena.advance(tick_context());
    assert_eq!(at_two.render_snapshots[0].sprite, 16);
    assert_eq!(
        at_two.render_snapshots[0].bounds,
        EffectBounds {
            left: -45,
            top: -150,
            right: 45,
            bottom: 0
        }
    );

    arena.advance(tick_context());
    arena.advance(tick_context());
    let at_five = arena.advance(tick_context());
    assert_eq!(at_five.render_snapshots[0].sprite, 17);
    assert!(at_five.render_snapshots[0].stage);
    assert_eq!(
        at_five.render_snapshots[0].bounds,
        EffectBounds {
            left: -45,
            top: -220,
            right: 45,
            bottom: 0
        }
    );
}

#[test]
fn type16_follows_its_source_sprite_sequence_and_owner_direction() {
    let mut arena = EffectArena::default();
    let mut rng = ScriptedRng::default();
    arena
        .spawn(16, None, 2, EFFECT_SIDE_LEFT, 90.0, 215.0, &mut rng)
        .unwrap();

    let first = arena.advance(tick_context());
    assert_eq!(first.render_snapshots[0].sprite, 41);
    assert_eq!(first.render_snapshots[0].facing, EFFECT_SIDE_LEFT);

    arena.advance(tick_context());
    let third = arena.advance(tick_context());
    assert_eq!(third.render_snapshots[0].sprite, 42);
}

#[test]
fn freeze_suppresses_motion_and_lifetime_but_not_frame_animation() {
    let mut arena = EffectArena::default();
    let mut rng = ScriptedRng::default();
    arena
        .spawn(2, None, 0, EFFECT_SIDE_RIGHT, 10.0, 10.0, &mut rng)
        .unwrap();

    let frozen = arena.advance(EffectTickContext {
        flash_active: true,
        ..tick_context()
    });
    assert_eq!(frozen.render_snapshots[0].sprite, 4);
    assert_eq!(frozen.render_snapshots[0].y, 10.0);
    assert_eq!(arena.instance(0).unwrap().frame, 0);
}
