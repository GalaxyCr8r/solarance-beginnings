# Dead-Reckoning Movement System

## The Core Idea

Instead of sending a position every frame, the server stores a **physics snapshot** whenever a ship's inputs change. Both server and every client run the same deterministic `predict_movement` function to extrapolate where the ship is *right now* from that snapshot. The server never needs a game loop tick for movement at all.

```
Client presses thrust
       │
       ▼
reducer set_thrust_input() called
  1. Call predict_movement() to snapshot current pos/rot/velocity
  2. Attach new acceleration to the snapshot
  3. Write updated MovementState to DB
       │
       ▼
SpacetimeDB replicates the new row to every subscribed client
       │
       ▼
Each client calls predict_movement(snapshot, now) every render frame
to get a smooth interpolated position — no further network traffic needed
until the next input change
```

---

## MovementState Struct

Everything needed to extrapolate a ship's position is in one struct. This lives directly as a field on entity tables (e.g., embedded in `SpaceShip.movement`).

```rust
pub struct MovementState {
    /// World-space position at last_update_time.
    pub pos: Vec2,
    /// Forward speed (px/s). Always >= 0 — ships cannot fly backward.
    pub velocity: f32,
    /// Heading in degrees. 0° = east (+x axis). Increases clockwise (Y is down).
    pub rotation: f32,
    /// Rate of heading change (deg/s). Positive = clockwise.
    pub angular_velocity: f32,
    /// When this snapshot was recorded, microseconds since Unix epoch.
    pub last_update_time: i64,
    /// Constant linear acceleration from last_update_time onward.
    /// Positive = thrusting, negative = braking. px/s²
    pub acceleration: f32,
    /// Constant angular acceleration from last_update_time onward. deg/s²
    /// Set to ±max_angular_acceleration while turning, 0 when released.
    pub angular_acceleration: f32,
    /// Hard cap on forward speed. px/s
    pub max_speed: f32,
    /// Hard cap on angular speed. deg/s
    pub max_turn_rate: f32,
    /// When true and angular_acceleration == 0, omega bleeds toward zero
    /// at max_turn_rate/2 deg/s². Simulates rotational friction.
    pub dampen_angular_rotation: bool,
}
```

**Key insight:** `acceleration` and `angular_acceleration` are constants stored at the time of the snapshot. The client doesn't need any more data — it applies these constants forward in time using the physics integrator.

---

## predict_movement

```rust
pub fn predict_movement(state: &MovementState, current_time: i64) -> (Vec2, f32, f32, f32)
// Returns: (position, heading_degrees, speed_px_per_s, angular_velocity_deg_per_s)
```

Located in `solarance-shared/src/physics/mod.rs`. Both server reducers and client rendering call this identically.

The simulation is **event-driven rather than fixed-step**:
- It finds the next moment when `v` or `ω` would hit a clamping boundary.
- It advances only to that boundary, then continues.
- This is O(1) regardless of how large `dt` is (a 60-second gap costs the same as a 1-second gap).

Sub-cases handled:
| Situation | Method |
|---|---|
| `ω = 0`, `a = 0` | Straight line, constant speed — trivial |
| `ω = 0`, `a ≠ 0` | Kinematic `d = v·t + ½·a·t²` |
| `ω ≠ 0`, `a = 0` | Closed-form circular arc: `Δx = r·(sin θ₁ − sin θ₀)` |
| `ω ≠ 0`, `a ≠ 0` | 20-step Euler; always short because `dt` is bounded by the next event |
| `ω` changing (dampening or angular accel) | 20-step Euler with trapezoidal rule for `Δθ` |

---

## How Reducers Use It

The pattern is the same in every input reducer: **predict → update inputs → write snapshot**.

```rust
#[reducer]
pub fn set_thrust_input(ctx: &ReducerContext, is_thrusting: bool, is_breaking: bool)
    -> Result<(), String>
{
    let mut ship = /* fetch ship */;

    // Early return if nothing changed (avoids unnecessary DB write)
    if /* inputs unchanged */ { return Ok(()); }

    let now = ctx.timestamp.to_micros_since_unix_epoch();

    // 1. Predict where the ship is RIGHT NOW under current acceleration
    let (pos, rot, vel, ang_vel) = predict_movement(&ship.movement, now);

    // 2. Determine new acceleration from the new input
    let new_acceleration = if is_thrusting { config.max_acceleration }
                           else if is_breaking { -config.max_acceleration }
                           else { 0.0 };

    // 3. Store snapshot with updated acceleration
    ship.movement = MovementState {
        pos, velocity: vel, rotation: rot, angular_velocity: ang_vel,
        acceleration: new_acceleration,
        last_update_time: now,
        ..ship.movement
    };

    dsl.update_space_ship_by_id(ship);
    Ok(())
}
```

The same pattern applies to `set_turn_input` (changes `angular_acceleration`). The two reducers are independent because linear and angular motion are decoupled.

---

## Input State

`InputState` is stored alongside `MovementState` on the ship. It captures the raw key state so the server can detect no-ops and early-return without doing any physics work:

```rust
pub struct InputState {
    pub is_thrusting: bool,
    pub is_breaking: bool,   // mutually exclusive with is_thrusting
    pub turn_direction: i8,  // -1 (left), 0 (none), +1 (right)
}
```

---

## Bullets

Bullets use the same `MovementState` field. When a weapon fires:

1. `predict_movement` is called to find the ship's exact position at the moment of firing (the client sends the timestamp it fired at, not the current server time — handling for clock skew).
2. A `Bullet` row is created with `velocity = 250.0`, `acceleration = 0.0`, `angular_velocity = 0.0`, rotation = ship's current heading.
3. Hit detection in `submit_hit` also calls `predict_movement` on both the bullet and the target ship at the claimed hit timestamp, then checks the squared distance against a threshold.

This means bullet movement and hit detection are fully deterministic and require zero server-side tick work.

---

## Coordinate System

- **X** increases right, **Y** increases down (screen space).
- **Rotation** 0° = east (+x). Increases clockwise.
- **Forward direction vector**: `(cos θ_rad, sin θ_rad)`.
- `rotation_to_vector` in the shared lib uses the north-up (0° = north) convention for rendering — note the different convention from the simulation internals.

---

## Bandwidth Characteristics

| Old approach | New approach |
|---|---|
| Position written every game tick for every moving entity | Position written only on input change |
| hi-res table: ~10–20 writes/s per ship | Typical: 1–5 writes/s per ship (key press / release) |
| All clients in sector receive all position updates | View function filters to current-sector ships only |
| Server must run a game loop | Server has no movement game loop — pure event-driven |

---

## solarance-shared Crate

The physics lives in `solarance-shared/src/physics/mod.rs`. This crate is intentionally minimal — no game engine deps, no WASM feature flags — so it can be compiled into:
- The SpacetimeDB module (server-side hit detection)
- The Bevy/egui client (client-side interpolation)

When porting to the main repo, this crate should be added as a workspace member and referenced by both the server module and the game client.

`Cargo.toml` dependency: `solarance-shared = { path = "../solarance-shared" }`
