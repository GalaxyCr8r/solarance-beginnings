use log::info;
use spacetimedb::ReducerContext;
use crate::spacetimedsl::prelude::*;

use crate::logic::ships::station_interactions::{dock_to_station, undock_from_station};
use crate::tables::stations::StationId;
use crate::tables::ships::*;
use crate::utility::try_server_only;

/// Admin: force-dock a ship at a station, skipping the range check the player
/// path enforces.
///
/// Reuses `dock_to_station` — the same writes `try_to_dock_to_station` performs
/// (clean-stop movement snapshot, StellarObject deletion, Ship location/station
/// update, owner notification), minus the docking range and under-construction
/// gates. Keeps the "docked ship" invariant in one deep helper so the admin path
/// can't drift from the player path.
#[spacetimedb::reducer]
pub fn admin_dock_ship(
    ctx: &ReducerContext,
    ship_id: u64,
    station_id: u64,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    let ship = dsl.get_ship_by_id(ShipId::new(ship_id))?;

    if *ship.get_location() != ShipLocation::Sector {
        return Err(format!(
            "admin_dock_ship: ship #{} is not in a sector (location {:?}); only in-sector ships can dock",
            ship_id,
            ship.get_location()
        ));
    }

    let station = dsl
        .get_station_by_id(&StationId::new(station_id))
        .map_err(|_| format!("admin_dock_ship: station #{} not found", station_id))?;

    // `dock_to_station` writes `station_id` / `location` but not `sector_id`, so a
    // cross-sector dock would leave the ship's sector stale until it undocks.
    // Reject instead — `admin_teleport_ship_to_sector` is the fix path.
    if ship.get_sector_id().value() != station.get_sector_id().value() {
        return Err(format!(
            "admin_dock_ship: ship #{} is in sector {} but station #{} is in sector {}; teleport it there first",
            ship_id,
            ship.get_sector_id().value(),
            station_id,
            station.get_sector_id().value()
        ));
    }

    let ship_sobj = dsl
        .get_stellar_object_by_id(&ship.get_sobj_id())
        .map_err(|_| {
            format!(
                "admin_dock_ship: ship #{} claims sobj #{} but no such StellarObject exists",
                ship_id,
                ship.get_sobj_id().value()
            )
        })?;

    info!(
        "admin_dock_ship: caller={} ship #{} -> station #{} ({})",
        ctx.sender().to_abbreviated_hex(),
        ship_id,
        station_id,
        station.get_name(),
    );

    dock_to_station(&dsl, &ship, &ship_sobj, &station)?;
    Ok(())
}

/// Admin: force-undock a ship, placing it in space at its station's pose.
///
/// Reuses `undock_from_station` — the same writes the player path performs
/// (fresh StellarObject, Ship location/sector update, stopped movement snapshot
/// at the station pose, status timer + movement controller, owner notification).
#[spacetimedb::reducer]
pub fn admin_undock_ship(ctx: &ReducerContext, ship_id: u64) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    let ship = dsl.get_ship_by_id(ShipId::new(ship_id))?;

    if *ship.get_location() != ShipLocation::Station {
        return Err(format!(
            "admin_undock_ship: ship #{} is not docked (location {:?}); nothing to undock",
            ship_id,
            ship.get_location()
        ));
    }

    // Same guard the player-facing `undock_ship` enforces. `undock_from_station`
    // itself calls `initialize_controller_for_player` unconditionally, so
    // skipping this would hand the owner a second movement controller.
    let owner = ship.get_player_id();
    if dsl
        .get_ships_by_player_id(&owner)
        .any(|s| *s.get_location() == ShipLocation::Sector)
    {
        return Err(format!(
            "admin_undock_ship: ship #{} not undocked — player {} is already controlling an in-sector ship",
            ship_id,
            owner.value().to_abbreviated_hex()
        ));
    }

    info!(
        "admin_undock_ship: caller={} ship #{} undocking from station #{}",
        ctx.sender().to_abbreviated_hex(),
        ship_id,
        ship.get_station_id().value(),
    );

    undock_from_station(&dsl, &ship)?;
    Ok(())
}
