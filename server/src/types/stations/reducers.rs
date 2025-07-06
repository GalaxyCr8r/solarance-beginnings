use crate::types::{
    common::utility::*,
    items::{ GetItemDefinitionRowOptionById, ItemDefinitionId },
    players::{ GetPlayerRowOptionById, UpdatePlayerRowById },
    ships::{ utility::*, * },
    stations::modules::{ trading_port_module, GetTradingPortModuleRowOptionById },
    stellarobjects::*,
};

use super::*;

///////////////////////////////////////////////////////////
// Reducers ///
///////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn buy_item_from_trading_port(
    ctx: &ReducerContext,
    trading_port_id: StationModuleId,
    docked_ship_id: ShipGlobalId,
    item_id: ItemDefinitionId,
    quantity: u32
) -> Result<(), String> {
    is_server_or_ship_owner(ctx, Some(docked_ship_id.clone()))?;
    let dsl = dsl(ctx);

    let ship = dsl.get_docked_ship_by_id(&docked_ship_id)?;

    // Get Trading Port Module and it's inventory item that matches the item_id
    let trading_port_module = dsl.get_trading_port_module_by_id(&trading_port_id)?;

    // TODO: Check faction standing.

    if
        let Some(mut item_listing) = dsl
            .get_station_module_inventory_items_by_module_id(&trading_port_id)
            .filter(|item| { item.get_resource_item_id() == item_id })
            .next()
    {
        if item_listing.get_quantity() >= &quantity {
            // Based on the ratio of quantity to max quantity, and using the item's base price, decide on total price
            let item_def = dsl.get_item_definition_by_id(item_id)?;
            let total_price = (item_listing.current_price(&item_def) * (quantity as f32)) as u64;

            let mut player = dsl.get_player_by_id(ship.get_player_id())?;

            // Check if the player has enough credits
            if total_price <= *player.get_credits() {
                attempt_to_load_cargo_into_ship(
                    ctx,
                    &mut dsl.get_ship_status_by_id(&docked_ship_id)?,
                    &docked_ship_id,
                    &item_def,
                    quantity as u16,
                    false
                )?;

                player.set_credits(*player.get_credits() - total_price);
                // TOOD: Add credits to station
                dsl.update_player_by_id(player)?;

                item_listing.set_quantity(item_listing.get_max_quantity() - quantity);
                dsl.update_station_module_inventory_item_by_id(item_listing)?;
            }
        }
    }

    Ok(())
}

#[spacetimedb::reducer]
pub fn sell_item_to_trading_port(
    ctx: &ReducerContext,
    trading_port_id: StationModuleId,
    docked_ship_id: ShipGlobalId,
    item_id: ItemDefinitionId,
    quantity: u32
) -> Result<(), String> {
    is_server_or_ship_owner(ctx, Some(docked_ship_id.clone()))?;
    let dsl = dsl(ctx);

    let ship = dsl.get_docked_ship_by_id(&docked_ship_id)?;

    // Get Trading Port Module and it's inventory item that matches the item_id
    let trading_port_module = dsl.get_trading_port_module_by_id(&trading_port_id)?;

    // TODO: Check faction standing.

    if
        let Some(mut item_listing) = dsl
            .get_station_module_inventory_items_by_module_id(&trading_port_id)
            .filter(|item| { item.get_resource_item_id() == item_id })
            .next()
    {
        if item_listing.get_quantity() + &quantity <= *item_listing.get_max_quantity() {
            // Based on the ratio of quantity to max quantity, and using the item's base price, decide on total price
            let item_def = dsl.get_item_definition_by_id(item_id)?;
            let total_price = (item_listing.current_price(&item_def) * (quantity as f32)) as u64;

            let mut player = dsl.get_player_by_id(ship.get_player_id())?;

            // Check if the station has enough credits
            //if total_price <= *station.get_credits() {
            remove_cargo_from_ship(
                ctx,
                &mut dsl.get_ship_status_by_id(&docked_ship_id)?,
                &item_def,
                quantity as u16
            )?;

            player.set_credits(*player.get_credits() + total_price);
            dsl.update_player_by_id(player)?;

            item_listing.set_quantity(item_listing.get_quantity() + quantity);
            dsl.update_station_module_inventory_item_by_id(item_listing)?;
            //}
        }
    }

    Ok(())
}
