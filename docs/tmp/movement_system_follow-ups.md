# Movement-System Rewrite — Follow-up GitHub Issues

These four work items were deferred from the movement-system rewrite (`docs/tmp/movement_system_plan.md`) so the foundational refactor could land focused. Each section below is a self-contained briefing — a fresh agent should be able to read one section and produce a polished GitHub issue without prior conversation context.

**How to use this doc:** start a new conversation per issue. Hand the agent the relevant section of this file plus a pointer to the repo. The agent should:
1. Read the cited reference files to confirm current state.
2. Draft a GitHub issue title + body.
3. Either open it via `gh issue create` (if the user authorizes) or output the markdown for manual posting.

The goal of each issue is to **describe the problem, the constraints, and the suggested approach** — not to commit to a specific implementation. Issues are scoping artifacts, not detailed designs.

---

## Issue 1 — `#[view]` per-sector visibility scoping

### Background

Most tables in the server module are declared `public` (e.g. `ship`, `asteroid`, `cargo_crate`, `visual_effect`). SpacetimeDB pushes every row of every public table to every subscribed client. With the new dead-reckoning movement system, ship/crate positions live as a `MovementState` field directly on those tables — meaning bandwidth scales with **total entity count across all sectors**, not entities in the player's current sector.

For the MVP-scale game (handful of sectors, two-digit player count) this is acceptable. As the game grows it becomes the dominant bandwidth cost.

SpacetimeDB has `#[view(accessor = name, public)]` functions that run a server-side query at subscription time, evaluated per-client. The view receives a `ViewContext` with `ctx.sender()` (the calling player's identity) and returns `impl Query<Row>` or `Vec<Row>`. **The underlying tables can be private**; the view is the controlled access point. When underlying rows change, the view re-evaluates and diffs are pushed to the client.

### Problem

After the movement-system rewrite, the client receives every ship's `MovementState` for every sector it is not in. We want the client to receive only entities in its current sector.

### Proposed approach (sketch — issue should describe, not commit)

- Convert `ship`, `asteroid`, `cargo_crate`, `visual_effect`, and `stellar_object` from `public` to private.
- Anchor on a `PlayerState`-style row (or derive from the player's `Ship.sector_id`).
- Add views like `current_sector_ships`, `current_sector_cargo_crates`, `current_sector_asteroids`, `current_sector_visual_effects`.
- Each view is a semijoin: `WHERE entity.sector_id == this_player.current_sector_id`.
- Replace the existing `SELECT * FROM ship` etc. subscriptions in `client/src/stdb/connector/subscriptions.rs` with view subscriptions.

### References

- `solarance-shared/docs/handoff/view-functions-visibility.md` — prototype implementation with worked examples (current_sector_ships, current_sector_bullets, current_system_visible_sectors, my_player_state). This is the primary reference.
- `client/src/stdb/connector/subscriptions.rs` — current subscription queries.
- `CLAUDE.md` (project root) — SpacetimeDB Rust rules, project DSL conventions.

### Non-goals

- Distance-based culling (e.g. "ships within 1000px of the player"). SpacetimeDB views don't currently support range predicates; sector-level scoping is the chosen granularity.
- Fog-of-war / `visited_sector` mechanics — separate future work.
- Client-side prediction reconciliation (separate issue, see #2 below).

### Acceptance criteria

- Issue describes which tables become private, which views are added, and what each view's filter predicate is.
- Issue notes the subscription rewrites needed on the client.
- Issue calls out that this depends on the movement-system rewrite landing first (so the schema is stable before views are designed).

---

## Issue 2 — Client-side prediction + reconciliation for the player's own ship

### Background

After the dead-reckoning movement system lands, every ship's position is computed via `predict_movement(state, now)` from the last server snapshot. For **non-self ships and cargo crates**, this hides RTT entirely — the snapshot was authoritative as of `last_update_time`, and predicting forward to "now" gives a smooth extrapolation regardless of network latency.

For the **local player's own ship**, RTT lag is still visible: the player presses W, the input reducer round-trips to the server (~50ms cloud RTT), the server writes a new MovementState snapshot, the client receives it, and only then does the ship begin accelerating on screen. The result feels sluggish.

The standard MMO networking pattern for this is **client-side prediction with reconciliation**:
1. On local input change, the client immediately applies the same input mapping to a **predicted** local copy of its ship's `MovementState`.
2. The client renders from the predicted state, so input feels instant.
3. The client also fires the reducer to the server.
4. When the authoritative server snapshot arrives, the client compares predicted vs. authoritative.
5. If they agree (within a small tolerance), keep predicting from the new authoritative state. If they diverge (e.g. server rejected the input, or another reducer mutated the state), lerp the predicted state toward authoritative over ~100ms to avoid a visible snap.

### Problem

Without prediction, controlling the player's own ship feels laggy under any non-trivial RTT. The grilling deferred this because:
- SpacetimeDB is "made for low-latency large datasets" — local network testing didn't surface the problem.
- The MVP target persona ("David, 38") is intermittent / non-twitch — a small amount of input lag is tolerable.
- Implementing prediction correctly requires careful tracking of which inputs have been acked vs. pending.

It is, however, a known scaling risk and will become noticeable once the game is on maincloud with users on different continents.

### Proposed approach (sketch)

- Maintain a `predicted_movement: MovementState` on the client, separate from the server-broadcast `Ship.movement`.
- On input change: apply the same flag → acceleration mapping the server uses, write the predicted state with `last_update_time = client_local_micros() + server_offset`.
- Render the local player's ship from `predicted_movement`; render all other ships from `ship.movement` as today.
- On every authoritative snapshot received: compare position/velocity. If divergence > threshold (tunable; e.g. 5px or 0.5 m/s), lerp predicted toward authoritative over a short window.
- Unify the input mapping into a shared function in `solarance-shared::physics` so client and server compute identical accelerations.

### Dependencies

- Movement-system rewrite (`docs/tmp/movement_system_plan.md`) must land first.
- Clock-skew estimator (Issue 3) must land first — prediction without a stable `server_offset` will worsen, not improve, the experience.

### References

- `solarance-shared/src/physics/mod.rs` — the prediction function (server-authoritative).
- `solarance-shared/docs/handoff/dead-reckoning-movement.md` — design rationale for the snapshot model.
- `client/src/gameplay/player.rs` — current `control_player_ship` flow.
- `client/src/gameplay/render.rs` — current render fan-out.

### Non-goals

- Reconciling predictions for non-self ships or cargo crates.
- Lag compensation for cross-player interactions (mining, docking proximity) — server remains authoritative.
- Rollback netcode (we do not re-simulate past frames; we only lerp).

### Acceptance criteria

- Issue describes the predicted-state model, the divergence-detection rule, and the lerp duration.
- Issue calls out the dependency chain (movement rewrite → clock skew → prediction).

---

## Issue 3 — Minimum-latency `server_offset` clock-skew estimator

### Background

The dead-reckoning movement system's `predict_movement(state, current_time)` extrapolates an entity's position by computing `delta = current_time - state.last_update_time` and integrating velocity/acceleration over that delta.

- `state.last_update_time` is in **server-clock microseconds** (set by the server when it wrote the snapshot).
- The client's `current_time` is its **local-clock microseconds**.
- These two clocks are not the same. They drift, they're set differently at boot, and they have offsets relative to each other on the order of tens to hundreds of milliseconds.

**Failure modes:**

- **Client clock behind server:** `delta` is small or negative. The prototype clamps to "return stored values as-is" — the ship appears frozen, then snaps forward when the next snapshot arrives.
- **Client clock ahead of server:** `delta` is too large. The ship visibly snaps forward each frame past the true position; when the next snapshot arrives, it appears to snap backward.

The prototype was only tested on LAN/loopback where the offset is microseconds and the issue is invisible. On maincloud the offset will be tens of milliseconds, making both failure modes user-visible.

### Solution

Maintain a per-client `server_offset` estimator. For every snapshot received, compute a candidate offset:

```
candidate = snapshot.last_update_time - client_local_micros_at_receipt
```

The candidate is biased upward by network latency: the snapshot was written at `last_update_time` server-side and arrived `RTT/2` later client-side. So `candidate` overestimates the true server offset by ~`RTT/2`.

The aggregator should be a **maximum**: across many candidates, the largest value corresponds to the snapshot that was delivered with the lowest network latency, which gives the most accurate offset. Use a slow decay (e.g. -100 µs/sec) to handle long-term clock drift.

When predicting, pass `current_time = client_local_micros() + server_offset` to `predict_movement`.

### Goal

- Implement the estimator (~30 LOC).
- Display the running average and per-snapshot deltas in `client/src/gameplay/gui/debug_widget.rs` so the developer can confirm correctness while testing.
- No automated test — this is validated by eye on the two-machine maincloud test (DoD for the movement-system rewrite).

### References

- `solarance-shared/src/physics/mod.rs` — `predict_movement`. Note the early-return branch when `delta` is non-positive: that branch is the "cozy freeze" failsafe and should remain even with the offset applied.
- `client/src/gameplay/gui/debug_widget.rs` — existing debug overlay.
- `solarance-shared/docs/handoff/dead-reckoning-movement.md` — explains the snapshot model.

### Non-goals

- NTP-style multi-sample synchronization. Overkill for this scale.
- Server-side adjustments (server is authoritative; only the client adjusts).
- Predicting beyond what the snapshot's stored caps allow.

### Acceptance criteria

- Issue describes the maximum-aggregator algorithm and the decay rule.
- Issue specifies what gets shown in the debug widget (running avg + last N deltas).

---

## Issue 4 — Generalize `combat/visual_effects.rs` for mining-asteroid effects

### Background

The MVP loop is `Find → Extract → Haul → Contribute → Grow`. The `Extract` action involves firing a mining laser at an asteroid for a few seconds. This visual should render consistently across all clients in the same sector — the player who fires sees the laser, and other players in the sector see it too.

The codebase already has a `visual_effect` table and timer system in `server/src/tables/combat.rs` and `server/src/logic/combat/visual_effects.rs`, designed for combat:

```rust
#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum VisualEffectType {
    WeaponFire,
    MissileFire,
    Explosion,
}

#[table(accessor = visual_effect, public)]
pub struct VisualEffect {
    id: u64,
    sector_id: u64,
    source: Vec2,
    target: Vec2,
    effect_type: VisualEffectType,
    created_at: Timestamp,
}
```

A scheduled `visual_effect_timer` deletes these rows after a short duration so the broadcast is a transient flash visible to all subscribers.

### Problem

Mining currently has no shared visual broadcast. Each client renders its own laser-effect locally when their own ship mines, but other players in the same sector see nothing. This breaks the "Social Convergence" pillar (per `CONTEXT.md`) — mining alongside another player should feel shared.

### Proposed approach (sketch)

- Rename `combat/visual_effects.rs` → `gameplay/visual_effects.rs` (or similar) since it is no longer combat-specific. Move the table to `tables/visual_effects.rs`.
- Add variants to `VisualEffectType`:
  - `MiningLaser` — continuous beam from ship to asteroid.
  - Possibly `MiningHit` — a small particle burst at the asteroid endpoint per tick.
- Emit a `MiningLaser` `VisualEffect` from the mining reducer (`try_mining_asteroid` and the periodic mining tick) with `source = ship.position`, `target = asteroid.position`.
- Client renders all `VisualEffectType::MiningLaser` rows for the current sector identically.

### Dependencies

- Movement-system rewrite must land first — `source: Vec2` on visual effects will be derived from the new `MovementState.position` for ships and the new `(x, y)` fields on asteroids.

### References

- `server/src/tables/combat.rs` — `VisualEffectType` enum and `VisualEffect` table.
- `server/src/logic/combat/visual_effects.rs` — the timer cleanup logic.
- The mining reducer (`try_mining_asteroid_reducer.rs`, server-side handler in `logic/ships/...` — agent should `grep -rn "try_mining"` to locate exact path).
- `CONTEXT.md` § 4 — Social Convergence design pillar.

### Non-goals

- Damage / destruction logic for asteroids. Separate concern handled by the mining reducer itself.
- 3D/particle systems. The visual is a 2D line + simple sprite.
- Cross-sector visibility — a player must be in the same sector to see the effect.

### Acceptance criteria

- Issue describes the rename / move, the new enum variants, and the emission point in the mining reducer.
- Issue notes the dependency on the movement-system rewrite (because positions move from transform tables to direct fields).
