# SpacetimeDB View Functions for Per-Player Visibility

## The Problem

In the original repo, most tables are `public` — every subscribed client receives every row. This means:

- A player in Sector 7 receives position data for ships in Sectors 1–100.
- The client has to filter locally what it actually cares about.
- Bandwidth scales with the total number of entities in the game, not the number in the player's sector.

SpacetimeDB **view functions** solve this by running a server-side query at subscription time. Each client subscribes to a view and receives only the rows that query returns *for them*.

---

## How Views Work in SpacetimeDB

A view is declared with `#[view(accessor = name, public)]`. It receives a `ViewContext` which includes `ctx.sender()` (the subscribing player's identity) and `ctx.from` (a read-only handle to all tables). It returns either a `Vec<Row>` or an `impl Query<Row>`.

SpacetimeDB evaluates the view per-client and keeps the client's subscription live — when underlying table rows change, the view is re-evaluated and diffs are pushed to the client.

**Views do not require the underlying tables to be `public`.** The tables can be private; the view acts as the controlled access point.

---

## Prototype Implementations

### Ships in the player's current sector

```rust
#[view(accessor = current_sector_ships, public)]
pub fn current_sector_ships(ctx: &ViewContext) -> impl Query<SpaceShip> {
    ctx.from
        .player_state()
        .filter(|player| player.id.eq(ctx.sender()))
        .right_semijoin(ctx.from.space_ship(), |player, ship| {
            player.current_sector_id.eq(ship.sector_id)
        })
}
```

- `player_state` is private. No client can read the full table.
- `space_ship` is also private. No client can read all ships in the game.
- Only ships whose `sector_id` matches the calling player's `current_sector_id` are returned.
- When the player calls `travel_to_sector`, `player_state` updates, the view re-evaluates, and the client automatically receives the new sector's ships and loses the old ones.

### Bullets in the player's current sector

```rust
#[view(accessor = current_sector_bullets, public)]
pub fn current_sector_bullets(ctx: &ViewContext) -> impl Query<Bullet> {
    ctx.from
        .player_state()
        .filter(|player| player.id.eq(ctx.sender()))
        .right_semijoin(ctx.from.bullet(), |player, bullet| {
            player.current_sector_id.eq(bullet.sector_id)
        })
}
```

Same pattern as ships — client only gets bullets in their current sector.

### Visible sectors within the current system

```rust
#[view(accessor = current_system_visible_sectors, public)]
pub fn current_system_visible_sectors(ctx: &ViewContext) -> impl Query<Sector> {
    let player = dsl.get_player_state_by_id(...)?;

    ctx.from
        .sector()
        .r#where(|sector| sector.system_id.eq(player.current_system_id))
        .left_semijoin(ctx.from.visited_sector(), |sector, visited| {
            sector.id.eq(visited.sector_id)
        })
}
```

Sectors are private. The player can only see sectors in their current system that they have a `visited_sector` record for (either from actually visiting, or from server-side "observe" on public sectors when they first enter the system).

### Per-player state (self only)

```rust
#[view(accessor = my_player_state, public)]
pub fn my_player_state(ctx: &ViewContext) -> Vec<PlayerState> {
    match ctx.db.player_state().id().find(ctx.sender()) {
        Some(p) => vec![p],
        None => Vec::new(),
    }
}
```

The player can read only their own `PlayerState` row.

---

## Pattern Summary

| Goal | Pattern |
|---|---|
| Player sees only their own row | `ctx.db.table().id().find(ctx.sender())` |
| Player sees entities in their current sector | `semijoin` on `player_state.current_sector_id == entity.sector_id` |
| Player sees explored parts of current system | `semijoin` against `visited_sector` or `visited_system` |
| Player sees no entity at all | Make the underlying table private with no view |

---

## Applying This to the Original Repo

### Replace public tables with view-gated tables

| Old (original repo) | New approach |
|---|---|
| `sobj_hi_res_transform, public` — all clients get all positions | `Ship` table private, `current_sector_ships` view returns only sector-local ships with `MovementState` |
| `sobj_low_res_transform, public` | Eliminated — dead reckoning removes the need for a low-res table |
| `ship_status, public` | Move to private; expose via `current_sector_ships` view (embed health/shields on `Ship`) or a separate per-sector view |
| `stellar_object, public` | Make private; expose only via sector-scoped views |

### PlayerState as the anchor

The key pivot table is `PlayerState` (or equivalent). Every sector-scoped view semjoins against it using `ctx.sender()`. When the player moves sectors, update `player_state.current_sector_id` in one reducer call — the view system handles propagating the new subscription state automatically.

### Fog of war

`visited_system` and `visited_sector` rows act as the fog-of-war database. The `current_system_visible_sectors` view implements fog by joining against these records. Extend this pattern to:

- Gate which stellar objects are visible (has the player ever been in the sector?).
- Gate which system-level intel is visible.
- Gate NPC ship positions to only players within sensor range (by adding a distance check to the semijoin predicate).

---

## What Views Cannot Do (Current SpacetimeDB Limitations)

- Views cannot currently do range/distance queries as a filter predicate — you cannot express "ships within 1000px of the player" directly in a view semijoin. The workaround is sector-level scoping (all ships in sector) which is coarser but sufficient for most gameplay.
- Views are read-only — they cannot call reducers or modify tables.
- A view returning `Vec<Row>` (rather than `impl Query`) loads all matched rows into memory on re-evaluation; prefer `impl Query` for large result sets.
