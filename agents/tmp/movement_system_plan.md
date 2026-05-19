# Movement-System Rewrite — Implementation Plan

Replaces the 20 Hz `delete-all + recreate-all` `sobj_hi_res_transform` broadcast with a dead-reckoning snapshot stored directly on moving entities. Sequenced so the codebase compiles at every checkpoint; old tables stay live until after the new path is wired and proven.

**Branch:** `refactor-2-movement-and-views`
**Shared crate:** `solarance-shared/` (already prototyped; needs minor cleanup)
**Target:** `task server:publish-clear` succeeds, two-machine maincloud test shows smooth movement.

**Checkpoint semantics:** "compiles cleanly," not "runs correctly." Between Phase 4 (server stops writing old transform tables) and Phase 8 (client stops reading them), the system is in a dual-reality state — old client subscriptions point at stale data. There are no production users to disrupt, so this is purely a dev-machine concern: don't expect smooth solo-run testing in that window. The first runtime checkpoint is end of Phase 8 (solo run smooth); the final one is Phase 10 (two-machine maincloud).

---

## Phase 1 — `solarance-shared` housekeeping

Make the shared crate consumable from server + client. Establish a single canonical `Vec2` and `MovementState` on the server side.

- Bump `solarance-shared/Cargo.toml`: `spacetimedb = "2.1.0"` → `"2.2.0"`.
- **Move `tables::common_types::Vec2` into `solarance-shared`** (whole module: struct, `SpacetimeType` derive, `PartialEq`/`Eq`/`Hash`, glam interop, math helpers). Delete `tables::common_types::Vec2` and update server imports (`JumpGate.target_gate_arrival_pos`, `ShipTypeDefinition::get_world_corners_at_position`, etc.) to point at `solarance_shared::Vec2`. The placeholder `Vec2 { x, y }` in `solarance-shared/src/physics/mod.rs` is *promoted* to this shared type — not replaced by `glam::Vec2`. Internal compute in `predict_movement` may convert to `glam::Vec2` privately at the function boundary if it wants glam's math API.
- Confirm `#[derive(SpacetimeType)]` on `MovementState` (and any nested types that become table columns).
- **Drop `dampen_angular_rotation: bool` from `MovementState`.** Angular damping is always-on inside `predict_movement` when `angular_acceleration == 0` and `angular_velocity != 0`. Spin-forever objects (none today) can opt out via `max_turn_rate = 0` (which makes `decel_rate = max_turn_rate / 2 = 0`). Saves a byte per snapshot, removes a configuration footgun, simplifies the snapshot helpers (no per-write decision about whether to dampen).
- Confirm radians-only public surface; `rotation_to_vector` is already deleted.
- Add `solarance-shared` as a path dep in `server/Cargo.toml` and `client/Cargo.toml`.

**Client-side gotcha (resurfaces in Phase 8):** `spacetime generate` will produce a parallel `server::bindings::Vec2` and `server::bindings::MovementState` on the client with `#[derive(__lib::ser::Serialize, __lib::de::Deserialize, Clone, PartialEq, Debug)]` instead of `SpacetimeType`. These are *different Rust types* from `solarance_shared::Vec2` / `solarance_shared::MovementState` even though they are structurally identical. This is by design and unavoidable — there is no way to map generated bindings to a pre-existing Rust type. The client must convert at the boundary before calling `predict_movement`. Phase 8 adds the `From` impls.

**Checkpoint:** `cargo check` clean on shared, server, client.

---

## Phase 2 — Additive schema (no deletions yet)

New fields exist in parallel with the old transform tables.

- `ShipTypeDefinition`: drop `base_turn_rate: f32`, add `base_angular_acceleration: f32` (rad/s²), add `base_max_turn_rate: f32` (rad/s — cap on angular velocity). Two free parameters now where there was one — the snapshot helper re-stamps `max_turn_rate` from `base_max_turn_rate`. Hand-tune both per ship type in `definitions/ship_type_definitions.rs`; don't mechanically translate from old `base_turn_rate` (different units, different physics quantity, the old values were due for retuning anyway).
- `Ship`: add `movement: MovementState`.
- `CargoCrate`: add `movement: MovementState`.
- `Asteroid`: add `position: Vec2` (rotation derived client-side from `id ⊕ time`).
- `Station`: add `position: Vec2, rotation: f32`.
- `JumpGate`: add `position: Vec2, rotation: f32`, and `target_gate_arrival_rotation: f32` (paired with the existing `target_gate_arrival_pos: Vec2`). Designer-tunable; usually points the arriving ship into the sector, away from the gate.
- All position fields use `solarance_shared::Vec2` (the single canonical type from Phase 1).
- Update `definitions/ship_type_definitions.rs` for `base_angular_acceleration`.

**Checkpoint:** `task server:publish-clear` succeeds.

---

## Phase 3 — Snapshot helpers

Two helpers, called from every site that touches movement.

New file: `server/src/logic/stellarobjects/movement.rs`

- `get_ship_movement_snapshot(dsl, ship_id) -> MovementState` — predict to `ctx.timestamp`, no write. Used by range checks (dock distance, mining range, jumpgate proximity).
- `write_ship_movement_snapshot(dsl, ship_id, |state| { ... })` — predict, mutate via closure, **re-stamp caps from `ShipTypeDefinition`: `max_speed ← base_speed`, `max_turn_rate ← base_max_turn_rate`**, write. Used by movement input, dock/undock, jumpgate transit, jettison.
- `transit_ship_to_sector(dsl, ship_id, destination_sector_id, arrival_pos, arrival_rotation) -> Result<(), String>` — atomically updates `Ship.sector_id`, `ShipStatus.sector_id`, `StellarObject.sector_id`, then calls `write_ship_movement_snapshot` with a clean-stop closure (`pos = arrival_pos`, `rotation = arrival_rotation`, velocity/ω/accelerations zeroed). Called by `try_to_use_jumpgate`. Distinct from `write_ship_movement_snapshot` because cross-sector transit has different invariants than same-sector position changes — don't fuse them.
- Same snapshot pair for `CargoCrate` — caps re-stamped from `global_config` (or a `cargo_crate_constants` row, whichever's cleaner) on every write.

**Caps contract:** caps live on the snapshot (denormalised) so `predict_movement` is self-contained and the client never needs to look up `ShipTypeDefinition` to render. Hot edits to `ShipTypeDefinition` take effect on the next input change. The mutation closure must not touch caps — re-stamping is the helper's job, not the caller's. This denormalisation also future-proofs against the case where not all clients can see every `ShipTypeDefinition` (e.g. faction-scoped ship types).

Both delegate to `solarance_shared::physics::predict_movement`. Two functions per entity beats a trait — keeps the call sites obvious in `spacetime logs` greps.

---

## Phase 4 — Movement input reducer

In `server/src/logic/ships/movement_controllers.rs`:

- **Drop `stellar_object_id` from `ShipMovementController`** — it was only used by the (now-gone) timer dispatch. The reducer looks up the ship via `dsl.get_ships_by_player_id(player_id).next()` instead.
- Apply mutual-exclusion precedence: `forward && backward → 0`, `left && right → 0`.
- **No-op early-return:** read the existing controller row, compare against incoming flags, return `Ok(())` if unchanged. Keypress repeats must not generate snapshot writes.
- Update `ShipMovementController` flags.
- Compute desired linear/angular acceleration from `(f, b, l, r)` × `ship_type.base_acceleration` / `base_angular_acceleration`.
- `write_ship_movement_snapshot(dsl, ship_id, |s| { s.linear_acceleration = ...; s.angular_acceleration = ...; })`.
- Delete the existing manual damping logic (lives in `predict_movement` now).

The reducer name (`update_ship_movement_controller`) stays — keeps the client API stable. The table survives with a narrowed role: it's the input-state mirror (matches the prototype handoff's `InputState` pattern), feeding the no-op check.

---

## Phase 5 — Lifecycle event resets

Every entry/exit point routes through `write_*_movement_snapshot`.

| Site | What writes |
|---|---|
| `logic/ships/creation.rs` (player ship spawn) | spawn pos, zero velocity/ω |
| `dock_to_station` | zero everything, `last_update_time = now` |
| `undock_from_station` | pos = station position, rotation = station-facing, zero dynamics |
| `try_to_use_jumpgate` | call `transit_ship_to_sector(...)` — updates all sector_ids + clean-stop snapshot in one helper |
| Cargo jettison from ship | toss speed in ship's heading direction (no momentum inheritance for MVP) |
| Cargo jettison from asteroid (no-cargo mining) | toss speed in a random unit-vector direction, spawned at asteroid position |

### Cargo crate snapshot fields at jettison

All read from `global_config` (added in Phase 7). Same `MovementState` shape as ships — uses the same `predict_movement` function:

- `pos` = jettison origin (ship's current pos for ship-jettison; asteroid position for asteroid-jettison)
- `velocity` = `cargo_crate_toss_speed` ± `cargo_crate_toss_speed_variance` (small random range)
- `rotation` = toss direction (ship's heading or random unit vector)
- `acceleration` = `-(cargo_crate_brake_rate ± cargo_crate_brake_rate_variance)` (small random range)
- `angular_velocity` = small random spin (visual flavor, optional)
- `angular_acceleration` = 0
- `max_speed` = the chosen toss speed (irrelevant in practice; velocity only decreases)
- `max_turn_rate` = a sane default from config (also enables the always-on angular damping in `predict_movement`)
- `last_update_time` = `ctx.timestamp`

**Brake-clamp at v=0:** `predict_movement` already correctly clamps `v` to 0 when `a < 0` reaches the boundary (see `v_event_time` and the simulation loop's snap-at-boundary logic). The stored `acceleration` stays at the negative brake rate after the crate has stopped — harmless as long as all reads go through the snapshot helpers. No fix-up rewrite needed.

---

## Phase 6 — Position-mutation audit

Confirm no caller bypasses the helpers, and confirm every site that should use them actually does. Four sweeps across all of `server/src` (not just `logic/`):

```bash
# Sweep 1 — transform/velocity tables
rg -n "sobj_internal_transform|sobj_velocity|sobj_hi_res_transform|sobj_low_res_transform" server/src

# Sweep 2 — position-mutation primitives
rg -n "set_x|set_y|set_rotation_radians|from_vec2|from_xy" server/src

# Sweep 3 — player windows (going away in Phase 9)
rg -n "sobj_player_window|create_sobj_player_window" server/src

# Sweep 4 — helper-coverage check (every site that should use them, does)
rg -n "get_ship_movement_snapshot|write_ship_movement_snapshot|get_cargo_crate_movement_snapshot|write_cargo_crate_movement_snapshot|transit_ship_to_sector" server/src
```

### Acceptance criteria per sweep

- **Sweep 1:** by end of Phase 6, only `tables/stellarobjects.rs` (the table definitions themselves) matches. Those rows get deleted in Phase 9. Any other hit is a stale caller that needs migration.
- **Sweep 2:** every hit is either inside the snapshot helpers (allowed), inside seed-data writing the new `position: Vec2` fields (allowed), or stale.
- **Sweep 3:** every hit goes away. `create_sobj_player_window_for` and its undock-flow caller are deleted; the timer is deleted in Phase 9.
- **Sweep 4 (the deep-modules check):** confirms no caller reimplements predict-then-write inline. If a caller has its own predict + mutate + write sequence instead of going through the helper, the helper isn't earning its keep — fold it in.

### Known callers to migrate

- `StellarObject::distance_squared` (`tables/stellarobjects.rs`) — calls `dsl.get_sobj_internal_transform_by_id`. Delete the method; no callers should remain post-migration. If any do, they're using the wrong primitive — they should use the snapshot helpers.
- `try_to_dock_to_station` distance check → `get_ship_movement_snapshot`.
- `try_mining_asteroid_reducer` range check → `get_ship_movement_snapshot` for the ship, asteroid `position` direct.
- Jumpgate proximity check in `try_to_use_jumpgate` → `get_ship_movement_snapshot` for the ship, jumpgate `position` direct.
- `recalculate_player_windows` and any other timer/reducer in `lifecycle/` that reads transforms — delete (covered by Phase 9).
- Seed-data scripts in `definitions/` that hard-code station/jumpgate positions via `sobj_internal_transform` — switch to writing the new `position: Vec2` fields directly. (Originally Open Question 2.)
- Anything in `server/src/admin/` that touches positions — likely debug reducers; migrate or delete.
- Anything in `server/src/logic/players/` that reads ship position → snapshot helpers.
- `auto_dampen` setters (`rg -n "auto_dampen" server/src`) — the only current callers are for cargo crates, and that role is now handled by the brake-rate `acceleration` field on `MovementState`. Confirm no setters survive.

---

## Phase 7 — Cargo crate despawn sweeper + jettison constants

Crates eventually go away without per-crate timers. Add the constants the jettison reducer (Phase 5) reads from.

**Added to `global_config`:**

- `cargo_crate_ttl_secs: u64` (default `14400` = 4 hr) — used by sweeper below.
- `cargo_crate_toss_speed: f32` — base toss velocity at jettison.
- `cargo_crate_toss_speed_variance: f32` — symmetric random range applied to the toss speed.
- `cargo_crate_brake_rate: f32` — magnitude of the negative acceleration applied to braking crates.
- `cargo_crate_brake_rate_variance: f32` — symmetric random range applied to the brake rate.
- `cargo_crate_max_turn_rate: f32` — cap on `MovementState.max_turn_rate` for crates.

**Sweeper:**

- New scheduled table `cargo_crate_despawn_sweeper_timer`, 30-min interval, registered in `lifecycle/timers.rs`.
- Reducer iterates all cargo crates, deletes any where `now - created_at > ttl`.

---

## Phase 8 — Client render path

- `task client:generate`.
- **Add `From` impls** in `client/src/server/impls.rs` (where all bindings-type impls already live) for the bindings → shared boundary (per Phase 1 gotcha):
  - `impl From<server::bindings::Vec2> for solarance_shared::Vec2`
  - `impl From<server::bindings::MovementState> for solarance_shared::MovementState`
  - Mechanical field-by-field copy. The only place the bindings/shared duplication is visible.
- `client/src/gameplay/render.rs` + per-class draw paths:
  - Ship: read `ship.movement` (bindings type) → convert to shared → call `predict_movement(now_micros)`.
  - CargoCrate: same.
  - Asteroid: read `position`, derive spin angle from `asteroid_id ⊕ time` deterministically.
  - Station / JumpGate: read `position, rotation` direct.
- `client/src/stdb/connector/subscriptions.rs`: drop `sobj_velocity`, `sobj_hi_res_transform`, `sobj_low_res_transform`, `sobj_player_window`.

**Checkpoint:** `cargo check` clean on client; solo run boots and a ship moves smoothly with no rotation-arrest bug.

---

## Phase 9 — Delete the old machinery

Demolition. The new path is now load-bearing.

- `server/src/tables/stellarobjects.rs`: delete `StellarObjectVelocity`, `StellarObjectTransformInternal`, `StellarObjectTransformHiRes`, `StellarObjectTransformLowRes`, `StellarObjectPlayerWindow` and their `referenced_by` entries.
- `server/src/logic/stellarobjects/transforms.rs`: delete `recalculate_sobj_transforms` reducer + timer.
- `server/src/logic/stellarobjects/player_windows.rs`: delete the timer + helpers (audit callers — `create_sobj_player_window_for` is invoked from undock flow; that whole call goes away too).
- `server/src/logic/ships/movement_controllers.rs`: delete `timer_update_all_ship_movement_controllers`, the `UpdateShipMovementControllers` scheduled table, `_update_ship_movement_controller`, and `try_update_ship_velocity`. The dead-reckoning model is event-driven — no tick required, damping lives in `predict_movement`. The `update_ship_movement_controller` input reducer is the only remaining writer.
- `server/src/lifecycle/timers.rs`: drop all three timer creations (transforms, player windows, movement controllers).

---

## Phase 10 — Verification (Definition of Done)

- `cargo check` server + client (no new warnings).
- `task server:publish-clear`.
- `task client:run` solo — confirm rotation arrest is gone and ship coasts naturally.
- **DoD:** two-machine maincloud test, eyeball smoothness for 5 min. Any visible jitter is logged in the clock-skew GH issue, not a blocker.

---

## Open implementation questions (decide as we hit them, not blocking)

1. **Seed-data scripts** that hard-code station/jumpgate positions via `sobj_internal_transform` — need to set new fields directly. Surfaces in Phase 6 audit.

---

## Out of scope (tracked separately)

See `docs/tmp/movement_system_follow-ups.md` for the four GitHub issues queued as follow-ups:

1. View functions for per-sector visibility scoping
2. Client-side prediction + reconciliation for the player's own ship
3. Minimum-latency `server_offset` clock-skew estimator + debug-widget readout
4. Generalize `combat/visual_effects.rs` for mining-asteroid effects
