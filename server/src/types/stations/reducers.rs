use log::info;
use spacetimedb::ScheduleAt;
use std::time::Duration;

use crate::types::{
    common::utility::*,
    items::{ItemDefinitionId, *},
    players::{GetPlayerRowOptionById, UpdatePlayerRowById},
    server_messages::utility::{send_error_message, send_info_message},
    ships::{utility::*, *},
    stations::timers::*,
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
    quantity: u32,
) -> Result<(), String> {
    is_server_or_ship_owner(ctx, Some(docked_ship_id.clone()))?;
    let dsl = dsl(ctx);

    let ship = dsl.get_docked_ship_by_id(&docked_ship_id)?;

    // Validate that the docked ship is at the same station as the module
    let station_module = dsl.get_station_module_by_id(&station_module_id)?;
    if ship.get_station_id() != station_module.get_station_id() {
        let player_id = ship.get_player_id().clone();
        let error_message = format!(
            "Cannot buy from station module: Your ship is docked at station {} but the module is at station {}.",
            ship.get_station_id(),
            station_module.get_station_id()
        );

        // Send server message for error feedback
        send_error_message(
            ctx,
            &player_id,
            error_message.clone(),
            Some("Station Trading"),
        )?;

        return Err(error_message);
    }

    // Get Trading Port Module and it's inventory item that matches the item_id
    //let trading_port_module = dsl.get_trading_port_module_by_id(&station_module_id)?;

    // TODO: Check faction standing.

    let mut item_listing = dsl
        .get_station_module_inventory_items_by_module_id(&station_module_id)
        .filter(|item| item.get_resource_item_id() == item_id)
        .next()
        .ok_or_else(|| {
            let player_id = ship.get_player_id().clone();
            let error_message = format!(
                "Cannot buy item #{}: This item is not available at this station.",
                item_id
            );

            // Send server message for error feedback
            let _ = send_error_message(
                ctx,
                &player_id,
                error_message.clone(),
                Some("Station Trading"),
            );

            format!(
                "Cannot sell #{} to station: No matching inventory item found.",
                item_id
            )
        })?;
    let item_def = dsl.get_item_definition_by_id(item_id)?;

    if item_listing.get_quantity() < &quantity {
        let player_id = ship.get_player_id().clone();
        let error_message = format!(
            "Cannot buy {}x {} from station: Not enough items available. Station has {} but you requested {}.",
            quantity,
            item_def.get_name(),
            item_listing.get_quantity(),
            quantity
        );

        // Send server message for error feedback
        send_error_message(
            ctx,
            &player_id,
            error_message.clone(),
            Some("Station Trading"),
        )?;

        return Err(format!(
            "Cannot buy {}x {} from station: Not enough items.",
            quantity,
            item_def.get_name()
        ));
    }

    // Use cached current price for performance
    let total_price = item_listing.get_cached_price() * quantity;

    // Check if the player has enough credits
    let mut player = dsl.get_player_by_id(ship.get_player_id())?;
    if (total_price as u64) > *player.get_credits() {
        let player_id = ship.get_player_id().clone();
        let error_message = format!(
            "Cannot buy {}x {} from station: Not enough credits. You have {}c but it costs {}c.",
            quantity,
            item_def.get_name(),
            player.get_credits(),
            total_price
        );

        // Send server message for error feedback
        send_error_message(
            ctx,
            &player_id,
            error_message.clone(),
            Some("Station Trading"),
        )?;

        return Err(format!(
            "Cannot buy {}x {} from station: Not enough credits. You have {}c but it costs {}c.",
            quantity,
            item_def.get_name(),
            player.get_credits(),
            total_price
        ));
    }

    if let Err(cargo_err) = attempt_to_load_cargo_into_ship(
        ctx,
        &mut dsl.get_ship_status_by_id(&docked_ship_id)?,
        &docked_ship_id,
        &item_def,
        quantity as u16,
        false,
    ) {
        let error_message = format!(
            "Cannot buy {}x {} from station: {}",
            quantity,
            item_def.get_name(),
            cargo_err
        );

        // Send server message for error feedback
        send_error_message(
            ctx,
            &ship.get_player_id(),
            error_message.clone(),
            Some("Station Trading"),
        )?;

        return Err(cargo_err);
    }

    player.set_credits(*player.get_credits() - (total_price as u64));
    // TOOD: Add credits to station
    dsl.update_player_by_id(player)?;

    item_listing.set_quantity(item_listing.get_quantity() - quantity);
    item_listing.set_cached_price(item_listing.calculate_current_price(&item_def));
    dsl.update_station_module_inventory_item_by_id(item_listing)?;

    send_info_message(
        ctx,
        &ship.get_player_id(),
        format!(
            "Station #{} Module #{}: Bought {}x {} for {}c.",
            station_module.station_id,
            station_module_id.value(),
            quantity,
            item_def.get_name(),
            total_price
        ),
        Some("station_module"),
    )?;

    Ok(())
}

/// A docked ship sells an item to a station module and its player (or faction) receives credits in return.
#[spacetimedb::reducer]
pub fn sell_item_to_station_module(
    ctx: &ReducerContext,
    station_module_id: StationModuleId,
    docked_ship_id: ShipGlobalId,
    item_id: ItemDefinitionId,
    quantity: u32,
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

    // Validate that the docked ship is at the same station as the module
    let station_module = dsl.get_station_module_by_id(&station_module_id)?;
    if ship.get_station_id() != station_module.get_station_id() {
        let player_id = ship.get_player_id().clone();
        let error_message = format!(
            "Cannot sell to station module: Your ship is docked at station {} but the module is at station {}.",
            ship.get_station_id(),
            station_module.get_station_id()
        );

        // Send server message for error feedback
        send_error_message(
            ctx,
            &player_id,
            error_message.clone(),
            Some("Station Trading"),
        )?;

        return Err(error_message);
    }

    // Get Trading Port Module and it's inventory item that matches the item_id
    ////let trading_port_module = dsl.get_trading_port_module_by_id(&station_module_id)?;

    // TODO: Check faction standing.

    let mut item_listing = dsl
        .get_station_module_inventory_items_by_module_id(&station_module_id)
        .filter(|item| item.get_resource_item_id() == item_id)
        .next()
        .ok_or_else(|| {
            let player_id = ship.get_player_id().clone();
            let error_message = format!(
                "Cannot sell item #{}: This station does not accept this item type.",
                item_id
            );

            // Send server message for error feedback
            let _ = send_error_message(
                ctx,
                &player_id,
                error_message.clone(),
                Some("Station Trading"),
            );

            format!(
                "Cannot sell #{} to station: No matching inventory item found.",
                item_id
            )
        })?;
    let item_def = dsl.get_item_definition_by_id(item_id)?;

    if item_listing.get_quantity() + &quantity > *item_listing.get_max_quantity() {
        let player_id = ship.get_player_id().clone();
        let error_message = format!(
            "Cannot sell {}x {} to station: Not enough space left in module inventory. Station can only accept {} more items.",
            quantity,
            item_def.get_name(),
            item_listing.get_max_quantity() - item_listing.get_quantity()
        );

        // Send server message for error feedback
        send_error_message(
            ctx,
            &player_id,
            error_message.clone(),
            Some("Station Trading"),
        )?;

        return Err(format!(
            "Cannot sell {}x {} to station: Not enough space left in module inventory.",
            quantity,
            item_def.get_name()
        ));
    }

    // Use cached current price for performance
    let total_price = item_listing.get_cached_price() * quantity; // cache buy/sell prices separately

    // Check if the station has enough credits
    //if total_price <= *station.get_credits() {
    if let Err(cargo_err) = remove_cargo_from_ship(
        ctx,
        &mut dsl.get_ship_status_by_id(&docked_ship_id)?,
        &item_def,
        quantity as u16,
    ) {
        let player_id = ship.get_player_id().clone();
        let error_message = format!(
            "Cannot sell {}x {} to station: {}",
            quantity,
            item_def.get_name(),
            cargo_err
        );

        // Send server message for error feedback
        send_error_message(
            ctx,
            &player_id,
            error_message.clone(),
            Some("Station Trading"),
        )?;

        return Err(cargo_err);
    }

    let mut player = dsl.get_player_by_id(ship.get_player_id())?;
    player.set_credits(player.get_credits() + &(total_price as u64));
    dsl.update_player_by_id(player)?;

    item_listing.set_quantity(item_listing.get_quantity() + quantity);
    item_listing.set_cached_price(item_listing.calculate_current_price(&item_def));
    dsl.update_station_module_inventory_item_by_id(item_listing)?;

    send_info_message(
        ctx,
        &ship.get_player_id(),
        format!(
            "Station #{} Module #{}: Sold {}x {} for {}c.",
            station_module.station_id,
            station_module_id.value(),
            quantity,
            item_def.get_name(),
            total_price
        ),
        Some("station_module"),
    )?;

    Ok(())
}

/// Creates station timers for the given station ID.
/// Sets up both production and status schedules for the station.
#[spacetimedb::reducer]
pub fn add_station_timers(ctx: &ReducerContext, station_id: u64) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(ctx)?;
    let station_id = StationId::new(station_id);

    // Verify the station exists
    let _station = dsl.get_station_by_id(&station_id)?;

    // Check if production schedule already exists
    if dsl
        .get_station_production_schedule_by_id(&station_id)
        .is_ok()
    {
        info!(
            "Station production schedule already exists for station {}",
            station_id
        );
    } else {
        // Set up station production schedule (every 30 seconds)
        dsl.create_station_production_schedule(
            &station_id,
            ScheduleAt::Interval(Duration::from_secs(30).into()), // TODO: Make this dependant on a GlobalConfig value
            ctx.timestamp,
        )?;
    }

    // Check if status schedule already exists
    if dsl.get_station_status_schedule_by_id(&station_id).is_ok() {
        info!(
            "Station status schedule already exists for station {}",
            station_id
        );
    } else {
        // Set up station status schedule (every 10 seconds)
        dsl.create_station_status_schedule(
            &station_id,
            ScheduleAt::Interval(Duration::from_secs(10).into()), // TODO: Make this dependant on a GlobalConfig value
            ctx.timestamp,
        )?;
    }

    info!("Created station timers for station {}", station_id);

    Ok(())
}
