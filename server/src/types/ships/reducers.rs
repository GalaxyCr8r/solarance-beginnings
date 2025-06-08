use crate::types::{items::ItemDefinition, utility::try_server_only};

use super::*;

///////////////////////////////////////////////////////////
// Reducers ///
///////////////////////////////////////////////////////////

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