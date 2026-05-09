# Transform Table Migration Plan

## What Exists in the Original Repo

The original Solarance: Beginnings module uses five tables per moving object to track position:

```
StellarObject            — the entity (Ship, Asteroid, Station, etc.)
├── sobj_velocity        — x, y, rotation_radians, auto_dampen
├── sobj_internal_transform — private, exact position (updated every server tick)
├── sobj_hi_res_transform   — public, high-frequency position broadcast
└── sobj_low_res_transform  — public, low-frequency position broadcast
```

Additionally there is `sobj_player_window` (a bounding-box-based visibility system) and `ship_movement_controller` (a separate controller table tracking WASD key state).

### Problems with this approach

1. **Five tables per entity** means five separate DB writes per physics tick. The server must drive a game loop that constantly writes new positions.
2. **hi-res vs low-res** is a manual bandwidth-reduction heuristic that still requires continual writes. There is no concept of "the physics will coast from here; don't bother me until inputs change."
3. **sobj_velocity** stores an instantaneous velocity but no acceleration, so the client cannot extrapolate — it must wait for the next hi-res tick for an updated position. This caps smoothness at the server's write rate.
4. **sobj_player_window** is a bespoke spatial filter that adds complexity without addressing the root cause.

---

## Target Schema

Collapse all five tables into a single `movement: MovementState` field embedded directly on the entity row:

```rust
// REMOVE these tables entirely:
//   sobj_velocity
//   sobj_internal_transform
//   sobj_hi_res_transform
//   sobj_low_res_transform
//   sobj_player_window        (replaced by view functions)
//   ship_movement_controller  (replaced by InputState on the ship)

// MODIFY Ship (or whatever entity carries a StellarObject):
#[table(name = ship)]       // NOT public — served via view function instead
pub struct Ship {
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    #[unique]
    pub player_id: Identity,

    #[index(btree)]
    pub sector_id: u64,          // keep — used by view join

    pub ship_type_id: u32,
    pub faction_id: u32,
    pub health: f32,
    pub shields: f32,
    pub energy: f32,

    // Dead-reckoning snapshot — replaces all transform/velocity tables
    pub movement: MovementState,

    // Raw input state — replaces ship_movement_controller
    pub input_state: InputState,

    pub last_fired: Timestamp,
}
```

`StellarObject` itself may still be useful as a lightweight index for non-moving or rarely-moving objects (stations, asteroids, jump gates). For ships specifically it is now redundant because `Ship` directly carries `sector_id` and `movement`.

---

## What to Do With StellarObject

**Option A — Keep StellarObject as a meta-entity for non-ship objects only.**
Asteroids, stations, and jump gates don't move continuously. They only need a static position, not a full `MovementState`. Keep `StellarObject` + a simple `(x, y, rotation)` for them. Remove the hi-res/low-res/velocity tables for everything; just use static position fields.

**Option B — Remove StellarObject entirely.**
Define separate tables for each entity class (`Asteroid`, `Station`, `JumpGate`). Each carries its own position field. No shared parent ID. This is simpler to query but means per-entity view functions.

Option A is lower risk for an incremental migration. Option B is cleaner long-term.

---

## InputState Replaces ship_movement_controller

```rust
pub struct InputState {
    pub is_thrusting: bool,
    pub is_breaking: bool,   // mutually exclusive with is_thrusting
    pub turn_direction: i8,  // -1, 0, +1
}
```

The old `ship_movement_controller` stored `forward/backward/left/right` booleans as a separate table joined by player identity. Embedding `InputState` directly on the ship eliminates that join, keeps the key state close to the physics state, and lets reducers early-return when inputs haven't changed.

---

## Reducer Migration

| Old pattern | New pattern |
|---|---|
| Game loop calls `sobj_velocity` writer every tick | No game loop — server is silent until input changes |
| Client sends WASD state to `ship_movement_controller` | Client sends input edges to `set_thrust_input` / `set_turn_input` |
| Server reads velocity + internal transform, steps physics, writes hi/low-res | Reducer calls `predict_movement(ship.movement, now)` → stores snapshot |
| Client renders from hi_res_transform (last known pos) | Client calls `predict_movement(ship.movement, now())` every frame |

The key reducers needed:

```rust
set_thrust_input(ctx, is_thrusting: bool, is_breaking: bool)
set_turn_input(ctx, turn_direction: i8)
// (optional: set_forward_thrust for analog velocity override)
```

See `spacetimedb/src/reducers.rs` in the prototype for complete implementations.

---

## Migration Steps

1. **Add `solarance-shared` as a workspace dependency.** It must be reachable from both the server module and the game client.

2. **Define `MovementState` and `InputState`** as `#[derive(SpacetimeType)]` custom types. Copy them from `spacetimedb/src/physics.rs` in the prototype (or reference `solarance_shared::physics::MovementState` directly via a wrapper).

3. **Add `movement: MovementState` and `input_state: InputState` fields to `Ship`.** These are new columns — existing rows will need a default (zero-initialized is safe; set `last_update_time = 0` which the physics code treats as "uninitialized → return stored values as-is").

4. **Write the two input reducers** (`set_thrust_input`, `set_turn_input`). Each one: predict → update input + acceleration → write snapshot.

5. **Delete the five transform/velocity tables.** Remove `sobj_velocity`, `sobj_internal_transform`, `sobj_hi_res_transform`, `sobj_low_res_transform`, `sobj_player_window`. Remove `ship_movement_controller`.

6. **Replace the game loop movement tick** with nothing — or, if NPCs still need server-driven movement, keep the loop only for NPC ships and have it call `predict_movement` then mutate `angular_acceleration` / `acceleration` based on AI goals, writing a new snapshot.

7. **Update the client** to call `predict_movement(ship.movement, current_time_micros())` in the render loop rather than reading from `sobj_hi_res_transform`.

---

## NPC Ships

NPCs need server-side movement. The pattern:

- Keep the server game loop for NPCs only.
- On each NPC AI tick, compute the desired heading/speed.
- **Only write a new `MovementState` snapshot when the NPC's commanded acceleration or angular_acceleration changes** (same threshold logic as player input). Do not write every frame.
- Clients extrapolate NPC positions identically to player positions.

This means NPC bandwidth is proportional to how often they change direction, not to how many NPCs exist.
