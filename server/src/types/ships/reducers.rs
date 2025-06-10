use crate::types::{items::{GetItemDefinitionRowOptionById, ItemDefinition}, ships::utility::create_cargo_crate_nearby_ship, utility::{is_server_or_owner, try_server_only}};

use super::*;

///////////////////////////////////////////////////////////
// Reducers ///
///////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn jettison_cargo_from_ship(ctx: &ReducerContext, ship_id: u64, ship_cargo_id: u64, amount: u16) -> Result<(), String> {
    info!("Yo bruh we got the dets {}, {}, for x{}!", ship_id, ship_cargo_id, amount);

    let dsl = dsl(ctx);
    let ship_object = dsl.get_ship_object_by_ship_id(ShipInstanceId::new(ship_id)).ok_or(
        format!("Blah1") // TODO error message here
    )?;
    
    is_server_or_owner(ctx, ship_object.get_sobj_id())?;

    let mut ship_cargo = dsl.get_ship_cargo_item_by_id(ShipCargoItemId::new(ship_cargo_id)).ok_or(
        format!("Blah2") // TODO error message here
    )?;
    let item_def = dsl.get_item_definition_by_id(ship_cargo.get_item_id()).ok_or(
        format!("Blah3") // TODO error message here
    )?;

    // Does the ship actually have that amount of item?
    if ship_cargo.get_quantity() < &amount {
        return Err(format!("Blah4")) // TODO error message here
    } else if ship_cargo.get_quantity() == &amount {
        dsl.delete_ship_cargo_item_by_id(&ship_cargo);
    } else {
        ship_cargo.quantity -= amount;
        dsl.update_ship_cargo_item_by_id(ship_cargo)?;
    }

    create_cargo_crate_nearby_ship(ctx, &ship_object.get_sobj_id(), &item_def, amount)?;

    Ok(())
}

// #[spacetimedb::reducer]
// pub fn try_create_ship_cargo_item(ctx: &ReducerContext, ship: ShipInstance, item: ItemDefinition, amount: u16) -> Result<(), String> {
//     try_server_only(ctx)?;
//     let dsl = dsl(ctx);

//     if ship.get_remaining_cargo_space() >= *item.get_volume_per_unit() * amount {
//         let _ = dsl.create_ship_cargo_item(&ship, &item, amount)?; // IF this function is going to stay - it also needs to update the ship instance's used cargo space

//         Ok(())
//     } else {
//         Err("Not enough cargo space".to_string())
//     }
// }