use log::info;
use solarance_shared::Vec2;
use spacetimedb::ReducerContext;
use crate::spacetimedsl::prelude::*;

use crate::logic::stellarobjects::movement::transit_ship_to_sector;
use crate::tables::sectors::SectorId;
use crate::tables::ships::*;
use crate::utility::try_server_only;

/// Admin: teleport a ship to `target_sector_id`, arriving at the sector origin
/// with zeroed dynamics.
///
/// Reuses `transit_ship_to_sector` — the same atomic set of writes
/// `try_to_use_jumpgate` performs (Ship / ShipStatus / StellarObject `sector_id`
/// + a clean-stop movement snapshot), minus the gate range / energy gates. That
/// keeps the "cross-sector transit" invariant in one deep helper so the admin
/// path can't drift from the player path.
#[spacetimedb::reducer]
pub fn admin_teleport_ship_to_sector(
    ctx: &ReducerContext,
    ship_id: u64,
    target_sector_id: u64,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    let ship = dsl.get_ship_by_id(ShipId::new(ship_id))?;

    // A docked ship has no StellarObject (sobj_id == 0), so transit would fail
    // on the sobj lookup. Require it be in-sector so the teleport is well-defined.
    if *ship.get_location() != ShipLocation::Sector {
        return Err(format!(
            "admin_teleport_ship_to_sector: ship #{} is not in a sector (location {:?}); undock it first",
            ship_id,
            ship.get_location()
        ));
    }

    let destination = dsl
        .get_sector_by_id(&SectorId::new(target_sector_id))
        .map_err(|_| {
            format!("admin_teleport_ship_to_sector: target sector #{} not found", target_sector_id)
        })?;

    info!(
        "admin_teleport_ship_to_sector: caller={} ship #{} -> sector #{} ({})",
        ctx.sender().to_abbreviated_hex(),
        ship_id,
        destination.get_id().value(),
        destination.get_name(),
    );

    // ponytail: arrival at sector origin (0,0), no rotation — the "send ship to
    // sector" use case doesn't need a position picker. Add args if it ever does.
    transit_ship_to_sector(&dsl, &ship.get_id(), &destination.get_id(), Vec2::ZERO, 0.0)
}
