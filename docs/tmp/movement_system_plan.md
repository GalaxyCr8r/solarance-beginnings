# Movement-System Rewrite — Implementation Plan

Replaces the 20 Hz `delete-all + recreate-all` `sobj_hi_res_transform` broadcast with a dead-reckoning snapshot stored directly on moving entities. Sequenced so the codebase compiles at every checkpoint; old tables stay live until after the new path is wired and proven.

**Branch:** `refactor-2-movement-and-views`
**Shared crate:** `solarance-shared/` (already prototyped; needs minor cleanup)
**Target:** `task server:publish-clear` succeeds, two-machine maincloud test shows smooth movement.

---

## Phase 1 — `solarance-shared` housekeeping

Make the shared crate consumable from server + client.

- Bump `solarance-shared/Cargo.toml`: `spacetimedb = "2.1.0"` → `"2.2.0"`.
- Swap placeholder `Vec2 { x, y }` → `glam::Vec2` throughout `solarance-shared/src/physics/mod.rs`.
- Confirm `#[derive(SpacetimeType)]` on `MovementState` (and any nested types that become table columns).
- Confirm radians-only public surface; `rotation_to_vector` is already deleted.
- Add `solarance-shared` as a path dep in `server/Cargo.toml` and `client/Cargo.toml`.

**Checkpoint:** `cargo check` clean on shared, server, client.

---

## Phase 2 — Additive schema (no deletions yet)

New fields exist in parallel with the old transform tables.

- `ShipTypeDefinition`: drop `base_turn_rate: f32`, add `base_angular_acceleration: f32`.
- `Ship`: add `movement: MovementState`.
- `CargoCrate`: add `movement: MovementState`.
- `Asteroid`: add `x: f32, y: f32` (rotation derived client-side from `id ⊕ time`).
- `Station`: add `x: f32, y: f32, rotation: f32`.
- `JumpGate`: add `x: f32, y: f32, rotation: f32`.
- Update `definitions/ship_type_definitions.rs` for `base_angular_acceleration`.

**Checkpoint:** `task server:publish-clear` succeeds.

---

## Phase 3 — Snapshot helpers

Two helpers, called from every site that touches movement.

New file: `server/src/logic/stellarobjects/movement.rs`

- `get_ship_movement_snapshot(dsl, ship_id) -> MovementState` — predict to `ctx.timestamp`, no write. Used by range checks (dock distance, mining range, jumpgate proximity).
- `write_ship_movement_snapshot(dsl, ship_id, |state| { ... })` — predict, mutate via closure, write. Used by movement input, dock/undock, jumpgate transit, jettison.
- Same pair for `CargoCrate`.

Both delegate to `solarance_shared::physics::predict_movement`. Two functions per entity beats a trait — keeps the call sites obvious in `spacetime logs` greps.

---

## Phase 4 — Movement input reducer

In `server/src/logic/ships/movement_controllers.rs`:

- Apply mutual-exclusion precedence: `forward && backward → 0`, `left && right → 0`.
- Update `ShipMovementController` flags as today.
- Compute desired linear/angular acceleration from `(f, b, l, r)` × `ship_type.base_acceleration` / `base_angular_acceleration`.
- `write_ship_movement_snapshot(dsl, ship_id, |s| { s.linear_acceleration = ...; s.angular_acceleration = ...; })`.
- Delete the existing manual damping logic (lives in `predict_movement` now).

The reducer name (`update_ship_movement_controller`) stays — keeps the client API stable.

---

## Phase 5 — Lifecycle event resets

Every entry/exit point routes through `write_*_movement_snapshot`.

| Site | What writes |
|---|---|
| `logic/ships/creation.rs` (player ship spawn) | spawn pos, zero velocity/ω |
| `dock_to_station` | zero everything, `last_update_time = now` |
| `undock_from_station` | pos = station (x,y), rotation = station-facing, zero dynamics |
| `try_to_use_jumpgate` | pos = arrival, **clean stop** (zero velocity + ω) |
| Cargo jettison from ship | snapshot with forward-toss velocity along ship heading |
| Cargo jettison from asteroid (no-cargo mining) | snapshot with random-angle velocity from asteroid (x,y) |

Cargo crate brake is a small random-range negative acceleration set at jettison time.

---

## Phase 6 — Position-mutation audit

Confirm no caller bypasses the helpers.

```bash
grep -rn "sobj_internal_transform\|sobj_velocity\|set_x\|set_y" server/src/logic
```

Every hit either gets routed through helpers or is dead code. `try_to_dock_to_station`'s distance check should use `get_ship_movement_snapshot` (read variant).

---

## Phase 7 — Cargo crate despawn sweeper

Crates eventually go away without per-crate timers.

- Add `cargo_crate_ttl_secs: u64` to `global_config` (default `14400` = 4 hr).
- New scheduled table `cargo_crate_despawn_sweeper_timer`, 30-min interval, registered in `lifecycle/timers.rs`.
- Reducer iterates all cargo crates, deletes any where `now - created_at > ttl`.

---

## Phase 8 — Client render path

- `task client:generate`.
- `client/src/gameplay/render.rs` + per-class draw paths:
  - Ship: read `ship.movement`, call `predict_movement(now_micros)`.
  - CargoCrate: same.
  - Asteroid: read `(x, y)`, derive spin angle from `asteroid_id ⊕ time` deterministically.
  - Station / JumpGate: read `(x, y, rotation)` direct.
- `client/src/stdb/connector/subscriptions.rs`: drop `sobj_velocity`, `sobj_hi_res_transform`, `sobj_low_res_transform`, `sobj_player_window`.

**Checkpoint:** `cargo check` clean on client; solo run boots and a ship moves smoothly with no rotation-arrest bug.

---

## Phase 9 — Delete the old machinery

Demolition. The new path is now load-bearing.

- `server/src/tables/stellarobjects.rs`: delete `StellarObjectVelocity`, `StellarObjectTransformInternal`, `StellarObjectTransformHiRes`, `StellarObjectTransformLowRes`, `StellarObjectPlayerWindow` and their `referenced_by` entries.
- `server/src/logic/stellarobjects/transforms.rs`: delete `recalculate_sobj_transforms` reducer + timer.
- `server/src/logic/stellarobjects/player_windows.rs`: delete the timer + helpers (audit callers — `create_sobj_player_window_for` is invoked from undock flow; that whole call goes away too).
- `server/src/lifecycle/timers.rs`: drop the two timer creations.

---

## Phase 10 — Verification (Definition of Done)

- `cargo check` server + client (no new warnings).
- `task server:publish-clear`.
- `task client:run` solo — confirm rotation arrest is gone and ship coasts naturally.
- **DoD:** two-machine maincloud test, eyeball smoothness for 5 min. Any visible jitter is logged in the clock-skew GH issue, not a blocker.

---

## Open implementation questions (decide as we hit them, not blocking)

1. **CargoCrate brake clamp at v=0** — does the prototype's `v_event_time` auto-zero velocity, or do we need a fix-up rewrite? Will know in Phase 5; if the latter, simplest is to schedule a one-shot snapshot rewrite at `now + v_to_zero_time`.
2. **Asteroid rotation field** — recommendation: omit entirely from the table; client derives spin from `asteroid_id` deterministically. Confirm in Phase 2.
3. **Seed-data scripts** that hard-code station/jumpgate positions via `sobj_internal_transform` — need to set new fields directly. Surfaces in Phase 6 audit.

---

## Out of scope (tracked separately)

See `docs/tmp/movement_system_follow-ups.md` for the four GitHub issues queued as follow-ups:

1. View functions for per-sector visibility scoping
2. Client-side prediction + reconciliation for the player's own ship
3. Minimum-latency `server_offset` clock-skew estimator + debug-widget readout
4. Generalize `combat/visual_effects.rs` for mining-asteroid effects
