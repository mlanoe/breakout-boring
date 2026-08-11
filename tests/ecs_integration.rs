//! Headless proof that the ported collision/score logic works end to end —
//! without a window. `MinimalPlugins` (no render/audio/input), same
//! technique as `bevy-boring/tests/simulation.rs`. This can't exercise
//! `move_paddle`/`update_scoreboard`/`setup`/`play_collision_sound` (they
//! need keyboard input, UI text, asset loading, or audio respectively) —
//! it targets `apply_velocity` and `check_for_collisions`, the two systems
//! that carry the actual game mechanics, both written in native Boring.

use bevy::prelude::*;
use breakout_boring::{apply_velocity, check_for_collisions, Ball, Brick, Collider, Score, Velocity};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(Score { count: 0 })
        .add_systems(Update, (apply_velocity, check_for_collisions).chain());
    app
}

#[test]
fn apply_velocity_moves_transform_by_velocity_times_dt() {
    let mut app = test_app();
    // A lone ball with no collider nearby: apply_velocity should just move
    // it; check_for_collisions should find nothing to react to.
    app.world_mut().spawn((
        Ball {},
        Transform::from_xyz(0.0, 0.0, 0.0),
        Velocity { x: 100.0, y: 0.0 },
    ));
    app.world_mut().spawn((
        Transform::from_xyz(10_000.0, 10_000.0, 0.0),
        Collider {},
        Brick {},
    ));

    // MinimalPlugins has no real-time clock ticking on its own between
    // manual `update()` calls in a test — advance `Time` explicitly so
    // `delta_secs()` inside `apply_velocity` is non-zero and deterministic.
    app.update();
    let mut time = app.world_mut().resource_mut::<Time>();
    time.advance_by(std::time::Duration::from_millis(16));
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&Transform, &Velocity)>();
    let (transform, velocity) = query.single(world).unwrap();
    assert_eq!(velocity.x, 100.0);
    assert!(
        transform.translation.x > 0.0,
        "expected the ball to have moved in +x, got {}",
        transform.translation.x
    );
}

#[test]
fn collision_with_a_brick_despawns_it_and_increments_score() {
    let mut app = test_app();
    app.world_mut().spawn((
        Ball {},
        Transform::from_xyz(0.0, 0.0, 0.0),
        Velocity { x: 0.0, y: 0.0 },
    ));
    let brick = app
        .world_mut()
        .spawn((
            Brick {},
            Collider {},
            // Overlapping the ball (radius 15.0 hardcoded in
            // check_for_collisions) so a collision is detected immediately.
            Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(20.0, 20.0, 1.0),
                ..Default::default()
            },
        ))
        .id();

    app.update();
    let mut time = app.world_mut().resource_mut::<Time>();
    time.advance_by(std::time::Duration::from_millis(16));
    app.update();

    assert!(
        app.world().get_entity(brick).is_err(),
        "expected the brick to have despawned on collision"
    );
    let score = app.world().resource::<Score>();
    assert_eq!(score.count, 1, "expected the score to have incremented once");
}

#[test]
fn collision_reflects_ball_velocity() {
    let mut app = test_app();
    app.world_mut().spawn((
        Ball {},
        Transform::from_xyz(0.0, 0.0, 0.0),
        Velocity { x: 5.0, y: -5.0 },
    ));
    // A brick directly below the ball: the closest-point offset is
    // dominated by y, hit from above -> Collision::Top -> reflect_y only.
    app.world_mut().spawn((
        Brick {},
        Collider {},
        Transform {
            translation: Vec3::new(0.0, -20.0, 0.0),
            scale: Vec3::new(100.0, 20.0, 1.0),
            ..Default::default()
        },
    ));

    app.update();
    let mut time = app.world_mut().resource_mut::<Time>();
    time.advance_by(std::time::Duration::from_millis(16));
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&Velocity>();
    let velocity = query.single(world).unwrap();
    assert_eq!(velocity.x, 5.0, "x should be unaffected by a Top collision");
    assert_eq!(velocity.y, 5.0, "y should have reflected (was moving down, hit from above)");
}
