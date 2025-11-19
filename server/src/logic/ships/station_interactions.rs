use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::{
    logic::ships::player_controller::initialize_player_controller,
    tables::{
        factions::FactionId,
        players::PlayerId,
        server_messages::utility::send_info_message,
        ships::{timers::*, *},
        stations::*,
        stellarobjects::{reducers::create_sobj_player_window_for, utility::*, *},
    },
    utility::is_server_or_ship_owner,
};

///////////////////////////////////////////////////////////////////////////////////
///  Reducers

/// Used by a player client.
/// Requests to undock the given Ship on top of the station it was docked at and returns the new Ship row.
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

/////////////////////////////////////////////////////////////////////////////
///  Utilities

/// Creates the Ship object plus removes the Ship and StellarObject but keeps the cargo, health, etc.
pub fn dock_to_station(
    ctx: &ReducerContext,
    ship: &Ship,
    ship_sobj: &StellarObject,
    station: &Station,
) -> Result<Ship, String> {
    let dsl = dsl(ctx);

    // Remove the ship's StellarObject
    let _ = dsl.delete_stellar_object_by_id(ship_sobj);

    // Create Ship object
    let docked = &mut ship.clone();
    docked.set_sobj_id(StellarObjectId::new(0));
    docked.set_station_id(station.get_id());
    docked.set_location(ShipLocation::Station);
    info!("Updating docked ship's station and location");
    let _ = dsl.update_ship_by_id(docked.clone())?;

    send_info_message(
        ctx,
        &ship.get_player_id(),
        format!(
            "Docked successfully with Station #{}: {}",
            station.get_id().value(),
            station.get_name()
        ),
        Some("status"),
    )?;

    Ok(docked.clone())
}

pub fn undock_from_station(ctx: &ReducerContext, docked: &Ship) -> Result<Ship, String> {
    let dsl = dsl(ctx);

    let station = dsl.get_station_by_id(docked.get_station_id())?;
    let station_transform = dsl.get_sobj_internal_transform_by_id(station.get_sobj_id())?;
    let ship_type = dsl.get_ship_type_definition_by_id(docked.get_shiptype_id())?;

    let sobj = create_sobj_internal(
        ctx,
        StellarObjectKinds::Ship,
        &station.get_sector_id(),
        station_transform,
    )?;

    let ship = &mut docked.clone();
    ship.set_sobj_id(&sobj);
    ship.set_sector_id(station.get_sector_id());
    ship.set_station_id(StationId::new(0));
    ship.set_location(ShipLocation::Sector);
    dsl.update_ship_by_id(ship.clone())?;

    // Ensure there's still a ship status timer.
    if dsl
        .get_ship_status_timer_by_ship_id(docked.get_id())
        .is_err()
    {
        let _ = create_status_timer_for_ship(ctx, &ship.get_id(), &ship_type.get_id());
    }

    if docked.get_player_id().value() != Identity::ONE {
        // There is a real player controlling this ship, so create the necessary helpers.
        let _ = create_sobj_player_window_for(ctx, docked.get_player_id().value(), sobj.get_id())?;
        let _ = initialize_player_controller(ctx, &docked.get_player_id(), &sobj);
    } else {
        // There is NOT a real player controllering this ship, so error for now.
        return Err("Unsupported: There was an attempt to undock an NPC ship!".to_string());
    }

    send_info_message(
        ctx,
        &ship.get_player_id(),
        format!(
            "Undocked successfully with Station #{}: {}",
            station.get_id().value(),
            station.get_name()
        ),
        Some("status"),
    )?;

    Ok(ship.clone())
}
