# Denormalize `sector_id` across `Ship`, `ShipStatus`, and `StellarObject`

A ship's sector membership is stored in three places: `Ship.sector_id`, `ShipStatus.sector_id`, and `StellarObject.sector_id`. They must be kept in sync on every cross-sector event (jumpgate transit, undock, debug teleport); the `transit_ship_to_sector` helper in `logic/stellarobjects/movement.rs` is the single enforcement point.

This duplication is **deliberate, not a normalization mistake**. SpacetimeDB's RLS filter SQL is not full SQL — it has a hard cap on join count, well below what a normalised three-hop sector lookup would require. Views (when they land per follow-up #1) are more permissive but still pay a real per-row cost per join. Storing `sector_id` directly on each table lets every RLS filter and view that scopes by sector be a single-row predicate (`WHERE row.sector_id == this_player.current_sector_id`) with zero joins.

## Considered alternatives

- **Single source of truth on `StellarObject.sector_id`.** Rejected: forces RLS/view filters to join `ShipStatus → StellarObject` (or `Ship → StellarObject`), which exceeds the RLS join budget for any non-trivial filter chain and degrades view performance.
- **`Ship.sector_id` only (drop the other two).** Rejected for the same reason — `ShipStatus` filters that need sector scoping would have to join through `Ship`. The `ShipStatus.sector_id` index is load-bearing for the per-sector status enumeration path.

## Consequences

- Every reducer that moves a ship across sectors **must** go through `transit_ship_to_sector` — open-coding three (or more) `set` calls is a footgun that will silently desync the duplicates. The plan's Phase 3 establishes this helper as the only sanctioned cross-sector mutator.
- Write amplification is real (3 row writes per sector change) but rare — sector changes happen on jumpgate, undock, and debug paths, not on the hot per-frame movement path.
- If RLS gains join-count headroom or views become free-join in a future SpacetimeDB release, this ADR should be revisited.
