# Client Architecture Review — Macroquad, Prediction, and the Road to MVP

> **Reviewed:** `client/` against `docs/Solarance_Beginnings_MVP_Design_Doc.md`, `docs/reports/mvp-gap-analysis.md` (server side, 2026-04-30), and the deep-modules guidance in `CLAUDE.md`.
>
> **Date:** 2026-05-17
> **Scope:** Client only. Server gaps are covered in the prior MVP gap report.
> **Verdict in one line:** **Stay on Macroquad through MVP. Fix three concrete classes of client-side debt before they become bugs.**

---

## 1. Executive Summary

The client *will* support the MVP loop (Find → Extract → Haul → Contribute → Watch it grow) without a rewrite. Macroquad + egui is the right tool for the scope the design doc calls out: 5–10 concurrent players, one solar system, ten hand-placed sectors, no combat.

The fears you raised are real but they are **not Macroquad's fault** and Bevy would not magically fix them. The three classes of problem worth addressing are:

1. **Cached SpacetimeDB rows go stale** when row-level security evicts them (sector switch, undock, sub-rotation). This is the source of the past bugs you mentioned and will recur as the world grows.
2. **`GameState` is a god object** — a parameter soup that 12 GUI windows all reach into.
3. **`stdb/utils.rs` is a shallow junk drawer**, not the deep module CLAUDE.md asks for.

None of these requires switching engines. They require modest, targeted refactors — the kind that pay for themselves the first time they prevent a bug. Sections 4–6 below propose concrete fixes scoped so each can land in a single PR without blocking the shared-building spike.

The Macroquad-vs-Bevy question is examined in §7. Short version: **defer until after MVP, and probably forever**.

---

## 2. What the Client Already Does Well

Credit where it's due — several things are already aligned with the design principles in `CLAUDE.md`:

- **`pose_for_object()` in `client/src/stdb/utils.rs:36–92`** is a genuine deep module. Caller asks "where is this stellar object right now?", implementation hides a 5-way dispatch over `StellarObjectKinds`, calls `solarance_shared::predict_movement()` for ships and cargo crates, and reads static columns for asteroids/stations/jumpgates. The comment on `gameplay/render.rs:36–38` documents exactly this contract. **Keep doing this.**
- **`predicted_player_snapshot()` (`stdb/utils.rs:94–114`)** wraps the get-ship + predict-movement two-step into one call. Same deep-module pattern.
- **Per-frame fresh pose computation.** The render loop in `gameplay/render.rs:35` iterates `db.stellar_object()` fresh every frame rather than caching a snapshot. This is the right default and avoids most staleness bugs on the *render* path.
- **Background starfield + parallax** via shader + `bg_camera` (`gameplay.rs:151–157`) is cleanly separated from sector rendering.
- **Generated bindings + `server/impls.rs` From conversions** keep type translation in one place.
- **Subscription queries are centralised** in `stdb/connector/subscriptions.rs`. Whatever the staleness story is, *what* the client subscribes to is at least co-located.

So the architecture has good bones. The work below is hardening, not rebuilding.

---

## 3. Does the Client Currently Support the MVP?

Reading the MVP design doc and walking through the client folder, here is the honest gap list:

### 3.1 Already there

| MVP requirement | Client status |
|---|---|
| Fly a corvette in a sector | ✅ `gameplay/player.rs` + `gameplay/render.rs` |
| Travel between sectors via jumpgates | ✅ `JumpGate` is rendered and targetable |
| Mine asteroids | ✅ Mining laser visual + `mining_active` flag in `GameState` |
| Cargo hold display | ✅ `ship_details_window.rs` (Cargo tab) |
| Chat (global/sector/faction) | ✅ `chat_widget.rs`, three channels wired |
| Faction display | ✅ `faction_window.rs` |
| Map (galaxy + sector) | ✅ `map_window.rs` |
| Ship creation + faction choice | ✅ `creation_window.rs` |
| Docked / out-of-play interface | ✅ `out_of_play_screen.rs` |
| Background starfield | ✅ shader + parallax |

### 3.2 Missing for the MVP shared-building spike

These are the gaps that block the **next spike** the design doc calls out ("Build the minimum shared-building loop between two players"):

| MVP requirement | Client gap | Estimated effort |
|---|---|---|
| **Contribute cargo to a station's construction pool** | The `contribute_to_station` reducer is generated in the bindings (per server's gap-analysis report), but no GUI calls it. `out_of_play_screen.rs` lacks a "Contribute" panel. | ~1 day. Add a panel inside the docked-station view that lists cargo items + a "deposit" button per stack. |
| **Visualise station construction progress** | The client subscribes to `station_under_construction` but no widget renders the progress bar, contributor count, or per-resource pool. | ~½ day. New widget under `gui/`, drawn in the in-sector HUD when a construction site is nearby, AND in the docked view. |
| **See another player contributing in real time** | Subscription is wired, but there's no UI affordance ("Sara just contributed 50 iron — 41% complete"). | ~½ day, depends on server pushing a `ServerMessage` event on contribution. |
| **Welcome-back screen** | No screen exists. The login flow drops straight into the in-sector view or `creation_window`. | ~1 day for the v0 text-only version (assets + station progress + simple counters). Server already has `ServerMessage` infrastructure per the prior report. |

None of these require architectural changes. They're additive GUI work on top of the existing window pattern.

### 3.3 Not needed for MVP, already present

These exist in the client and are fine to leave in place (they don't actively hurt MVP) but are technically post-MVP per the design doc:

- Weapon firing visualisation in `visual_effects.rs` (combat is absent from MVP)
- Combat-mode toggle (`Q` key, `state.combat_mode`)
- Ship details "Equipment" tab in `ship_details_window.rs`
- Multi-ship asset tree in `assets_window.rs` (MVP is one ship per player)
- Faction relations / standings tab in `faction_window.rs`

**Recommendation:** don't delete these — they're future scaffolding. But don't *iterate* on them either until the core loop is proven fun.

### 3.4 Will the MVP require massive rewrites?

**No.** The remaining MVP-blocking work is *additive*: 3–4 new windows/panels totalling ~3 days of focused work. The existing window pattern (`State` struct + `draw(egui_ctx, ctx, state, open)` function) is repeatable and well-understood at this point. There is no design constraint in the MVP doc that the current client cannot meet.

The risk is not "the MVP is unachievable on this codebase." The risk is "the MVP ships with a class of intermittent bug that you've already seen once, and will see again at the worst possible moment in front of David."

That class of bug is §4.

---

## 4. The Real Problem: Stale Cached Rows

This is the one you flagged and it deserves the headline. **It is the single highest-impact thing to fix.**

### 4.1 What's happening

SpacetimeDB row-level security determines which rows the client is subscribed to. When the player jumps sectors, undocks, or a subscription rotates, rows are *evicted* — they vanish from `ctx.db.foo().iter()` and `find()` returns `None`. Any code that previously cloned a row into client-local state now holds a ghost.

### 4.2 Concrete instances in the current client

| Field | File | Risk |
|---|---|---|
| `current_target_sobj: Option<StellarObject>` | `gameplay/state.rs:42` | Cloned row stored across frames. Read in 7+ places. If you target an asteroid then jump sectors, this row is stale. |
| `selected_ship: Option<Ship>` | `gameplay/gui/assets_window.rs:14` | Cached `Ship` row. If the docked ship subscription drops, the window reads ghost fields. |
| `selected_ship: Option<Ship>` | `gameplay/gui/out_of_play_screen.rs:33` | Same problem, full-screen UI. |
| `currently_selected_module: Option<(u8, StationModule, StationModuleBlueprint)>` | `gameplay/gui/out_of_play_screen.rs:32` | A *tuple* of two rows. Most fragile of the bunch. |

There are likely more — a quick grep for `Option<.*>` fields in `gui/*.rs` `State` structs will catch others.

### 4.3 What "fixed" looks like

The fix is a discipline, not a framework:

> **Client-side state may cache an ID. It may not cache a row.**

Every place that currently stores `Option<Ship>` should store `Option<ShipId>` (or the equivalent `u64`). Reads re-query the DB at the moment of use, and treat `None` as "the row is gone — clear my reference."

Pattern, applied to `current_target_sobj`:

```rust
// In GameState:
pub current_target_sobj_id: Option<u64>,

// Read site:
if let Some(id) = game_state.current_target_sobj_id {
    if let Some(target) = ctx.db().stellar_object().id().find(&id) {
        // use target — guaranteed fresh
    } else {
        // row was evicted — clean up
        game_state.current_target_sobj_id = None;
    }
}
```

This is annoying to write inline at every read site, which is exactly why §5 calls for a deeper utils module — `get_current_target(ctx, game_state) -> Option<StellarObject>` hides the validate-or-clear dance.

### 4.4 An even better long-term fix

Register `on_delete` callbacks for every table whose ID you cache. When a row is evicted, the callback clears the matching client-side state. This is the "push" version of the validate-on-read approach above. It's strictly better but requires more plumbing. **Do the validate-on-read fix first** (it's defensive against any source of staleness, including bugs in your own subscription queries) and add `on_delete` later if needed.

### 4.5 Why this matters disproportionately

The design doc's core promise to David is *intermittent play with progress preserved*. The worst-case failure of stale-row bugs is showing him a UI element ("Outpost Echo: 42% complete") referencing a row that no longer exists, then panicking when he clicks it. That kind of bug is the exact opposite of the "calm" experience the doc promises. **Fix this before the two-player spike, not after.**

---

## 5. `stdb/utils.rs` Is Half a Deep Module

`CLAUDE.md` says "deep modules" — simple interface, complex implementation. Looking at `client/src/stdb/utils.rs`:

### 5.1 The deep parts (keep)

- `pose_for_object` (lines 36–92) — 5-way dispatch + prediction. ✅
- `predicted_player_snapshot` (lines 94–114) — get-ship + predict. ✅
- `get_faction_shortname` (lines 136–150) — recursive parent walk. ✅

### 5.2 The shallow parts

- `get_player(db, id)` — one-line wrapper around `db.player().id().find(&id)`. Adds nothing.
- `get_player_sobj_id(ctx)` — one-line wrapper.
- `get_sector_name(ctx, id)` — one-line wrapper.

These don't earn their keep. They were probably added when the same call site was written twice and someone refactored. Per CLAUDE.md: *"new abstractions earn their place by hiding multi-step operations or domain logic, not by renaming a single DSL call."*

### 5.3 The missing parts

What's *not* in the utils module is more telling. Callers throughout the GUI do things like:

```rust
ctx.db().ship().iter().filter(|s| s.player_id == identity && s.location == ShipLocation::Docked)
```

…inline, in multiple files. That's a domain query ("all of my docked ships") that *should* be a function. Same for "all players in my faction", "all asteroids in this sector", etc.

### 5.4 Proposed: re-shape `stdb/utils.rs` around domain queries

Replace the thin wrappers with **domain-shaped accessors**, each of which hides:

- The DSL lookup
- The validate-or-return-None logic from §4
- The cross-table joins (ship → ship_object → ship_type_definition is currently inlined in `render.rs:46–51` and `status_widget.rs`)

Example shape:

```rust
// Replaces game_state.current_target_sobj
pub fn get_current_target(ctx: &DbConnection, game_state: &mut GameState) -> Option<StellarObject>;

// Replaces inline filter
pub fn get_my_ships_in_sector(ctx: &DbConnection, sector_id: SectorId) -> Vec<Ship>;
pub fn get_my_docked_ships(ctx: &DbConnection) -> Vec<Ship>;

// Replaces the ship → ship_type_definition dance in render
pub fn get_ship_with_type(ctx: &DbConnection, sobj_id: u64) -> Option<(Ship, ShipTypeDefinition)>;
```

These are deep — they hide multi-step lookups and staleness handling. **They are also the natural seam at which to introduce row-level-security defensive code in *one* place.**

---

## 6. `GameState` Has Become a God Object

### 6.1 What it currently holds

`GameState` (in `gameplay/state.rs:16–49`) holds:

- 8 GUI window `State` sub-structs
- 4 loose `open: bool` flags for windows
- The `DbConnection` reference
- 2 cameras
- 4 gameplay flags (`current_target_sobj`, `combat_mode`, `mining_active`, `movement_flags`)
- A `HashMap<u64, FiringEffect>` for visual effects

Every GUI window receives `&mut GameState` even when it only needs its own state struct and a boolean. The signature pattern across `gui/*.rs` is:

```rust
pub fn draw(
    egui_ctx: &Context,
    ctx: &DbConnection,
    state: &mut State,
    open: &mut bool,
) -> Option<...>;
```

…where `state` and `open` are both fields of `GameState`, plus several windows take the *whole* `&mut GameState` so they can flip a sibling window's open flag. That's the god object pattern.

### 6.2 Concrete cleanup

Three small moves, each independently shippable:

1. **Move `open: bool` into the window's own `State` struct.** Then `GameState` no longer needs `assets_window_open`, `details_window_open`, etc. Each window's `draw()` takes only its own `State`.
2. **Extract `WindowsState` from `GameState`.** Group the 8 window state structs into a sub-struct. `GameState` becomes: `windows`, `gameplay` (the gameplay flags), `display` (cameras), `ctx`. Four logical groups instead of 20 sibling fields.
3. **Replace `current_target_sobj: Option<StellarObject>` with `current_target_id: Option<u64>`** (per §4). This is the same change as the staleness fix and belongs in the same PR.

After these three moves, no GUI window needs `&mut GameState` — they each need only their own `State` and the `DbConnection`. The interface gets *smaller* than the implementation, which is the deep-module test from CLAUDE.md.

---

## 7. Macroquad vs. Bevy — The Honest Answer

You said: *"I'm hesitant to take on the switch from Macroquad to Bevy unless absolutely necessary, and probably after the MVP even if it is necessary."* That instinct is right. Here's why.

### 7.1 What ECS would actually buy you

Bevy's ECS would, genuinely, force better discipline in three places:

- **Explicit system ordering** — the current `gameplay.rs` main loop (lines 143–305) mutates `game_state` in 5+ places (visual effects update, render, egui callbacks, input handling, channel drains). Bevy makes that ordering a schedule.
- **Mutation safety at compile time** — Bevy's query system prevents two systems from mutably borrowing the same component simultaneously.
- **Natural separation of "data" from "render"** — Bevy components are tiny structs; the god-object pattern from §6 is harder to fall into.

### 7.2 What ECS would *not* fix

- **Stale SpacetimeDB rows** — this is a data-source problem, not a render-architecture problem. ECS doesn't know your DB rows were evicted.
- **Shallow utils modules** — that's a design discipline, independent of engine.
- **GUI complexity** — `bevy_egui` exists but the window state problem is the same problem in Bevy's world.

### 7.3 What the switch would cost

- 100% of `client/src/gameplay/` would be rewritten (rendering, input, GUI integration, main loop).
- The starfield shader would need to be re-plumbed via Bevy's render graph.
- `egui_macroquad` → `bevy_egui` — *most* widget code ports but it's not free.
- The asset loading path in `main.rs` (the macOS bundle handling) would need to be reworked.
- Probably 2–4 weeks of focused full-time work; for an evenings-and-weekends project with a full-time job + young kids, that's months of calendar time and a *lot* of motivation cost.

### 7.4 The verdict

Looked at against the MVP audience (5–10 concurrent players, one system, ten sectors, no combat, no NPCs), Macroquad is **comfortably within its design envelope**. The pain points in the current codebase are *not* the kind ECS resolves — they're staleness, encapsulation, and missing domain queries.

**Stay on Macroquad through MVP.** Revisit after the core loop is proven fun. Even then, the question to ask is not "would Bevy be nicer?" — it would. The question is "is the MVP-validated client genuinely blocked by Macroquad?" If David is playing happily on Macroquad, the answer is no.

If you do eventually switch, the work in §4–§6 makes the migration *easier*, not harder — a deeper utils module and a cleaner `GameState` translate directly to Bevy resources and systems.

---

## 8. Recommended Order of Operations

Each item below is sized to be a single focused PR. They are ordered by ratio of (risk reduced / hours invested).

| # | Work | Why now | Effort |
|---|---|---|---|
| 1 | **Stop caching rows. Cache IDs.** Convert `current_target_sobj`, `selected_ship`, `currently_selected_module` to ID-only. Add validate-or-clear at every read site. | Eliminates the entire class of staleness bug you've already hit once. Required before the two-player spike. | ½–1 day |
| 2 | **Add a `contribute_to_station` GUI panel** in `out_of_play_screen.rs`. | Unblocks the next spike per the design doc. | ~1 day |
| 3 | **Add a station-construction progress widget**, visible both in-sector and docked. | Required for the "watch it grow" emotional beat. | ~½ day |
| 4 | **Reshape `stdb/utils.rs` around domain queries**, with the staleness-handling baked in. Delete the one-line wrappers. | Pays back the cost of fix #1 — read sites get one line, not five. | 1 day |
| 5 | **Move `open: bool` into each window's `State`**; group window states into a `WindowsState` sub-struct on `GameState`. | Removes the god-object parameter passing. Each window now has a small, deep interface. | ½ day |
| 6 | **Build a v0 welcome-back screen** (text-only). | Final MVP-blocking client gap. | ~1 day |
| 7 | **Delete dead code** in `gameplay/player.rs:86–134` (the commented-out `_control_player_ship`). | Hygiene. Free. | 5 min |

**Total to reach "MVP-shaped client":** ~5–6 focused days. None of it requires switching engines, and items 1, 4, and 5 make a future Bevy migration easier if you ever do decide to.

---

## 9. Open Questions Worth Your Judgement

These are decisions only you can make; flagging them so they don't get silently defaulted:

1. **Server clock skew compensation** — `stdb/utils.rs:23–27` notes a deferred "Phase 10 server_offset estimator." At 5–10 players this may not be visible. If you ship MVP and David sees ships visibly snap on subscription updates, this is why. Decide whether to backlog or to address pre-MVP.
2. **`on_delete` callbacks vs. validate-on-read** — §4 recommends validate-on-read first. If you find yourself writing it five times, it may be worth the up-front cost of `on_delete` callbacks for each cached-ID type. Worth revisiting after fix #1 lands.
3. **Combat-system code carried in client** — currently `visual_effects.rs` and the combat-mode toggle exist but the design doc explicitly cuts combat. Leaving them in is fine, but worth deciding now whether *new* combat-adjacent code is permitted in the client between now and MVP. Recommend: no — even small additions accrete.

---

## 10. TL;DR

- **MVP is reachable on the current client.** ~5–6 days of focused work, no rewrite.
- **The bugs you've already seen come from cached SpacetimeDB rows going stale**, not from Macroquad. Fix that one discipline (cache IDs, not rows) and the class disappears.
- **`stdb/utils.rs` is half a deep module** — finish it, and you get a single place to handle staleness, joins, and domain queries.
- **`GameState` is a god object** — three small moves shrink it into something the GUI windows don't have to know the whole shape of.
- **Don't migrate to Bevy** until after MVP, and probably not even then. The pain points aren't ECS-shaped.
