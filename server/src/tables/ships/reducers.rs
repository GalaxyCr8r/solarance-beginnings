use crate::tables::{
    players::PlayerId, server_messages::utility::send_info_message, ships::utility::*,
    stellarobjects::*,
};
use crate::utility::*;

use super::*;

///////////////////////////////////////////////////////////
// Reducers ///
///////////////////////////////////////////////////////////

/// Teleports a ship to a specific sector by ID (convenience wrapper).
/// This is a hard teleport that bypasses jump gates and jump drives.
// #[spacetimedb::reducer]
// pub fn teleport_to_sector_ids(
//     ctx: &ReducerContext,
//     ship_id: u64,
//     destination_sector_id: u64,
// ) -> Result<(), String> {
//     let s_id = ShipId::new(ship_id);
//     try_server_only(ctx)?;

//     teleport_to_sector(
//         ctx,
//         dsl(ctx).get_ship_by_id(s_id)?,
//         Sector::get(ctx, &SectorId::new(destination_sector_id))?,
//         0.0,
//         0.0,
//     )
// }

/// Undocks the given Ship on top of the station it was docked at and returns the new Ship row.
#[spacetimedb::reducer]
pub fn undock_ship(ctx: &ReducerContext, ship: Ship) -> Result<(), String> {
    is_server_or_ship_owner(ctx, Some(ship.get_id().clone()))?;
    let dsl = dsl(ctx);

    // Exit early if the player is already controlling a ship
    if dsl
        .get_sobj_player_window_by_id(PlayerId::new(ctx.sender))
        .is_ok()
    {
        return Err(
            "Player requested to undock another ship, but they are already controlling one!"
                .to_string(),
        );
    }

    if *ship.get_location() == ShipLocation::Station {
        undock_from_station(ctx, &ship)?;
    } else {
        info!(
            "Ship {} attempting to undock is already undocked!",
            ship.get_id()
        );
    }

    Ok(())
}
