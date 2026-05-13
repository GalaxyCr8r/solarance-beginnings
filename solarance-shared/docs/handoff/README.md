# Handoff: Movement Prototype → Solarance: Beginnings

This folder documents the concepts proven in `solarance-movement-prototype` that should be ported into the main Solarance repo. The three pillars are:

| Document | What it covers |
|---|---|
| [dead-reckoning-movement.md](dead-reckoning-movement.md) | The `MovementState` / `predict_movement` physics model and how reducers keep it in sync |
| [transform-table-migration.md](transform-table-migration.md) | Concrete plan for collapsing `StellarObject` + four transform tables into one dead-reckoning field per entity |
| [view-functions-visibility.md](view-functions-visibility.md) | SpacetimeDB `#[view]` functions for per-player visibility scoping |

## Why this matters

The original Solarance repo sends an exact position every tick via `sobj_hi_res_transform` / `sobj_low_res_transform`. This means:

- Bandwidth grows linearly with players per sector.
- The server has to run a game loop that writes new positions every frame.
- Clients have no way to interpolate between server ticks — they just render the last known position.

The prototype proves that you can instead:

1. Store a physics snapshot in the DB only when **inputs change** (not every frame).
2. Run identical deterministic physics on both server and client — `predict_movement` from `solarance-shared`.
3. Gate what snapshot data each client receives using **view functions** keyed on their sector.

The result is that position updates drop from tens-per-second to a handful per input event, view functions ensure clients only subscribe to ships in their sector, and the server never needs to run a game-loop tick at all for movement.
