//! # Dead-Reckoning Physics
//!
//! This module predicts where a ship is *right now* given a snapshot of its
//! state (position, heading, speeds, accelerations) that was recorded at some
//! point in the past.  Both the server and every client run the same code so
//! that a ship's rendered position stays consistent without sending an update
//! every frame.
//!
//! ## Coordinate system
//! * **X** increases to the right, **Y** increases downward (screen space).
//! * **theta** (rotation/heading) is in **radians**.  0 points along the
//!   positive-X axis (right); angles increase clockwise on screen because Y is
//!   inverted.
//! * A ship's forward velocity vector is therefore `(cos θ, sin θ)`.
//!
//! ## Key variable names used throughout
//! | Name | Meaning | Units |
//! |------|---------|-------|
//! | `v` | forward speed (scalar, always ≥ 0) | px/s |
//! | `a` | linear acceleration (positive = thrust, negative = brake) | px/s² |
//! | `theta` / `θ` | heading angle | radians |
//! | `omega` / `ω` | angular velocity — rate of heading change | rad/s |
//! | `alpha` / `α` | angular acceleration — rate of ω change | rad/s² |
//! | `r` | radius of curvature of the ship's arc = v / ω | px |
//!
//! ## How the simulation works
//! Rather than blindly stepping through all of `dt` with tiny fixed steps,
//! `simulate` identifies the next "event" — the moment `v` or `ω` hits a
//! clamping boundary — and advances only to that boundary before moving on.
//! This gives O(1) complexity regardless of how large `dt` is (a client that
//! held a key for 60 s still runs in the same time as one that held it for
//! 1 s) and keeps accuracy high because each numerical sub-phase is short.

use std::hash::Hasher;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// World-space 2D vector. Shared between server tables (via `SpacetimeType`)
/// and client renderers — the single canonical position/heading type.
#[cfg_attr(feature = "server", derive(spacetimedb::SpacetimeType))]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    pub fn distance_to(&self, other: &Vec2) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Squared distance — cheaper than `distance_to` when you only need to
    /// compare distances (avoids the sqrt).
    pub fn distance_to_sq(&self, other: &Vec2) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }

    pub fn sub(&self, other: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn signed_angle_to(&self, other: &Vec2) -> f32 {
        let cross = self.x * other.y - self.y * other.x;
        let dot = self.x * other.x + self.y * other.y;
        cross.atan2(dot)
    }

    pub fn to_glam(&self) -> glam::Vec2 {
        glam::Vec2 { x: self.x, y: self.y }
    }

    pub fn from_glam(vec: glam::Vec2) -> Vec2 {
        Vec2 { x: vec.x, y: vec.y }
    }
}

impl PartialEq for Vec2 {
    fn eq(&self, other: &Self) -> bool {
        // Compare the bit patterns of the floats.
        // This means 0.0 and -0.0 are different, and NaN == NaN.
        self.x.to_bits() == other.x.to_bits() && self.y.to_bits() == other.y.to_bits()
    }
}

impl Eq for Vec2 {}

impl std::hash::Hash for Vec2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl From<glam::Vec2> for Vec2 {
    fn from(glam_vec: glam::Vec2) -> Self {
        Vec2 { x: glam_vec.x, y: glam_vec.y }
    }
}

impl From<Vec2> for glam::Vec2 {
    fn from(vec: Vec2) -> Self {
        glam::Vec2 { x: vec.x, y: vec.y }
    }
}

/// A complete snapshot of an entity's motion at a single point in time.
/// Stored in the database by the server whenever inputs change; clients
/// extrapolate forward from this snapshot using `predict_movement`.
///
/// Angular damping is **always on** inside `predict_movement` — when
/// `angular_acceleration == 0` and `angular_velocity != 0`, ω bleeds toward
/// zero at `max_turn_rate / 2` rad/s². To opt out (spin-forever objects),
/// set `max_turn_rate = 0` (decel_rate becomes 0 and no event fires).
#[cfg_attr(feature = "server", derive(spacetimedb::SpacetimeType))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MovementState {
    /// World-space position at `last_update_time`.
    pub pos: Vec2,
    /// Forward speed at `last_update_time`. Always ≥ 0 — ships cannot fly
    /// backward. Units: pixels per second.
    pub velocity: f32,
    /// Heading at `last_update_time`. See module-level coordinate notes.
    /// Units: radians.
    pub rotation: f32,
    /// Rate of heading change at `last_update_time`. Positive = clockwise
    /// on screen (because Y is down). Units: radians per second.
    pub angular_velocity: f32,
    /// When this snapshot was recorded, as microseconds since the Unix epoch.
    /// Used to compute `dt = current_time − last_update_time`.
    pub last_update_time: i64,
    /// Constant linear acceleration applied from `last_update_time` onward.
    /// Positive = thrusting forward, negative = braking. Units: px/s².
    pub acceleration: f32,
    /// Constant angular acceleration applied from `last_update_time` onward.
    /// Set to ±base_angular_acceleration while a turn key is held, or 0 when
    /// released (the always-on dampening then bleeds ω to zero).
    /// Units: radians/s².
    pub angular_acceleration: f32,
    /// Hard cap on forward speed. `v` is never allowed to exceed this.
    /// Units: pixels per second.
    pub max_speed: f32,
    /// Hard cap on angular speed. `|ω|` is never allowed to exceed this.
    /// Also doubles as the dampening rate (decel_rate = max_turn_rate / 2).
    /// Set to 0 to opt out of angular damping (spin-forever). Units: rad/s.
    pub max_turn_rate: f32,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Extrapolates a ship's state from `state.last_update_time` to `current_time`.
///
/// Returns `(position, heading_radians, speed_px_per_s, angular_velocity_rad_per_s)`.
///
/// All four values are produced by a single unified simulation pass, so they
/// are always mutually consistent (no risk of position and velocity drifting
/// apart because they were computed separately).
///
/// If `current_time` is at or before the snapshot time (clock skew, same
/// frame) the stored values are returned unchanged.
pub fn predict_movement(state: &MovementState, current_time: i64) -> (Vec2, f32, f32, f32) {
    if state.last_update_time == 0 || current_time <= state.last_update_time {
        return (
            state.pos,
            state.rotation,
            state.velocity,
            state.angular_velocity,
        );
    }

    // Convert microsecond timestamps to a floating-point dt in seconds.
    let dt = (current_time - state.last_update_time) as f32 / 1_000_000.0;
    simulate(state, dt)
}

// ---------------------------------------------------------------------------
// Core simulation
// ---------------------------------------------------------------------------

/// Advances the physics state forward by `total_dt` seconds.
///
/// The loop divides `total_dt` into phases, each ending at the next "event"
/// — the moment `v` or `ω` would hit a clamping boundary.  Within each
/// phase, the physics is either solved analytically (constant ω) or
/// numerically (ω changing), but the numerical sub-steps are always short
/// because they're bounded by the event time, not by `total_dt`.
fn simulate(state: &MovementState, total_dt: f32) -> (Vec2, f32, f32, f32) {
    // Unpack constants for this prediction run.
    let max_v = state.max_speed;
    let max_omega = state.max_turn_rate;
    // Rotational-friction deceleration rate (half of max turn rate). When
    // max_turn_rate == 0, decel_rate is 0 — spin-forever opt-out.
    let decel_rate = state.max_turn_rate / 2.0;
    let a = state.acceleration;       // linear acceleration (px/s²)
    let alpha = state.angular_acceleration; // angular acceleration (rad/s²)

    // Working state — mutated each phase.
    let mut x = state.pos.x;
    let mut y = state.pos.y;
    let mut theta = state.rotation;          // heading in radians
    let mut v = state.velocity.max(0.0);     // clamp out any stale negative velocity
    let mut omega = state.angular_velocity;  // angular velocity in rad/s

    let mut remaining = total_dt;

    while remaining > f32::EPSILON {
        // Find how long until each velocity hits its clamping boundary.
        let t_omega = omega_event_time(omega, alpha, max_omega, decel_rate);
        let t_v = v_event_time(v, a, max_v);

        // Advance only to the nearest boundary (or the end of remaining time).
        let phase_dt = remaining.min(t_omega).min(t_v);

        // phase_dt can be tiny when v is nearly zero and a is negative —
        // avoid an infinite loop of vanishingly small phases.
        if phase_dt < f32::EPSILON {
            break;
        }

        if t_omega.is_finite() {
            // ω is changing this phase (accelerating or dampening toward zero).
            // Use numerical integration — but because phase_dt ≤ t_omega, the
            // phase is short and accuracy is high regardless of total_dt.
            numerical_phase(
                &mut x, &mut y, &mut theta, &mut v, &mut omega,
                a, alpha, max_v, max_omega, decel_rate, phase_dt,
            );
        } else {
            // ω is constant this phase — use closed-form analytical formulas.
            constant_omega_phase(&mut x, &mut y, &mut theta, &mut v, omega, a, max_v, phase_dt);
        }

        // Snap velocities exactly at boundary to prevent floating-point drift
        // accumulating across many phases.
        if t_omega.is_finite() && phase_dt >= t_omega - 1e-6 {
            omega = if alpha.abs() < f32::EPSILON {
                // dampening finished: ω has bled all the way to zero
                0.0
            } else if alpha > 0.0 {
                max_omega   // positive acceleration hit the cap
            } else {
                -max_omega  // negative acceleration hit the cap
            };
        }
        if t_v.is_finite() && phase_dt >= t_v - 1e-6 {
            v = if a > 0.0 { max_v } else { 0.0 };
        }

        remaining -= phase_dt;
    }

    // Normalise heading to [0, 2π) so callers never see e.g. 7 rad or −0.5 rad.
    theta %= std::f32::consts::TAU;
    if theta < 0.0 {
        theta += std::f32::consts::TAU;
    }

    (Vec2 { x, y }, theta, v, omega)
}

// ---------------------------------------------------------------------------
// Event-time helpers
// ---------------------------------------------------------------------------

/// Returns the number of seconds until `ω` reaches its clamping boundary.
///
/// When `α ≠ 0` the ship is actively turning. ω changes linearly as
/// `ω(t) = ω₀ + α·t`, so the time to reach the cap is simply:
///     t = (target − ω₀) / α
///
/// When `α == 0` and `decel_rate > 0`, ω is bleeding toward zero at a
/// constant rate (the always-on rotational friction), so:
///     t = |ω| / decel_rate
///
/// Returns `INFINITY` when ω is already at its boundary, or when `decel_rate`
/// is 0 (spin-forever opt-out via `max_turn_rate = 0`).
fn omega_event_time(omega: f32, alpha: f32, max_omega: f32, decel_rate: f32) -> f32 {
    if alpha.abs() > f32::EPSILON {
        // Active angular acceleration toward ±max_omega.
        let target = if alpha > 0.0 { max_omega } else { -max_omega };
        let t = (target - omega) / alpha;
        // A non-positive t means ω is already past (or at) the boundary.
        if t > 0.0 { t } else { f32::INFINITY }
    } else if decel_rate > f32::EPSILON && omega.abs() > f32::EPSILON {
        // Rotational friction: ω decays linearly to zero.
        omega.abs() / decel_rate
    } else {
        f32::INFINITY // ω is constant — no angular event
    }
}

/// Returns the number of seconds until `v` reaches its clamping boundary.
///
/// `v` changes linearly as `v(t) = v₀ + a·t`, so:
/// * Thrusting (a > 0): time to reach `max_v` is `(max_v − v) / a`
/// * Braking  (a < 0): time to reach `0` is `−v / a`  (both numerator and
///   denominator are negative, so the result is positive)
///
/// Returns `INFINITY` when `a == 0`, or when `v` is already at its boundary
/// (signal to the caller: no linear event in this direction).
fn v_event_time(v: f32, a: f32, max_v: f32) -> f32 {
    if a > f32::EPSILON {
        let t = (max_v - v) / a;
        if t > 0.0 { t } else { f32::INFINITY }
    } else if a < -f32::EPSILON {
        // t = -v / a.  With v ≥ 0 and a < 0, this is positive as long as v > 0.
        // If v is already 0, t = 0 which is not > 0, so INFINITY is returned —
        // meaning "no future event; we're already at the floor."
        let t = -v / a;
        if t > 0.0 { t } else { f32::INFINITY }
    } else {
        f32::INFINITY // a == 0 — v is constant
    }
}

// ---------------------------------------------------------------------------
// Phase integrators
// ---------------------------------------------------------------------------

/// Advances position, heading, and speed for one phase in which `ω` is
/// constant.  Chooses among three sub-cases:
///
/// 1. **Straight line, constant speed** (`ω ≈ 0`, `a ≈ 0`):
///    Simple `Δpos = v · dt · heading_vector`.
///
/// 2. **Straight line, accelerating/braking** (`ω ≈ 0`, `a ≠ 0`):
///    Kinematic formula `d = v·t + ½·a·t²`.  `dt` is bounded by `t_v` (the
///    event time), so `v` cannot overshoot `[0, max_v]`.  A stopped ship with
///    negative `a` is a no-op — it cannot be pushed backward.
///
/// 3. **Pure circular arc** (`ω ≠ 0`, `a ≈ 0`):
///    The ship moves along an arc of radius `r = v / ω_rad`.  Integrating the
///    velocity vector `(cos θ, sin θ)` over the arc from θ₀ to θ₁ gives the
///    closed-form displacement:
///        Δx =  r · (sin θ₁ − sin θ₀)
///        Δy = −r · (cos θ₁ − cos θ₀)
///    (The minus on Δy arises from the screen-Y-down coordinate system.)
///
/// 4. **Arc with changing speed** (`ω ≠ 0`, `a ≠ 0`):
///    No clean closed form — falls back to 20-step Euler integration.  `dt`
///    is bounded by `t_v` so the sub-steps are short and the error is small.
fn constant_omega_phase(
    x: &mut f32,
    y: &mut f32,
    theta: &mut f32,
    v: &mut f32,
    omega: f32,
    a: f32,
    max_v: f32,
    dt: f32,
) {
    if omega.abs() < f32::EPSILON {
        // ── Straight-line motion ────────────────────────────────────────────
        if a.abs() < f32::EPSILON {
            // Constant speed: Δpos = v · dt · (cos θ, sin θ)
            *x += theta.cos() * *v * dt;
            *y += theta.sin() * *v * dt;
        } else if *v <= 0.0 && a < 0.0 {
            // Ship is already stopped and braking: nothing to do.
            // Without this guard, d = ½·a·dt² would be negative and push the
            // ship backward through the world.
        } else if *v >= max_v && a > 0.0 {
            // Already at the speed cap and still thrusting. Without this
            // guard the kinematic formula below would happily push v past
            // max_v: `v_event_time` returns INFINITY when v == max_v
            // (because t = 0 isn't > 0), so phase_dt becomes the full
            // remaining time instead of being bounded by t_v, and
            // `v += a·dt` accumulates indefinitely. Coast at max_v.
            *x += theta.cos() * max_v * dt;
            *y += theta.sin() * max_v * dt;
            *v = max_v;
        } else {
            // Kinematic formula: d = v₀·t + ½·a·t²
            // dt is bounded by t_v so v stays within [0, max_v]; clamp on
            // both sides as defense in depth against rounding.
            let d = *v * dt + 0.5 * a * dt * dt;
            *x += theta.cos() * d;
            *y += theta.sin() * d;
            *v = (*v + a * dt).clamp(0.0, max_v);
        }
    } else if a.abs() < f32::EPSILON {
        // ── Pure circular arc (analytical) ─────────────────────────────────
        // Radius of curvature: r = v / ω.
        // Integrating the heading vector over the arc yields:
        //   Δx =  r · (sin θ₁ − sin θ₀)
        //   Δy = −r · (cos θ₁ − cos θ₀)
        let theta0 = *theta;
        let theta1 = theta0 + omega * dt;
        let r = *v / omega;
        *x += r * (theta1.sin() - theta0.sin());
        *y -= r * (theta1.cos() - theta0.cos());
        *theta += omega * dt; // heading advances at constant ω
    } else {
        // ── Arc with changing speed — numerical fallback ────────────────────
        // dt is bounded by t_v, so speed only changes up to the next clamping
        // event; 20 sub-steps is more than sufficient for this short window.
        const STEPS: usize = 20;
        let step_dt = dt / STEPS as f32;
        for _ in 0..STEPS {
            *x += theta.cos() * *v * step_dt;
            *y += theta.sin() * *v * step_dt;
            *theta += omega * step_dt;
            *v = (*v + a * step_dt).clamp(0.0, max_v);
        }
    }
}

/// Advances position, heading, speed, and angular velocity for one phase in
/// which `ω` is actively changing (either under angular acceleration or
/// rotational dampening).
///
/// `dt` is bounded by `t_omega` (the time to the next angular clamping event)
/// so it is always short.  Within that short window, 20 sub-steps of Euler
/// integration are accurate enough.
///
/// **Rotation uses the trapezoidal rule** (`Δθ = (ω_old + ω_new) / 2 · dt`)
/// rather than plain Euler (`Δθ = ω · dt`).  When ω changes linearly — which
/// is exact for both constant angular acceleration and constant dampening rate
/// — the trapezoidal rule is also exact (no truncation error).
fn numerical_phase(
    x: &mut f32,
    y: &mut f32,
    theta: &mut f32,
    v: &mut f32,
    omega: &mut f32,
    a: f32,
    alpha: f32,
    max_v: f32,
    max_omega: f32,
    decel_rate: f32,
    dt: f32,
) {
    const STEPS: usize = 20;
    let step_dt = dt / STEPS as f32;

    for _ in 0..STEPS {
        let omega_old = *omega;

        // ── Update ω ───────────────────────────────────────────────────────
        if alpha.abs() < f32::EPSILON {
            // Rotational friction (always-on when alpha == 0 and decel_rate > 0).
            // Cap the decrement so we don't overshoot zero in a single step.
            let d_omega = -omega.signum() * decel_rate * step_dt;
            if d_omega.abs() >= omega.abs() {
                *omega = 0.0;
            } else {
                *omega += d_omega;
            }
        } else {
            // Active turn: ω = ω + α·dt, clamped to ±max_omega.
            *omega = (*omega + alpha * step_dt).clamp(-max_omega, max_omega);
        }

        // ── Update position (forward Euler, using heading at start of step) ─
        *x += theta.cos() * *v * step_dt;
        *y += theta.sin() * *v * step_dt;

        // ── Update heading (trapezoidal rule) ──────────────────────────────
        // Average of ω before and after the ω update eliminates the first-
        // order error that plain Euler would accumulate.
        *theta += (omega_old + *omega) / 2.0 * step_dt;

        // ── Update speed ───────────────────────────────────────────────────
        *v = (*v + a * step_dt).clamp(0.0, max_v);
    }
}

#[cfg(test)]
mod tests;
