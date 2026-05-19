# Reducer and Movement Refactor — Client Compatibility Update

**Date:** 2026-05-06
**SDK version:** spacetimedb-sdk 2.2.0

---

## Background

The spacetimedb-sdk was bumped to 2.2.0 and the server schema was refactored. The old `PlayerShipController` type — a monolithic struct that combined movement, combat, mining, docking, and targeting flags — was removed from the server entirely. Its responsibilities were split across two new tables and a set of explicit reducers:

| Old | New |
|-----|-----|
| `PlayerShipController` (all-in-one) | `ShipMovementController` (movement flags, `id: Identity`) |
| `PlayerShipController.fire_weapons/fire_missiles` | `fire_weapons(target_sobj_id: u64)` reducer |
| `PlayerShipController.mining_laser_on` | `try_mining_asteroid(asteroid_sobj_id)` / `stop_mining_asteroid()` reducers |
| `PlayerShipController.dock` | `undock_ship(ship)` reducer |
| `PlayerShipController.targetted_sobj_id` | Client-local state only (no server persistence yet) |
| `PlayerShipController.cargo_bay_open` | Removed — no replacement reducer exists yet |
| `update_npc_controller` reducer | Removed entirely |

`NpcShipController` (`id: u64`) is now server-only; all its reducers (`set_npc_controller_target`, `update_npc_behavior`, etc.) call `try_server_only` and cannot be invoked by clients.

---

## Files Changed

### `client/src/gameplay/state.rs`
Added `mining_active: bool` to `GameState` (initialized `false`). Used to track whether the player has an active mining beam, since the `ShipMiningTimer` table is not available in the client cache.

### `client/src/gameplay/player.rs`
- Removed the entire input-processing block that looked up `npc_ship_controller` and set flags on it.
- Kept the combat mode toggle (Q key — local state).
- **Movement input is stubbed.** A `ShipMovementController` table exists (`id: Identity`) but the server has no public reducer for clients to update its `forward/backward/left/right` flags yet. This will be implemented when the server-side `update_ship_movement_controller` reducer is added.

### `client/src/gameplay.rs`
- Removed the E-key block that looked up `npc_ship_controller` by identity (type mismatch: `id` is `u64`, not `Identity`) and called `update_npc_controller`.
- Replaced with direct `game_state.current_target_sobj` mutation — targeting is now client-local state only.

### `client/src/gameplay/gui/minimap_widget.rs`
- Removed the `npc_ship_controller` lookup gate around `list_sector_objects`.
- Removed `controller: &mut PlayerShipController` parameter from `list_sector_objects`.
- Target selection in the minimap now reads/writes `game_state.current_target_sobj` directly instead of going through a server controller row.

### `client/src/gameplay/gui/status_widget.rs`
Complete rewrite of the ship function button strip:

| Button | Before | After |
|--------|--------|-------|
| Cargo Bay | toggled `controller.cargo_bay_open` | **Removed** (no replacement reducer) |
| Mining Beam | toggled `controller.mining_laser_on` | Calls `try_mining_asteroid` / `stop_mining_asteroid`; enabled only when target is an Asteroid |
| Autodocking | toggled `controller.dock` | Calls `undock_ship(player_ship)` |
| Fire Weapons | set `controller.fire_weapons = true` | Calls `fire_weapons(target.id)`; enabled only when a target is selected |
| Fire Missiles | set `controller.fire_missles = true` | **Removed** (no player missile reducer exists) |

Helper function signatures changed from `(ui, controller: &mut PlayerShipController, game_state)` to `(ui, ctx: &DbConnection, game_state: &mut GameState)`.

### `client/src/gameplay/render/in_sector.rs`
- Removed `controller: &PlayerShipController` parameter from `draw_mining_laser`.
- Mining laser is now drawn when `game_state.mining_active` is `true`.

### `client/src/gameplay/render.rs`
- Removed the `npc_ship_controller().id().find(&ctx.identity())` gate (type mismatch: `id` is `u64`, not `Identity`).
- Replaced with a `player_transform.is_some()` check — player-specific rendering (mining laser, player ship on top, radar) now runs whenever the player has a hi-res transform.
- Updated `draw_mining_laser` call to match new signature.

### `client/src/gameplay/gui/debug_widget.rs`
- Changed "Has Controller" debug check from `npc_ship_controller().id().find(&player.id)` to `ship_movement_controller().id().find(&player.id)`.
- `ShipMovementController.id` is `Identity`, which matches `player.id`.

---

## Known Gaps / TODOs

- **Ship movement is broken.** The server's `ShipMovementController` table has `forward/backward/left/right` flags that the physics timer reads, but no public reducer exists for clients to set them. Movement input is stubbed in `player.rs` pending a server-side `update_ship_movement_controller` reducer.
- **Targeting is not persisted to the server.** `game_state.current_target_sobj` is reset on reconnect. A player targeting reducer may be needed in the future.
- **Cargo bay open/close UI was removed.** No equivalent server reducer exists; feature will need to be re-added when the server API is defined.
- **Player missile firing was removed.** No player missile reducer exists in the current server schema.
