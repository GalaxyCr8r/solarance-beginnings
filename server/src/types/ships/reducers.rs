use glam::Vec2;

use crate::types::{
    items::*,
    jumpgates::*,
    sectors::*,
    ships::utility::*,
    stellarobjects::*,
    utility::*,
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
    info!("Yo bruh we got the dets {}, {}, for x{}!", ship_id, ship_cargo_id, amount);

    let dsl = dsl(ctx);
    let ship_object = dsl.get_ship_object_by_ship_id(ShipInstanceId::new(ship_id)).ok_or(
        format!("Blah1") // TODO error message here
    )?;

    is_server_or_sobj_owner(ctx, ship_object.get_sobj_id())?;

    let mut ship_cargo = dsl.get_ship_cargo_item_by_id(ShipCargoItemId::new(ship_cargo_id)).ok_or(
        format!("Blah2") // TODO error message here
    )?;
    let item_def = dsl.get_item_definition_by_id(ship_cargo.get_item_id()).ok_or(
        format!("Blah3") // TODO error message here
    )?;

    // Does the ship actually have that amount of item?
    if ship_cargo.get_quantity() < &amount {
        return Err(format!("Blah4")); // TODO error message here
    } else if ship_cargo.get_quantity() == &amount {
        dsl.delete_ship_cargo_item_by_id(&ship_cargo);
    } else {
        ship_cargo.quantity -= amount;
        dsl.update_ship_cargo_item_by_id(ship_cargo)?;
    }

    create_cargo_crate_nearby_ship(ctx, &ship_object.get_sobj_id(), &item_def, amount)?;

    Ok(())
}

#[spacetimedb::reducer]
pub fn teleport_to_sector_ids(
    ctx: &ReducerContext,
    ship_id: u64,
    destination_sector_id: u64
) -> Result<(), String> {
    let ship = ShipInstanceId::new(ship_id);
    teleport_to_sector(
        ctx,
        ShipInstance::get(ctx, &ship).ok_or(
            "Failed to teleport to sector, couldn't find ship instance."
        )?,
        Sector::get(ctx, &SectorId::new(destination_sector_id)).ok_or(
            "Failed to teleport to sector, couldn't find sector."
        )?
    )
}

/// Hard shift to target sector. This does not take into account jump gate or jump drives.
#[spacetimedb::reducer]
pub fn teleport_to_sector(
    ctx: &ReducerContext,
    mut ship: ShipInstance,
    destination_sector: Sector
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    ship.set_current_sector_id(&destination_sector);
    if ship.get_sobj_id().is_some() {
        if let Some(mut sobj) = StellarObject::get(ctx, &ship.get_sobj_id().unwrap()) {
            sobj.set_sector_id(&destination_sector);
            if let Some(mut transform) = StellarObjectTransformInternal::get(ctx, &sobj.get_id()) {
                transform = transform.from_vec2(Vec2::ZERO);
                dsl.update_sobj_internal_transform_by_sobj_id(transform)?;
            }
            dsl.update_stellar_object_by_id(sobj)?;
        }
        if let Some(mut ship_obj) = dsl.get_ship_object_by_sobj_id(&ship.get_sobj_id().unwrap()) {
            ship_obj.set_sector_id(&destination_sector);
            dsl.update_ship_object_by_ship_and_sobj(ship_obj)?;
        }
    }
    dsl.update_ship_instance_by_id(ship)?;

    Ok(())
}

pub fn teleport_via_jumpgate(
    ctx: &ReducerContext,
    mut ship: ShipInstance,
    jumpgate: &JumpGate
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    ship.set_current_sector_id(&jumpgate.get_target_sector_id());
    if ship.get_sobj_id().is_some() {
        if let Some(mut sobj) = StellarObject::get(ctx, &ship.get_sobj_id().unwrap()) {
            sobj.set_sector_id(&jumpgate.get_target_sector_id());
            if let Some(mut transform) = StellarObjectTransformInternal::get(ctx, &sobj.get_id()) {
                transform = transform.from_vec2(jumpgate.get_target_gate_arrival_pos().to_glam());
                dsl.update_sobj_internal_transform_by_sobj_id(transform)?;
            }
            dsl.update_stellar_object_by_id(sobj)?;
        }
        if let Some(mut ship_obj) = dsl.get_ship_object_by_sobj_id(&ship.get_sobj_id().unwrap()) {
            ship_obj.set_sector_id(&jumpgate.get_target_sector_id());
            dsl.update_ship_object_by_ship_and_sobj(ship_obj)?;
        }
    }
    dsl.update_ship_instance_by_id(ship.clone())?;

    Ok(())
}
