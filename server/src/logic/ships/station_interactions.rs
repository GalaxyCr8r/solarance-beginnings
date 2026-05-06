use log::info;
use spacetimedb::{Identity, ReducerContext};
use spacetimedsl::*;

use crate::{
    logic::{
        ships::{movement_controllers::initialize_controller_for_player, status::*},
        stellarobjects::{player_windows::*, stellar_object_creation::*},
    },
    tables::{
        common_types::Vec2,
        jumpgates::JumpGate,
        players::{get_player_ship_and_sobj, PlayerId},
        sectors::GetSectorRowOptionById,
        server_messages::send_info_message,
        ships::*,
        stations::*,
        stellarobjects::*,
    },
    utility::is_server_or_ship_owner,
};

///////////////////////////////////////////////////////////////////////////////////
///  Reducers

/// Tries to dock to station using the player's current ship.
pub fn try_to_dock_to_station(ctx: &ReducerContext, station: &Station) -> Result<(), String> {
    let dsl = dsl(ctx);
    let (ship_object, ship_sobj) = get_player_ship_and_sobj(&dsl, &PlayerId::new(ctx.sender()))?;

    // TODO: Check if same faction

    // TODO: If not, check faction standing

    info!("Trying to dock to station!");
    dock_to_station(&dsl, &ship_object, &ship_sobj, station)?;

    Ok(())
}

/// Used by a player client.
/// Requests to undock the given Ship on top of the station it was docked at and returns the new Ship row.
#[spacetimedb::reducer]
pub fn undock_ship(ctx: &ReducerContext, ship: Ship) -> Result<(), String> {
    let dsl = dsl(ctx);
    is_server_or_ship_owner(&dsl, Some(ship.get_id().clone()))?;

    // Exit early if the player is already controlling a ship
    if dsl
        .get_sobj_player_window_by_id(PlayerId::new(ctx.sender()))
        .is_ok()
    {
        return Err(
            "Player requested to undock another ship, but they are already controlling one!"
                .to_string(),
        );
    }

    if *ship.get_location() == ShipLocation::Station {
        undock_from_station(&dsl, &ship)?;
    } else {
        info!(
            "Ship {} attempting to undock is already undocked!",
            ship.get_id()
        );
    }

    Ok(())
}

pub fn try_to_use_jumpgate(ctx: &ReducerContext, jumpgate: &JumpGate) -> Result<(), String> {
    let dsl = dsl(ctx);
    let (mut ship_object, _) = get_player_ship_and_sobj(&dsl, &PlayerId::new(ctx.sender()))?;
    let mut ship_status = dsl.get_ship_status_by_id(ship_object.get_id())?;

    // Jump once they have more than 100 energy
    if *ship_status.get_energy() > 100.0 {
        let pos: &Vec2 = jumpgate.get_target_gate_arrival_pos();
        let destination_sector = dsl.get_sector_by_id(jumpgate.get_target_sector_id())?;

        ship_status.set_energy(ship_status.get_energy() - 100.0);
        ship_status.set_sector_id(&destination_sector);

        ship_object.set_sector_id(&destination_sector);

        if let Ok(mut sobj) = dsl.get_stellar_object_by_id(&ship_object.get_sobj_id()) {
            sobj.set_sector_id(&destination_sector);
            if let Ok(mut transform) = dsl.get_sobj_internal_transform_by_id(&sobj.get_id()) {
                transform.set_x(pos.x);
                transform.set_y(pos.y);
                dsl.update_sobj_internal_transform_by_id(transform)?;
            }

            dsl.update_stellar_object_by_id(sobj)?;
        }

        send_info_message(
            &dsl,
            &ship_object.get_player_id(),
            format!(
                "Jumped successfully via jumpgate to sector #{}: {}",
                destination_sector.get_id().value(),
                destination_sector.get_name()
            ),
            Some("status"),
        )?;

        dsl.update_ship_status_by_id(ship_status)?;
        dsl.update_ship_by_id(ship_object)?;
    }

    Ok(())
} // try_to_use_jumpgate

/////////////////////////////////////////////////////////////////////////////
///  Utilities

/// Creates the Ship object plus removes the Ship and StellarObject but keeps the cargo, health, etc.
pub fn dock_to_station<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    ship: &Ship,
    ship_sobj: &StellarObject,
    station: &Station,
) -> Result<Ship, String> {
    // Remove the ship's StellarObject
    let _ = dsl.delete_stellar_object_by_id(ship_sobj); // Should this error really be suppressed?

    // Create Ship object
    let docked = &mut ship.clone();
    docked.set_sobj_id(StellarObjectId::new(0));
    docked.set_station_id(station.get_id());
    docked.set_location(ShipLocation::Station);
    info!("Updating docked ship's station and location");
    let _ = dsl.update_ship_by_id(docked.clone())?;

    send_info_message(
        dsl,
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

pub fn undock_from_station<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    docked: &Ship,
) -> Result<Ship, String> {
    let station = dsl.get_station_by_id(docked.get_station_id())?;
    let station_transform = dsl.get_sobj_internal_transform_by_id(station.get_sobj_id())?;
    let ship_type = dsl.get_ship_type_definition_by_id(docked.get_shiptype_id())?;

    let sobj = create_sobj_internal(
        dsl,
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
        let _ = create_status_timer_for_ship(dsl, &ship.get_id(), &ship_type.get_id());
        // Should this error really be suppressed?
    }

    if docked.get_player_id().value() != Identity::ONE {
        // There is a real player controlling this ship, so create the necessary helpers.
        create_sobj_player_window_from_dsl(dsl, docked.get_player_id().value(), sobj.get_id())?;
        let _ = initialize_controller_for_player(dsl, &docked.get_player_id(), &sobj);
    // Should this error really be suppressed?
    } else {
        // There is NOT a real player controllering this ship, so error for now.
        return Err("Unsupported: There was an attempt to undock an NPC ship!".to_string());
    }

    send_info_message(
        dsl,
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
