
use crate::types::{
    jumpgates::*,
    ships::utility::*,
    stellarobjects::*,
    common::utility::*,
};

use super::*;

///////////////////////////////////////////////////////////
// Reducers ///
///////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn jettison_cargo_from_ship(
    ctx: &ReducerContext,
    ship_id: u64,
    ship_cargo_id: u64,
    amount: u16
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let ship = dsl.get_ship_by_id(ShipGlobalId::new(ship_id)).ok_or(
        format!("Failed to find ship")
    )?;

    is_server_or_sobj_owner(ctx, Some(ship.get_sobj_id()))?;

    let mut ship_cargo = dsl.get_ship_cargo_item_by_id(ShipCargoItemId::new(ship_cargo_id)).ok_or(
        format!("Failed to find cargo item")
    )?;
    let item_def = dsl.get_item_definition_by_id(ship_cargo.get_item_id()).ok_or(
        format!("Failed to find item definition for cargo")
    )?;

    // Does the ship actually have that amount of item?
    if ship_cargo.get_quantity() < &amount {
        return Err(format!("Failed to verify that the cargo item actually had the amount requested to yeet."));
    } else if ship_cargo.get_quantity() == &amount {
        dsl.delete_ship_cargo_item_by_id(&ship_cargo);
    } else {
        ship_cargo.quantity -= amount;
        dsl.update_ship_cargo_item_by_id(ship_cargo)?;
    }

    create_cargo_crate_nearby_ship(ctx, &ship.get_sobj_id(), &item_def, amount)?;

    Ok(())
}

#[spacetimedb::reducer]
pub fn teleport_to_sector_ids(
    ctx: &ReducerContext,
    ship_id: u64,
    destination_sector_id: u64
) -> Result<(), String> {
    let s_id = ShipGlobalId::new(ship_id);
    teleport_to_sector(
        ctx,
        dsl(ctx).get_ship_by_id(s_id).ok_or(
            "Failed to teleport to sector, couldn't find ship instance."
        )?,
        Sector::get(ctx, &SectorId::new(destination_sector_id)).ok_or(
            "Failed to teleport to sector, couldn't find sector."
        )?,
        0., 0.
    )
}

/// Hard shift to target sector. This does not take into account jump gate or jump drives.
#[spacetimedb::reducer]
pub fn teleport_to_sector(
    ctx: &ReducerContext,
    mut ship: Ship,
    destination_sector: Sector,
    x: f32, y: f32
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    ship.set_sector_id(&destination_sector);
    if let Some(mut sobj) = dsl.get_stellar_object_by_id(&ship.get_sobj_id()) {
        sobj.set_sector_id(&destination_sector);
        if let Some(mut transform) = StellarObjectTransformInternal::get(ctx, &sobj.get_id()) {
            transform.set_x(x);
            transform.set_y(y);
            dsl.update_sobj_internal_transform_by_sobj_id(transform)?;
        }
        dsl.update_stellar_object_by_id(sobj)?;
    }
    if let Some(mut ship_status) = dsl.get_ship_status_by_id(&ship.get_id()) {
        ship_status.set_sector_id(&destination_sector);
        dsl.update_ship_status_by_id(ship_status)?;
    }
    dsl.update_ship_by_id(ship)?;

    Ok(())
}

pub fn teleport_via_jumpgate(
    ctx: &ReducerContext,
    ship: Ship,
    jumpgate: &JumpGate
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    let pos: &crate::types::common::Vec2 = jumpgate.get_target_gate_arrival_pos();
    teleport_to_sector(ctx, ship,
        dsl.get_sector_by_id(jumpgate.get_target_sector_id()).ok_or("Failed to find jumpgate's target sector.")?,
        pos.x, pos.y)
}
