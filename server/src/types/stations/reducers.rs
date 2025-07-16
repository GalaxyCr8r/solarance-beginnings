use log::info;

use crate::types::{
    common::utility::*,
    items::{ GetItemDefinitionRowOptionById, ItemDefinitionId },
    players::{ GetPlayerRowOptionById, UpdatePlayerRowById },
    ships::{ utility::*, * },

};

use super::*;

///////////////////////////////////////////////////////////
// Reducers ///
///////////////////////////////////////////////////////////

/// Allows a docked ship to purchase items from a station's trading module.
/// Validates player ownership, credits, and cargo space before completing the transaction.
#[spacetimedb::reducer]
pub fn buy_item_from_station_module(
    ctx: &ReducerContext,
    station_module_id: StationModuleId,
    docked_ship_id: ShipGlobalId,
    item_id: ItemDefinitionId,
    quantity: u32
) -> Result<(), String> {
    is_server_or_ship_owner(ctx, Some(docked_ship_id.clone()))?;
    let dsl = dsl(ctx);

    let ship = dsl.get_docked_ship_by_id(&docked_ship_id)?;

    // Get Trading Port Module and it's inventory item that matches the item_id
    //let trading_port_module = dsl.get_trading_port_module_by_id(&station_module_id)?;

    // TODO: Check faction standing.

    let mut item_listing = dsl
        .get_station_module_inventory_items_by_module_id(&station_module_id)
        .filter(|item| item.get_resource_item_id() == item_id)
        .next()
        .ok_or(format!("Cannot sell #{} to station: No matching inventory item found.", item_id))?;
    let item_def = dsl.get_item_definition_by_id(item_id)?;

    if item_listing.get_quantity() < &quantity {
        return Err(
            format!(
                "Cannot buy {}x {} from station: Not enough items.",
                quantity,
                item_def.get_name()
            )
        );
    }

    // Use cached current price for performance
    let total_price = item_listing.get_cached_price() * quantity;

    // Check if the player has enough credits
    let mut player = dsl.get_player_by_id(ship.get_player_id())?;
    if (total_price as u64) > *player.get_credits() {
        return Err(
            format!(
                "Cannot buy {}x {} from station: Not enough credits. You have {}c but it costs {}c.",
                quantity,
                item_def.get_name(),
                player.get_credits(),
                total_price
            )
        );
    }

    attempt_to_load_cargo_into_ship(
        ctx,
        &mut dsl.get_ship_status_by_id(&docked_ship_id)?,
        &docked_ship_id,
        &item_def,
        quantity as u16,
        false
    )?;

    player.set_credits(*player.get_credits() - (total_price as u64));
    // TOOD: Add credits to station
    dsl.update_player_by_id(player)?;

    item_listing.set_quantity(item_listing.get_quantity() - quantity);
    item_listing.set_cached_price(item_listing.calculate_current_price(&item_def));
    dsl.update_station_module_inventory_item_by_id(item_listing)?;

    Ok(())
}

/// A docked ship sells an item to a station module and its player (or faction) receives credits in return.
#[spacetimedb::reducer]
pub fn sell_item_to_station_module(
    ctx: &ReducerContext,
    station_module_id: StationModuleId,
    docked_ship_id: ShipGlobalId,
    item_id: ItemDefinitionId,
    quantity: u32
) -> Result<(), String> {
    is_server_or_ship_owner(ctx, Some(docked_ship_id.clone()))?;
    let dsl = dsl(ctx);

    let ship = dsl.get_docked_ship_by_id(&docked_ship_id)?;

    info!(
        "Attempting to sell {}x {} from ship {} to trading port {}",
        quantity,
        item_id,
        ship.get_id(),
        station_module_id
    );

    // Get Trading Port Module and it's inventory item that matches the item_id
    ////let trading_port_module = dsl.get_trading_port_module_by_id(&station_module_id)?;

    // TODO: Check faction standing.

    let mut item_listing = dsl
        .get_station_module_inventory_items_by_module_id(&station_module_id)
        .filter(|item| item.get_resource_item_id() == item_id)
        .next()
        .ok_or(format!("Cannot sell #{} to station: No matching inventory item found.", item_id))?;
    let item_def = dsl.get_item_definition_by_id(item_id)?;

    if item_listing.get_quantity() + &quantity > *item_listing.get_max_quantity() {
        return Err(
            format!(
                "Cannot sell {}x {} to station: Not enough space left in module inventory.",
                quantity,
                item_def.get_name()
            )
        );
    }

    // Use cached current price for performance
    let total_price = item_listing.get_cached_price() * quantity; // cache buy/sell prices separately

    // Check if the station has enough credits
    //if total_price <= *station.get_credits() {
    remove_cargo_from_ship(
        ctx,
        &mut dsl.get_ship_status_by_id(&docked_ship_id)?,
        &item_def,
        quantity as u16
    )?;

    let mut player = dsl.get_player_by_id(ship.get_player_id())?;
    player.set_credits(player.get_credits() + &(total_price as u64));
    dsl.update_player_by_id(player)?;

    item_listing.set_quantity(item_listing.get_quantity() + quantity);
    item_listing.set_cached_price(item_listing.calculate_current_price(&item_def));
    dsl.update_station_module_inventory_item_by_id(item_listing)?;
    //}

    Ok(())
}
