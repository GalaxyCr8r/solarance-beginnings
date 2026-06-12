---
status: accepted
date: 2026-06-12
prompted-by: "#149"
---

# Docking removes the ship from the sector simulation

## Context

When a ship docks (`dock_to_station`, `server/src/logic/ships/station_interactions.rs`), its `StellarObject` row is **deleted**, its movement snapshot is zeroed, `Ship.location` flips to `Station`, and `Ship.sobj_id` is set to the sentinel `0`. Undocking creates a brand-new `StellarObject` at the station's sector. A docked ship therefore has no position, no movement state, and no presence in any sector-scoped query — only its `Ship` row (cargo, status, equipment) persists.

## Decision

A docked ship **exits the sector simulation entirely**. "The player's ship" is consequently an ambiguous phrase, and the codebase distinguishes two concepts:

- **Piloted Ship** — `location == Sector`; has a `StellarObject` + `MovementState`; the only kind of ship the sector simulation (movement ticks, dead-reckoning, sector subscriptions, minimap, range checks) can see.
- **Docked Ship** — `location == Station` (or `Ship` for carrier docking); no `StellarObject`; reachable only through the `Ship` table by player/station id.

`get_player_ship` (client, `stdb/utils.rs`) returns the **Piloted** ship only — `None` while docked. This is correct behavior, not a bug, but the name does not say so; issue #149 (welcome-back panel suppressed for docked players) came from exactly this misreading.

## Why this shape

The rejected alternative was keeping the `StellarObject` alive with a `docked` flag. That would make "where is my ship" a single uniform query, but every movement tick, sector scan, collision/range check, and dead-reckoning extrapolation would need a docked-filter — and a forgotten filter means parked ships drifting through dock walls. Deleting the sobj makes the docked state *structurally* invisible to the simulation: the cheap, default-safe failure mode is "docked ships are absent," not "docked ships are erroneously simulated."

## Consequences

- Client UI must decide per-surface whether it cares about the **Piloted** ship (HUD, minimap, movement) or **any owned** ship (assets, welcome-back panel, out-of-play screen). Gating a should-always-show surface on the Piloted ship is the #149 failure mode — new UI should default to ownership checks (`ship table by player_id`) unless it genuinely needs a position.
- `Ship.sobj_id == 0` is a sentinel, not a foreign key, while docked. Code must not resolve it.
- Undock re-mints the sobj, so sobj ids are **not stable across dock cycles**; nothing may persist a sobj id as a long-lived ship reference — `ShipId` is the durable identity.
- Canonical vocabulary (Piloted Ship / Docked Ship) lives in `CONTEXT.md` §3; `get_player_ship` should eventually be renamed `get_piloted_ship` to make the contract self-evident.
