use crate::types::{
    common::utility::*, server_messages::utility::send_info_message, ships::utility::*,
    stellarobjects::*,
};

use super::*;

///////////////////////////////////////////////////////////
// Reducers ///
///////////////////////////////////////////////////////////

/// Allows a player to jettison cargo from their ship into space as a cargo crate.
/// Validates ship ownership and cargo availability before creating the crate.
#[spacetimedb::reducer]
pub fn jettison_cargo_from_ship(
    ctx: &ReducerContext,
    ship_id: u64,
    ship_cargo_id: u64,
    amount: u16,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let ship = dsl.get_ship_by_id(ShipId::new(ship_id))?;

    is_server_or_sobj_owner(ctx, Some(ship.get_sobj_id()))?;

    let mut ship_cargo = dsl.get_ship_cargo_item_by_id(ShipCargoItemId::new(ship_cargo_id))?;
    let item_def = dsl.get_item_definition_by_id(ship_cargo.get_item_id())?;

    // Does the ship actually have that amount of item?
    if ship_cargo.get_quantity() < &amount {
        return Err(format!(
            "Failed to verify that the cargo item actually had the amount requested to yeet."
        ));
    } else if ship_cargo.get_quantity() == &amount {
        dsl.delete_ship_cargo_item_by_id(&ship_cargo)?;
    } else {
        ship_cargo.quantity -= amount;
        dsl.update_ship_cargo_item_by_id(ship_cargo)?;
    }

    create_cargo_crate_nearby_ship(ctx, &ship.get_sobj_id(), &item_def, amount)?;

    send_info_message(
        ctx,
        &ship.get_player_id(),
        format!("Jettioned successfully {}x {}", amount, item_def.get_name(),),
        Some("status"),
    )?;

    Ok(())
}

/// Teleports a ship to a specific sector by ID (convenience wrapper).
/// This is a hard teleport that bypasses jump gates and jump drives.
#[spacetimedb::reducer]
pub fn teleport_to_sector_ids(
    ctx: &ReducerContext,
    ship_id: u64,
    destination_sector_id: u64,
) -> Result<(), String> {
    let s_id = ShipId::new(ship_id);
    try_server_only(ctx)?;

    teleport_to_sector(
        ctx,
        dsl(ctx).get_ship_by_id(s_id)?,
        Sector::get(ctx, &SectorId::new(destination_sector_id))?,
        0.0,
        0.0,
    )
}

/// Hard shift to target sector. This does not take into account jump gate or jump drives.
#[spacetimedb::reducer]
pub fn teleport_to_sector(
    ctx: &ReducerContext,
    mut ship: Ship,
    destination_sector: Sector,
    x: f32,
    y: f32,
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    ship.set_sector_id(&destination_sector);
    if let Ok(mut sobj) = dsl.get_stellar_object_by_id(&ship.get_sobj_id()) {
        sobj.set_sector_id(&destination_sector);
        if let Ok(mut transform) = dsl.get_sobj_internal_transform_by_id(&sobj.get_id()) {
            transform.set_x(x);
            transform.set_y(y);
            dsl.update_sobj_internal_transform_by_id(transform)?;
        }
        dsl.update_stellar_object_by_id(sobj)?;
    }
    if let Ok(mut ship_status) = dsl.get_ship_status_by_id(&ship.get_id()) {
        ship_status.set_sector_id(&destination_sector);
        dsl.update_ship_status_by_id(ship_status)?;
    }

    send_info_message(
        ctx,
        &ship.get_player_id(),
        format!(
            "Jumped successfully via jumpgate to sector #{}: {}",
            destination_sector.get_id().value(),
            destination_sector.get_name(),
        ),
        Some("status"),
    )?;

    dsl.update_ship_by_id(ship)?;

    Ok(())
}

// /// Docks the given Ship to the given station it is docking at and returns the new Ship row.
// #[spacetimedb::reducer]
// pub fn dock_ship(
//     ctx: &ReducerContext,
//     docking_ship: Ship,
//     station: StationId,
// ) -> Result<(), String> {
//     is_server_or_ship_owner(ctx, Some(docking_ship));

//     todo!() // I don't think this is something a client can directly request? We have `dock` as a flag in the player controller.
// }

/// Undocks the given Ship on top of the station it was docked at and returns the new Ship row.
#[spacetimedb::reducer]
pub fn undock_ship(ctx: &ReducerContext, ship: Ship) -> Result<(), String> {
    is_server_or_ship_owner(ctx, Some(ship.get_id().clone()))?;
    //let dsl = dsl(ctx);

    if *ship.get_location() == ShipLocation::Station {
        undock_from_station(ctx, ship)?;
    } else {
        info!(
            "Ship {} attempting to undock is already undocked!",
            ship.get_id()
        );
    }

    Ok(())
}
