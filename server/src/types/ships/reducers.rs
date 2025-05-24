use crate::types::items::ItemDefinition;

use super::*;

///////////////////////////////////////////////////////////
// Reducers ///
///////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn attempt_to_load_cargo_into_ship(ctx: &ReducerContext, ship: ShipInstance, item: ItemDefinition, amount: u16) -> Result<(), String> {
    let dsl = dsl(ctx);

    if *ship.get_cargo_capacity() >= *item.get_volume_per_unit() * amount {
        let _ = dsl.create_ship_cargo_item(&ship, &item, 5)?;

        Ok(())
    } else {
        Err("Not enough cargo space".to_string())
    }
}