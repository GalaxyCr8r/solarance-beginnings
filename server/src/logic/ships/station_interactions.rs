use log::info;
use spacetimedb::ReducerContext;
use crate::spacetimedsl::prelude::*;

use crate::{
    logic::{
        ships::{movement_controllers::initialize_controller_for_player, status::*},
        stellarobjects::{
            movement::{transit_ship_to_sector, write_ship_movement_snapshot},
            stellar_object_creation::create_sobj,
        },
    },
    tables::{
        jumpgates::*,
        players::{get_player_ship_and_sobj, PlayerId},
        sectors::GetSectorRowOptionById,
        messages::{send_direct_server_warning, send_direct_server_info},
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

    // Reject docking with an under-construction site. The row only exists for
    // stations that started life as construction sites; `is_operational` flips
    // to true on completion, so a missing row also implies "operational".
    if let Ok(under_construction) = dsl.get_station_under_construction_by_id(&station.get_id()) {
        if !*under_construction.get_is_operational() {
            let msg = format!(
                "Cannot dock at '{}' (station #{}): still under construction.",
                station.get_name(),
                station.get_id().value()
            );
            let _ = send_direct_server_warning(
                &dsl,
                &PlayerId::new(ctx.sender()),
                msg.clone(),
            );
            return Err(msg);
        }
    }

    // TODO: Check if same faction

    // TODO: If not, check faction standing

    info!("Trying to dock to station!");
    dock_to_station(&dsl, &ship_object, &ship_sobj, station)?;

    Ok(())
}

/// Used by a player client.
/// Requests to dock the player's current ship at the targeted station.
/// Validates the target is a station and the ship is within docking range.
#[spacetimedb::reducer]
pub fn dock_ship(ctx: &ReducerContext, target_sobj_id: u64) -> Result<(), String> {
    let dsl = dsl(ctx);
    let station = dsl
        .get_station_by_sobj_id(&StellarObjectId::new(target_sobj_id))
        .map_err(|_| "Target is not a station".to_string())?;

    let (ship_object, _) = get_player_ship_and_sobj(&dsl, &PlayerId::new(ctx.sender()))?;
    // Predicted-forward ship position; static station position.
    let ship_snapshot =
        crate::logic::stellarobjects::movement::get_ship_movement_snapshot(&dsl, &ship_object.get_id())?;
    let dist = ship_snapshot.pos.distance_to(station.get_position());

    const DOCK_RANGE: f32 = 500.0;
    if dist > DOCK_RANGE {
        return Err(format!("Too far to dock ({dist:.0} > {DOCK_RANGE})"));
    }

    try_to_dock_to_station(ctx, &station)
}

/// Used by a player client.
/// Requests to undock the given Ship on top of the station it was docked at and returns the new Ship row.
#[spacetimedb::reducer]
pub fn undock_ship(ctx: &ReducerContext, ship: Ship) -> Result<(), String> {
    let dsl = dsl(ctx);
    is_server_or_ship_owner(&dsl, Some(ship.get_id().clone()))?;

    // Exit early if the player is already controlling an in-sector ship.
    let player_id = PlayerId::new(ctx.sender());
    let already_in_sector = dsl
        .get_ships_by_player_id(&player_id)
        .any(|s| *s.get_location() == ShipLocation::Sector);
    if already_in_sector {
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

/// Used by a player client. Looks up the targeted jumpgate by its sobj id
/// and routes through `try_to_use_jumpgate` (which handles the energy gate
/// + cross-sector transit). Distance / faction gating happens server-side.
#[spacetimedb::reducer]
pub fn use_jumpgate(ctx: &ReducerContext, jumpgate_sobj_id: u64) -> Result<(), String> {
    let dsl = dsl(ctx);
    let jumpgate = dsl
        .get_jump_gate_by_id(&StellarObjectId::new(jumpgate_sobj_id))
        .map_err(|_| format!("No jumpgate at sobj #{}", jumpgate_sobj_id))?;
    try_to_use_jumpgate(ctx, &jumpgate)
}

/// Hard proximity check for jumpgate activation. Same units as ship pos
/// (pixels). Matches the dock range in spirit — you have to fly up to the
/// gate before it'll fire.
const JUMPGATE_USE_RANGE: f32 = 300.0;
const JUMPGATE_USE_ENERGY: f32 = 50.0;

pub fn try_to_use_jumpgate(ctx: &ReducerContext, jumpgate: &JumpGate) -> Result<(), String> {
    let dsl = dsl(ctx);
    let (ship_object, _) = get_player_ship_and_sobj(&dsl, &PlayerId::new(ctx.sender()))?;
    let mut ship_status = dsl.get_ship_status_by_id(ship_object.get_id())?;

    // Predicted-forward ship pos vs. the gate's static position. Reject if
    // the ship is too far away — this used to be missing, so a player on
    // the far side of the sector could still trigger the jump.
    let ship_snapshot = crate::logic::stellarobjects::movement::get_ship_movement_snapshot(
        &dsl,
        &ship_object.get_id(),
    )?;
    let dist = ship_snapshot.pos.distance_to(jumpgate.get_position());
    if dist > JUMPGATE_USE_RANGE {
        return Err(format!(
            "Too far to use jumpgate #{} ({dist:.0} > {JUMPGATE_USE_RANGE})",
            jumpgate.get_id().value()
        ));
    }

    // Jump once they have more than JUMPGATE_USE_ENERGY energy
    if *ship_status.get_energy() > JUMPGATE_USE_ENERGY {
        let arrival_pos = *jumpgate.get_target_gate_arrival_pos();
        let arrival_rotation = *jumpgate.get_target_gate_arrival_rotation();
        let destination_sector = dsl.get_sector_by_id(jumpgate.get_target_sector_id())?;

        ship_status.set_energy(ship_status.get_energy() - JUMPGATE_USE_ENERGY);
        dsl.update_ship_status_by_id(ship_status)?;

        // Single helper does all the sector_id updates + clean-stop snapshot
        // so a partial failure can't leave the ship half-transitioned.
        transit_ship_to_sector(
            &dsl,
            &ship_object.get_id(),
            &destination_sector.get_id(),
            arrival_pos,
            arrival_rotation,
        )?;

        send_direct_server_info(
            &dsl,
            &ship_object.get_player_id(),
            format!(
                "Jumped successfully via jumpgate to sector #{}: {}",
                destination_sector.get_id().value(),
                destination_sector.get_name()
            ),
        )?;
    } else {
        // TODO: Send a message to the player saying they don't have JUMPGATE_USE_ENERGY
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
    // Zero the dead-reckoning snapshot first so any client reading
    // Ship.movement after docking sees a stopped ship rather than continuing
    // to extrapolate from pre-dock velocity.
    write_ship_movement_snapshot(dsl, &ship.get_id(), |state| {
        state.velocity = 0.0;
        state.angular_velocity = 0.0;
        state.acceleration = 0.0;
        state.angular_acceleration = 0.0;
    })?;

    // Remove the ship's StellarObject
    let _ = dsl.delete_stellar_object_by_id(ship_sobj); // Should this error really be suppressed?

    // Create Ship object
    let docked = &mut ship.clone();
    docked.set_sobj_id(StellarObjectId::new(0));
    docked.set_station_id(station.get_id());
    docked.set_location(ShipLocation::Station);
    info!("Updating docked ship's station and location");
    let _ = dsl.update_ship_by_id(docked.clone())?;

    send_direct_server_info(
        dsl,
        &ship.get_player_id(),
        format!(
            "Docked successfully with Station #{}: {}",
            station.get_id().value(),
            station.get_name()
        ),
    )?;

    Ok(docked.clone())
}

pub fn undock_from_station<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    docked: &Ship,
) -> Result<Ship, String> {
    let station = dsl.get_station_by_id(docked.get_station_id())?;
    let ship_type = dsl.get_ship_type_definition_by_id(docked.get_shiptype_id())?;

    let sobj = create_sobj(dsl, StellarObjectKinds::Ship, &station.get_sector_id())?;

    let ship = &mut docked.clone();
    ship.set_sobj_id(&sobj);
    ship.set_sector_id(station.get_sector_id());
    ship.set_station_id(StationId::new(0));
    ship.set_location(ShipLocation::Sector);
    dsl.update_ship_by_id(ship.clone())?;

    // Stamp a fresh stopped snapshot at the station's pose. Caps get
    // re-stamped from ShipTypeDefinition by the helper.
    let station_pos = *station.get_position();
    let station_rotation = *station.get_rotation();
    write_ship_movement_snapshot(dsl, &ship.get_id(), |state| {
        state.pos = station_pos;
        state.rotation = station_rotation;
        state.velocity = 0.0;
        state.angular_velocity = 0.0;
        state.acceleration = 0.0;
        state.angular_acceleration = 0.0;
    })?;

    // Ensure there's still a ship status timer.
    if dsl
        .get_ship_status_timer_by_ship_id(docked.get_id())
        .is_err()
    {
        let _ = create_status_timer_for_ship(dsl, &ship.get_id(), &ship_type.get_id());
    }

    let _ = initialize_controller_for_player(dsl, &docked.get_player_id(), &sobj);

    send_direct_server_info(
        dsl,
        &ship.get_player_id(),
        format!(
            "Undocked successfully with Station #{}: {}",
            station.get_id().value(),
            station.get_name()
        ),
    )?;

    Ok(ship.clone())
}
