
This document defines the initial Domain-Specific Language (DSL) for **Solarance: Beginnings**. Every developer, agent, and domain expert must use these exact terms when discussing the game, writing documentation, or naming variables/classes in the codebase. 

If a concept cannot be described using these terms, it is likely out of scope for the MVP.

## 1. The Core Loop (Verbs)
These are the primary actions a player can take. Do not use synonyms (e.g., use `Contribute`, not `Donate` or `Give`).

*   **Find:** The act of navigating to a hand-placed resource node. *(Code: `Location`, `ResourceSource`)*
*   **Extract:** The interaction of pulling raw resources from a source into the ship's inventory. Do not use "mine" as a system noun to avoid confusion with future minigame mechanics. *(Code: `ExtractAction`, `ExtractionYield`)*
*   **Haul:** The state of transporting extracted resources in a ship's inventory across space. *(Code: `HaulingState`, `ShipInventory`)*
*   **Contribute:** The critical transfer of hauled resources from a Player's Ship to a Station's `ContributionPool`. This is the primary driver of progression. *(Code: `ContributeResource()`, `ContributionEvent`)*
*   **Grow / Update:** The automatic server-side reaction when a `ContributionPool` meets its threshold, updating the station visually and mechanically for all players.

## 2. World Architecture (Nouns)
Strict hierarchical containers for the game world.

*   **Solar System:** The highest-level map container. There is only one in the MVP.
*   **Sector:** A discrete, hand-placed area within the Solar System. It has a singular, defined purpose (e.g., "Asteroid Sector"). There is no coordinate space outside of Sectors.
*   **Jumpgate:** The only method of moving between Sectors. A point-to-point teleportation entity.
*   **Station:** A persistent, player-upgraded structure. **Rule:** Exactly one per Sector, maximum. 
*   **Module:** A specialized component (Storage, Refinery, Production, Assembly, Research, Repair, Defense) added to a Station. Stations are simply "warehouses with Modules."
*   **Capital Station:** The single, massive root station for a specific Faction where new players spawn.
*   **Resource Source:** A generic, hand-placed point of interest in a sector where players perform the `Extract` action (e.g., an asteroid). 

## 3. Player & Entities (Nouns)
*   **Player / Client:** The human interacting with the game. Designed around "David" (intermittent play, requires pause-able progression).
*   **Ship:** The player's avatar. In MVP, this is strictly a jack-of-all-trades **Corvette**. A Ship is always in exactly one of two states — say which one you mean (see ADR-0002):
    *   **Piloted Ship:** `location == Sector`. Has a `StellarObject` + `MovementState`; the **only** kind of ship the sector simulation (movement, dead-reckoning, sector subscriptions, range checks, HUD/minimap) can see. *(Code: `get_player_ship` returns this — `None` while docked.)*
    *   **Docked Ship:** `location == Station`. Its `StellarObject` is **deleted** on dock (`sobj_id` is a `0` sentinel — not a FK); cargo/status/equipment persist on the `Ship` row. Invisible to all sector-scoped queries; reachable only via the `Ship` table by player/station id.
    *   *Avoid:* the bare phrase "the player's ship" in code or issues — it hides the piloted-vs-owned distinction that caused #149. UI that should always show (welcome-back, assets, notifications) must gate on **ownership**, not on a Piloted Ship existing.
*   **Faction:** A team identifier. In MVP, strictly limited to a string name and a color (`Lrak Combine` / Red, `Rediar Federation` / Blue). Determines which Stations a player can `Contribute` to.
*   **Contribution Pool:** The required list of resources a Station or Module needs to reach the next growth stage. 
*   **Welcome-Back Summary:** A text-only data payload delivered to the client upon login, detailing offline ticks, station progress, and personal asset state. *(Code: `WelcomeBackPayload`, `OfflineTickCalculator`)*

## 4. Systems & Architecture
*   **Social Convergence:** The design philosophy and network event of multiple players observing a shared `ContributionPool` update simultaneously. 
*   **Offline Pause:** The rule that personal asset generation ceases when a player disconnects.
*   **Worker Process:** The single server instance managing the MVP. (Do not use "Orchestrator" or "Server Mesh" for MVP tasks).

### Movement & Position
*   **Movement State:** The dead-reckoning physics snapshot stored as a column on every moving entity (`Ship`, `Cargo Crate`). Contains position, velocity, acceleration, rotation, angular velocity, angular acceleration, caps, damping flags, and `last_update_time`. Lives in `solarance-shared::physics::MovementState` so client and server share identical extrapolation. Replaces the old `sobj_velocity` / `sobj_internal_transform` / `sobj_hi_res_transform` / `sobj_low_res_transform` tables.
*   **Predict Movement:** The pure function `predict_movement(state, current_time)` in `solarance-shared` that extrapolates a `MovementState` forward to a target time. Called by both client (every frame) and server (before any read or write of position).
*   **Snapshot Read:** `get_*_movement_snapshot(dsl, id)` — predict to `ctx.timestamp` without writing. Used for range checks (dock proximity, mining range, jumpgate proximity).
*   **Snapshot Write:** `write_*_movement_snapshot(dsl, id, |state| { ... })` — predict, mutate via closure, write. Used by movement input, dock/undock, jumpgate transit, jettison.
*   **Server Offset:** Client-side estimator that aligns the local clock to the server's clock so `predict_movement` doesn't see negative deltas (frozen ship) or oversized deltas (snap-forward). Computed as a maximum aggregator over recent snapshots; surfaced in the debug widget.
*   **Static Position:** Non-moving entities (`Asteroid`, `Station`, `Jumpgate`) carry `(x, y)` (and `rotation` for stations and jumpgates) as direct table columns. They do not have a `MovementState`. Asteroid spin is pure client-side animation derived from `asteroid_id ⊕ time`.

## 5. Messaging
Two distinct messaging systems. They do not share tables. Choosing between them *is* choosing the audience, which is why there is no separate "scope" column.

*   **Direct Server Message** (`DirectServerMessage`): A 1-to-1 message FROM the Server TO a single Player. Server is *always* the sender — there is no sender enum on this table. Replaces the old `ServerMessage` + `ServerMessageRecipient` pair and most existing `send_server_message_*` usages. The **Welcome-Back Summary** is delivered as a Direct Server Message. Player→player DMs are **out of MVP scope**; when added they will be a *separate* table (`DirectMessage`), not a reschema of this one. *(Do not use: ServerMessage, notification, PM, whisper.)*
*   **Channel Message:** A 1-to-many message posted to a **Channel**. Everyone subscribed to that Channel sees it. Fire-and-forget — no per-player delivery or read state is stored. *(Do not use: broadcast, group message.)*
*   **Channel:** The audience of a Channel Message. This replaces issue #101's proposed `Scope` enum: scope is *which Channel table you posted to*, not a stored column. The Channels, broadest to narrowest:
    *   **Server** (`ServerChannelMessage`): server-wide MOTD / updates. Sender is always `System`. Truly **public** — readable by anyone connected (incl. not-logged-in / banned) and surfaceable outside the game (e.g. a webpage). No View.
    *   **Galaxy** (`GalaxyChannelMessage`): galaxy-wide *player* chat. Gated to logged-in players (good-standing check added later) via a View. Distinct from Server so player chatter never mixes with official announcements.
    *   **Star System** (`StarSystemChannelMessage`): scoped to one `StarSystem` (`system_id`). In MVP there is one StarSystem, so this is *effectively* galaxy-wide today, but stays separate so it scopes correctly when more systems ship.
    *   **Sector** (`SectorChannelMessage`): scoped to one `Sector` (`sector_id`); audience is players whose Ship is in that Sector.
    *   **Faction** (`FactionChannelMessage`): scoped to one `Faction` (`faction_id`).
*   **Message Sender** (`MessageSender` enum): The author of a **Channel Message** — `Player(Identity)` or `System`. Lives only on the five channel tables (so System can post into any channel). `DirectServerMessage` has *no* sender field — its sender is always the Server, implicitly.
*   **Read state (login-relative):** Not stored server-side for either system. A message is "unread" if its `created_at` is later than the Player's `last_login`; the client highlights those and clears the highlight once the Player next sends a message.

## 6. Anti-Concepts (Banned Terminology)
To prevent scope creep, these terms are explicitly banned from MVP code, PRs, and design discussions. Once the MVP is released, this list will be updated. If you see them, flag them for the `Future Vision` backlog:

*   🚫 **Combat / Attack / Health (for ships) / Weapons** -> (Exterminate pillar is absent).
*   🚫 **Markets / Economy / Buy / Sell / Trade (between players)** -> (Exchange collapses into "Haul" and "Contribute").
*   🚫 **Explore / Procedural / Heatmap / Scan / Wormhole** -> (Exploration is stubbed; Sectors are hand-placed).
*   🚫 **Orgs / Guilds / Squads** -> (Only Factions exist).
*   🚫 **AI / NPCs / Pirates / Civilians** -> (No AI actors exist in MVP).
*   🚫 **Offline Income / Passive Generation** -> (Violates Offline Pause).
*   🚫 **Orchestrator / Handoff** -> (Over-engineering for <10 concurrent players).

---

### Messaging design decisions (issue #101, resolved 2026-05-30)
- `ServerMessage`/`ServerMessageRecipient` are split into **Direct Message** (1-to-1, server can be sender) and **Channel Message** (1-to-many, fire-and-forget). Six message tables total: `ServerChannelMessage`, `GalaxyChannelMessage`, `StarSystemChannelMessage`, `SectorChannelMessage`, `FactionChannelMessage`, `DirectServerMessage`. The five channel tables share `{ id, sender: MessageSender, body, created_at }` plus their indexed scope key; `DirectServerMessage` is `{ id, to: Identity (indexed), body, created_at }` with **no** sender field (server is always sender).
- **`MessageSender` enum lives only on channel tables**, not on `DirectServerMessage`. Player→player DM, if ever built, is a separate `DirectMessage` table.
- **Server vs Galaxy are deliberately two tables**: Server = official/public/no-view; Galaxy = player chat/gated/view. Kept separate so post-MVP multi-system growth is clean and so official announcements can live outside the game client.
- **No server-side windowing on any channel.** Views return the full filtered set; the client may limit how many it subscribes to. Client windowing (default last ~10 + "look back") is **post-MVP**.
- **View legality (no `.iter()` full scans):** StarSystem/Sector/Faction/DM views filter on their natural indexed FK. **Galaxy** has no natural key, so it carries a constant indexed `galaxy_id` (always `0` in MVP) the view filters on, plus an in-body `is logged-in?` gate (non-players / banned get an empty result). The `galaxy_id` column exists solely to keep the gated, un-windowed view legal; the whole-table read set is acceptable at MVP scale. Server channel needs no view, so no such key.
- The `ServerMessageRecipient` join table is dropped — DM recipient is a column, read state is login-relative and client-derived.
- Issue #101's `Scope` enum and `priority` field are dropped: scope is the Channel; priority is not modeled.
- **Sender is a `MessageSender` enum** (`Player(Identity)` + `System` to start; `Station` etc. added when needed). It is display-only.
- **All message tables are private; clients see them only through STDB Views** (`#[spacetimedb::view]`). Views are read-only Rust functions, so they may branch on enums/`Option` freely (the `MessageSender` enum is fine). Two real View constraints drive the schema instead:
  - **No full table scans** — a View may only reach a table via *indexed* `.find()`/`.filter()`. So every per-audience View filters on an **indexed** column (`to`, `sector_id`, `faction_id`). A View that returns "all rows" is illegal.
  - **Per-caller Views are O(subscribers)** — a View using `ctx.sender()` (`ViewContext`) is recomputed per client. DM/Sector/Faction Views are per-caller (unavoidable; fine at <10 players). Prefer `AnonymousViewContext` (shared) where the audience isn't caller-specific.

### How to use this document
- **When writing Code:** Name your classes strictly after these terms. E.g., `class ContributionPool`, `struct WelcomeBackSummary`, `fn extract_resource()`.
- **When writing Tickets/Commits:** Use the verbs defined here. E.g., *"Fix bug where Haul state drops inventory on Jumpgate use."*
- **When talking to AI / Agents:** Reference these terms directly. E.g., *"Write a function that calculates the Welcome-Back Summary based on the Offline Pause rules."*