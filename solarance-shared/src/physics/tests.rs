use super::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

fn make_state(
    x: f32,
    y: f32,
    velocity: f32,
    rotation: f32,
    angular_velocity: f32,
    time: i64,
) -> MovementState {
    MovementState {
        pos: Vec2 { x, y },
        velocity,
        rotation,
        angular_velocity,
        last_update_time: time,
        acceleration: 0.0,
        angular_acceleration: 0.0,
        max_speed: 500.0,
        // Equivalent to 180 deg/s.
        max_turn_rate: PI,
        dampen_angular_rotation: false,
    }
}

const BASE_TIME: i64 = 1; // non-zero so we don't hit the "uninitialized" guard
const ONE_SECOND: i64 = 1_000_000; // 1 second in microseconds

#[test]
fn straight_line_no_regression() {
    // Ship at origin heading 0 (east along +x in standard trig) with velocity 100 px/s
    let state = make_state(0.0, 0.0, 100.0, 0.0, 0.0, BASE_TIME);
    let (pos, rot, ..) = predict_movement(&state, BASE_TIME + ONE_SECOND);

    assert!(
        (pos.x - 100.0).abs() < 0.01,
        "x should be ~100, got {}",
        pos.x
    );
    assert!(pos.y.abs() < 0.01, "y should be ~0, got {}", pos.y);
    assert!(
        (rot - 0.0).abs() < 0.01,
        "rotation should be ~0, got {}",
        rot
    );
}

#[test]
fn stationary_ship_rotation_only() {
    // Ship not moving but rotating at π/2 rad/s for 1 second
    let state = make_state(5.0, 10.0, 0.0, 0.0, FRAC_PI_2, BASE_TIME);
    let (pos, rot, ..) = predict_movement(&state, BASE_TIME + ONE_SECOND);

    assert!(
        (pos.x - 5.0).abs() < 0.01,
        "x should stay at 5, got {}",
        pos.x
    );
    assert!(
        (pos.y - 10.0).abs() < 0.01,
        "y should stay at 10, got {}",
        pos.y
    );
    assert!(
        (rot - FRAC_PI_2).abs() < 0.01,
        "rotation should be ~π/2, got {}",
        rot
    );
}

#[test]
fn quarter_turn_arc() {
    // Ship at origin, heading 0 (east), speed 100, turning at π/2 rad/s.
    // After 1 second it has turned to π/2 and should be at approximately
    // the analytically-computed arc position.
    let state = make_state(0.0, 0.0, 100.0, 0.0, FRAC_PI_2, BASE_TIME);
    let (pos, rot, ..) = predict_movement(&state, BASE_TIME + ONE_SECOND);

    // Analytical: r = v/ω = 100 / (π/2) ≈ 63.66
    // x = r * (sin(π/2) - sin(0)) = r * 1 ≈ 63.66
    // y = -r * (cos(π/2) - cos(0)) = -r * (-1) = r ≈ 63.66
    let r = 100.0 / FRAC_PI_2;

    assert!((pos.x - r).abs() < 0.1, "x should be ~{}, got {}", r, pos.x);
    assert!((pos.y - r).abs() < 0.1, "y should be ~{}, got {}", r, pos.y);
    assert!(
        (rot - FRAC_PI_2).abs() < 0.01,
        "rotation should be ~π/2, got {}",
        rot
    );
}

#[test]
fn full_circle_returns_near_origin() {
    // Ship at origin, heading 0, speed 100, turning at 2π rad/s.
    // After 1 second it completes a full circle and should be back near the origin.
    let state = make_state(0.0, 0.0, 100.0, 0.0, TAU, BASE_TIME);
    let (pos, rot, ..) = predict_movement(&state, BASE_TIME + ONE_SECOND);

    assert!(
        pos.x.abs() < 0.5,
        "x should be near 0 after full circle, got {}",
        pos.x
    );
    assert!(
        pos.y.abs() < 0.5,
        "y should be near 0 after full circle, got {}",
        pos.y
    );
    // Rotation wraps back to 0
    assert!(
        rot.abs() < 0.01 || (rot - TAU).abs() < 0.01,
        "rotation should be ~0 or ~2π, got {}",
        rot
    );
}

#[test]
fn negative_angular_velocity() {
    // Same as quarter turn but turning left (negative angular velocity)
    let state = make_state(0.0, 0.0, 100.0, 0.0, -FRAC_PI_2, BASE_TIME);
    let (pos, rot, ..) = predict_movement(&state, BASE_TIME + ONE_SECOND);

    let r = 100.0 / FRAC_PI_2;

    // Turning left: x stays positive, y goes negative
    assert!((pos.x - r).abs() < 0.1, "x should be ~{}, got {}", r, pos.x);
    assert!(
        (pos.y + r).abs() < 0.1,
        "y should be ~-{}, got {}",
        r,
        pos.y
    );
    let expected = TAU - FRAC_PI_2; // wraps to 3π/2
    assert!(
        (rot - expected).abs() < 0.01,
        "rotation should be ~3π/2, got {}",
        rot
    );
}

#[test]
fn no_movement_when_time_not_advanced() {
    let state = make_state(10.0, 20.0, 50.0, FRAC_PI_4, FRAC_PI_2 / 3.0, 100);
    let (pos, rot, ..) = predict_movement(&state, 100); // same time
    assert!((pos.x - 10.0).abs() < 0.001);
    assert!((pos.y - 20.0).abs() < 0.001);
    assert!((rot - FRAC_PI_4).abs() < 0.001);

    let (pos2, rot2, ..) = predict_movement(&state, 50); // earlier time
    assert!((pos2.x - 10.0).abs() < 0.001);
    assert!((pos2.y - 20.0).abs() < 0.001);
    assert!((rot2 - FRAC_PI_4).abs() < 0.001);
}

#[test]
fn dampening_stops_rotation_within_step() {
    // max_turn_rate = π → decel_rate = π/2 rad/s²
    // angular_velocity = π/4 rad/s → t_stop = (π/4)/(π/2) = 0.5 s
    // rotation gained = 0.5 * (π/4) * 0.5 = π/16
    // After 1 second the ship should be at π/16 and holding.
    let mut state = make_state(0.0, 0.0, 0.0, 0.0, FRAC_PI_4, BASE_TIME);
    state.dampen_angular_rotation = true;

    let (_, rot, ..) = predict_movement(&state, BASE_TIME + ONE_SECOND);
    let expected = PI / 16.0;
    assert!(
        (rot - expected).abs() < 0.01,
        "rotation should be ~π/16 after dampening stop, got {}",
        rot
    );
}

#[test]
fn dampening_partial_deceleration_within_step() {
    // max_turn_rate = π → decel_rate = π/2 rad/s²
    // angular_velocity = π rad/s → t_stop = π/(π/2) = 2 s (longer than 1 s step)
    // rotation gained = π*1 - 0.5*(π/2)*1² = π - π/4 = 3π/4
    let mut state = make_state(0.0, 0.0, 0.0, 0.0, PI, BASE_TIME);
    state.dampen_angular_rotation = true;

    let (_, rot, ..) = predict_movement(&state, BASE_TIME + ONE_SECOND);
    let expected = 3.0 * PI / 4.0;
    assert!(
        (rot - expected).abs() < 0.01,
        "rotation should be ~3π/4 after partial dampening, got {}",
        rot
    );
}

#[test]
fn dampening_negative_angular_velocity_clamps_at_zero() {
    // Same as dampening_stops_rotation_within_step but turning left.
    // angular_velocity = -π/4 rad/s → stops at -π/16 → wraps to 2π - π/16
    let mut state = make_state(0.0, 0.0, 0.0, 0.0, -FRAC_PI_4, BASE_TIME);
    state.dampen_angular_rotation = true;

    let (_, rot, ..) = predict_movement(&state, BASE_TIME + ONE_SECOND);
    let expected = TAU - PI / 16.0;
    assert!(
        (rot - expected).abs() < 0.01,
        "rotation should be ~2π - π/16 after negative dampening stop, got {}",
        rot
    );
}

#[test]
fn dampening_off_does_not_affect_rotation() {
    // With dampening off, angular_velocity of π/4 rad/s for 1 s → π/4
    let state = make_state(0.0, 0.0, 0.0, 0.0, FRAC_PI_4, BASE_TIME);
    let (_, rot, ..) = predict_movement(&state, BASE_TIME + ONE_SECOND);
    assert!(
        (rot - FRAC_PI_4).abs() < 0.01,
        "rotation without dampening should be ~π/4, got {}",
        rot
    );
}

#[test]
fn dampening_arc_position_stops_curving() {
    // Moving ship with angular_velocity that will stop mid-step.
    // After angular_velocity reaches zero the ship should continue straight.
    // max_turn_rate = π → decel_rate = π/2 rad/s²
    // angular_velocity = π/4 rad/s → t_stop = 0.5 s
    // At t_stop, rotation = π/16. Ship then travels straight at π/16 for the remaining 0.5 s.
    let mut state = make_state(0.0, 0.0, 100.0, 0.0, FRAC_PI_4, BASE_TIME);
    state.dampen_angular_rotation = true;

    let (pos, rot, ..) = predict_movement(&state, BASE_TIME + ONE_SECOND);

    // After 1 second the ship must have stopped spinning
    let expected = PI / 16.0;
    assert!(
        (rot - expected).abs() < 0.5,
        "rotation should be ~π/16 after dampening arc, got {}",
        rot
    );

    // Position must be non-zero (ship was moving)
    let moved = (pos.x * pos.x + pos.y * pos.y).sqrt();
    assert!(
        moved > 10.0,
        "ship should have moved significantly, got distance {}",
        moved
    );
}
