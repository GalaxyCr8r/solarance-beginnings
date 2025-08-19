use std::time::Duration;

use glam::Vec2;
use log::info;
use spacetimedb::{rand::Rng, TimeDuration};

use crate::types::{
    common::utility::try_server_only,
    factions::FactionId,
    jumpgates::JumpGate,
    players::{timers::initialize_player_controller, PlayerId},
    server_messages::utility::send_info_message,
    ships::{reducers::teleport_to_sector, timers::*},
    stellarobjects::{reducers::create_sobj_player_window_for, utility::*, *},
};

use super::*;

pub fn same_sector_from_ids(
    ctx: &ReducerContext,
    id1: &StellarObjectId,
    id2: &StellarObjectId,
) -> bool {
    let dsl = dsl(ctx);

    if let Ok(sobj1) = dsl.get_stellar_object_by_id(id1) {
        if let Ok(sobj2) = dsl.get_stellar_object_by_id(id2) {
            return sobj1.get_sector_id() == sobj2.get_sector_id();
        }
    }
    false
}

/// Creates a brand new ship instance in a sector with a specific stellar object.
pub fn create_ship_from_sobj(
    ctx: &ReducerContext,
    ship_type: &ShipTypeDefinition,
    player_id: &PlayerId,
    sobj: &StellarObject,
) -> Result<(Ship, ShipStatus), String> {
    let dsl = dsl(ctx);

    let ship = (match dsl.create_ship(
        ship_type.get_id(),
        ShipLocation::Sector,
        sobj,
        StationId::new(0),
        sobj.get_sector_id(),
        player_id,
        FactionId::new(0),
    ) {
        Ok(ship) => {
            create_status_timer_for_ship(ctx, &ship.get_id(), &ship_type.get_id())?;
            Ok(ship)
        }
        Err(e) => Err(e.to_string()),
    })?;

    let ship_status = dsl.create_ship_status(
        &ship,
        sobj.get_sector_id(),
        player_id,
        ship_type.max_health as f32,
        ship_type.max_shields as f32,
        ship_type.max_energy as f32,
        0,
        ship_type.cargo_capacity,
        None,
    )?;

    return Ok((ship, ship_status));
}

/// Creates a brand new ship instance docked at a station.
pub fn create_ship_docked_at_station(
    ctx: &ReducerContext,
    ship_type: ShipTypeDefinition,
    player_id: PlayerId,
    station: Station,
) -> Result<(Ship, ShipStatus), String> {
    let dsl = dsl(ctx);

    let ship = (match dsl.create_ship(
        ship_type.get_id(),
        ShipLocation::Station,
        station.get_sobj_id(),
        &station,
        station.get_sector_id(),
        &player_id,
        FactionId::new(0),
    ) {
        Ok(ship) => {
            create_status_timer_for_ship(ctx, &ship.get_id(), &ship_type.get_id())?;
            Ok(ship)
        }
        Err(e) => Err(e.to_string()),
    })?;

    let ship_status = dsl.create_ship_status(
        &ship,
        station.get_sector_id(),
        &player_id,
        ship_type.max_health as f32,
        ship_type.max_shields as f32,
        ship_type.max_energy as f32,
        0,
        ship_type.cargo_capacity,
        None,
    )?;

    return Ok((ship, ship_status));
}

/// Removes cargo from a ship's cargo hold. Errors if the ship doesn't have enough of the item.
pub fn remove_cargo_from_ship(
    ctx: &ReducerContext,
    ship_status: &mut ShipStatus,
    item_def: &ItemDefinition,
    amount: u16,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let mut remaining_amount = amount;

    if amount == 0 {
        return Err(format!(
            "Tried to remove 0 amount of {} from ship #{:?}",
            item_def.get_name(),
            ship_status.get_id()
        ));
    }

    ship_status.used_cargo_capacity = ship_status.calculate_used_cargo_space(ctx); // Just go through and make sure everything is ship-shape.
    info!(
        "Attempting to remove {}x {} ({}v) from ship #{} with remaining cargo space of {}v",
        amount,
        item_def.get_name(),
        amount * item_def.get_volume_per_unit(),
        ship_status.id,
        ship_status.get_remaining_cargo_space()
    );

    // Check if the ship has enough of the item
    for cargo_item in dsl.get_ship_cargo_items_by_ship_id(ship_status.get_id()) {
        if cargo_item.get_item_id() == item_def.get_id() {
            if *cargo_item.get_quantity() <= remaining_amount {
                // If so, deduct the stack amount from remaining amount, and remove the stack from inventory. Continue looping.
                remaining_amount -= cargo_item.get_quantity();
                info!(
                    "Found a stack that fits all the remaining amount {}. Removing from cargo item for ship #{:?}: {}x {}",
                    remaining_amount,
                    ship_status.get_id(),
                    *cargo_item.get_quantity(),
                    item_def.get_name()
                );

                dsl.delete_ship_cargo_item_by_id(&cargo_item.get_id())?;
                continue;
            } else {
                info!(
                    "Found an existing stack of {} {} that has more than the requested quantity, removing {}",
                    cargo_item.get_quantity(),
                    item_def.get_name(),
                    remaining_amount
                );

                // Remove the amount from the stack and update it.
                let mut updated_cargo_item = cargo_item;

                let new_amount = updated_cargo_item.get_quantity() - remaining_amount;
                info!("New amount after removal: {}", new_amount);
                updated_cargo_item.set_quantity(new_amount);
                dsl.update_ship_cargo_item_by_id(updated_cargo_item)?;

                ship_status.used_cargo_capacity = ship_status.calculate_used_cargo_space(ctx);
                let _ = dsl.update_ship_status_by_id(ship_status.clone())?;

                return Ok(());
            }
        }
    }
    if remaining_amount > 0 {
        return Err(format!(
            "Ship #{} does not have enough {} to remove: requested {}, available 0",
            ship_status.id,
            item_def.get_name(),
            remaining_amount
        ));
    }

    // Update ship status after removing cargo items
    ship_status.used_cargo_capacity = ship_status.calculate_used_cargo_space(ctx);
    let _ = dsl.update_ship_status_by_id(ship_status.clone())?;

    Ok(())
}

/// Loads cargo into a ship's cargo hold, preferring existing cargo items.
/// It creates new cargo items if necessary, but if it can't it will create
/// a cargo crate instead if create_a_crate_if_failed is true and Ship
/// points to a Ship and not a DockedShip row.
pub fn attempt_to_load_cargo_into_ship(
    ctx: &ReducerContext,
    ship_status: &mut ShipStatus,
    ship_id: &ShipId,
    item_def: &ItemDefinition,
    amount: u16,
    create_a_crate_if_failed: bool,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let mut remaining_amount = amount;
    let mut overflow_amount = 0; // How many items could NEVER had fit in the ship and must be made into a crate.
    let units_per_stack = *item_def.get_units_per_stack() as u16;

    if amount == 0 {
        return Err(format!(
            "Tried to load 0 amount of {} into ship #{:?}",
            item_def.get_name(),
            ship_status.get_id()
        ));
    }

    ship_status.used_cargo_capacity = ship_status.calculate_used_cargo_space(ctx); // Just go through and make sure everything is ship-shape.
    info!(
        "Attempting to load {}x {} ({}v) into ship #{} with remaining cargo space of {}v",
        amount,
        item_def.get_name(),
        amount * item_def.get_volume_per_unit(),
        ship_status.id,
        ship_status.get_remaining_cargo_space()
    );

    // First check how many items can actually fit inside the cargo hold
    let additional_items_that_can_fit =
        ship_status.get_remaining_cargo_space() / item_def.get_volume_per_unit();
    if additional_items_that_can_fit < amount {
        overflow_amount = amount - additional_items_that_can_fit;
        remaining_amount = additional_items_that_can_fit;
        info!(
            "WARN: We can only fit {} more items, but we've been requested to add {}. Sending {} to overflow",
            additional_items_that_can_fit,
            amount,
            overflow_amount
        );
        info!(
            "Expected final used cargo capacity: {} / {}",
            ship_status.get_used_cargo_capacity()
                + additional_items_that_can_fit * item_def.get_volume_per_unit(),
            ship_status.max_cargo_capacity
        );
    }

    // Update already existing stacks of the item in the ship's cargo
    if remaining_amount > 0 {
        for mut cargo_item in dsl.get_ship_cargo_items_by_ship_id(ship_status.get_id()) {
            if cargo_item.get_item_id() == item_def.get_id() {
                // If it exists, then try to fill the stack.
                if cargo_item.quantity == units_per_stack {
                    continue;
                }

                let new_amount = if cargo_item.get_quantity() + remaining_amount > units_per_stack {
                    // If the new amount exceeds the max amount, we need to split it
                    info!(
                        "Found an existing stack of {} {}, filling to max amount...",
                        cargo_item.get_quantity(),
                        item_def.get_name()
                    );
                    remaining_amount =
                        cargo_item.get_quantity() + remaining_amount - units_per_stack;
                    units_per_stack
                } else {
                    info!(
                        "Found an existing stack of {} {}, adding {}",
                        cargo_item.get_quantity(),
                        item_def.get_name(),
                        remaining_amount
                    );
                    let tmp_amount = remaining_amount;
                    remaining_amount = 0; // We are loading the rest of the amount!
                    cargo_item.get_quantity() + tmp_amount
                };

                // If we got this far, then we're updating the cargo item.
                cargo_item.set_quantity(new_amount);
                info!(
                    "Updating cargo item for ship #{:?}: {}x {}",
                    ship_status.get_id(),
                    new_amount,
                    item_def.get_name()
                );
                let _ = dsl.update_ship_cargo_item_by_id(cargo_item)?;
            }
        }
    }

    // If there's still remaining amount that isn't in the overflow, then that must mean we still have cargo space for them.
    if remaining_amount > 0 {
        info!(
            "Remaining amount to load {}x {} into ship #{} with remaining cargo space of {}v",
            remaining_amount,
            item_def.get_name(),
            ship_status.id,
            ship_status.get_remaining_cargo_space()
        );

        // Make as many stacks as you need to.
        while remaining_amount > 0 {
            // If the item does not exist, we need to create a new cargo item
            let stack_amount = if remaining_amount > units_per_stack {
                units_per_stack
            } else {
                remaining_amount
            };
            remaining_amount -= stack_amount;

            // Create the cargo item
            info!(
                "Creating cargo item for ship #{:?}: {}x {}",
                ship_status.get_id(),
                stack_amount,
                item_def.get_name()
            );
            if let Err(e) =
                dsl.create_ship_cargo_item(ship_status.get_id(), item_def, stack_amount.into())
            {
                info!(
                    "Failed to create cargo item for ship {:?}, adding {} to overflow: {}",
                    ship_status.get_id(),
                    stack_amount,
                    e
                );
                overflow_amount += stack_amount;
            }
        }
    }
    ship_status.used_cargo_capacity = ship_status.calculate_used_cargo_space(ctx); // Just go through and make sure everything is ship-shape.

    if overflow_amount > 0 {
        info!(
            "Not enough cargo space: Remaining {} / Required {}",
            ship_status.get_used_cargo_capacity(),
            item_def.get_volume_per_unit() * overflow_amount
        );

        // If not enough space, check if we create a cargo crate instead
        if create_a_crate_if_failed {
            let ship_instance = dsl.get_ship_by_id(ship_id)?;
            if ship_instance.location == ShipLocation::Sector {
                // Only spawn if the ship is in a sector
                create_cargo_crate_nearby_ship(
                    ctx,
                    &ship_instance.get_sobj_id(),
                    item_def,
                    overflow_amount,
                )?;
            } else {
                return Err(
                    "Failed to create cargo crate because the ship_id does not point to a Ship in a sector.".to_string()
                );
            }
        } else {
            return Err(
                "Failed to load cargo because it ran out space inside the ship".to_string(),
            );
        }
    }

    ship_status.used_cargo_capacity = ship_status.calculate_used_cargo_space(ctx); // Just go through and make sure everything is ship-shape FINALLY.
    if ship_status.used_cargo_capacity > ship_status.max_cargo_capacity {
        return Err(
            "Despite our best efforts, we ended up with more cargo used than is maximum!"
                .to_string(),
        );
    }
    let _ = dsl.update_ship_status_by_id(ship_status.clone())?;

    send_info_message(
        ctx,
        &ship_status.get_player_id(),
        format!("Loaded successfully {}x {}", amount, item_def.get_name(),),
        Some("status"),
    )?;

    Ok(())
}

/// Crates a cargo crate nearby the given stellar object if it exists,
/// otherwise it'll place it randomly in its last known sector.
pub fn create_cargo_crate_nearby_ship(
    ctx: &ReducerContext,
    ship_sobj: &StellarObjectId,
    item_def: &ItemDefinition,
    quantity: u16,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    let sobj = dsl.get_stellar_object_by_id(ship_sobj)?;
    let pos = {
        if let Ok(transform) = dsl.get_sobj_internal_transform_by_id(ship_sobj) {
            transform.to_vec2()
        } else {
            info!("Could not find ship's stellar object transform, placing randomly...");
            Vec2::new(
                ctx.rng().gen_range(-2048.0..2048.0),
                ctx.rng().gen_range(-2048.0..2048.0),
            )
        }
    };

    let new_sobj = create_sobj_with_random_velocity(
        ctx,
        StellarObjectKinds::CargoCrate,
        &sobj.get_sector_id(),
        pos.x,
        pos.y,
        0.125,
        Some(0.9995),
    )?;

    info!(
        "Created cargo crate in sector #{:?} at {}, {}!",
        &sobj.get_sector_id(),
        pos.x,
        pos.y
    );

    let _ = dsl.create_cargo_crate(
        sobj.get_sector_id(),
        new_sobj.get_id(),
        item_def.get_id(),
        quantity,
        ctx.timestamp
            .checked_add(TimeDuration::from_duration(Duration::from_secs(
                /*D* /24 * /*H*/60 * */ /*M*/ 60,
            ))), // TODO cargo crate timer to despawn them
        None,
    )?;
    Ok(())
}

pub fn teleport_via_jumpgate(
    ctx: &ReducerContext,
    ship: Ship,
    jumpgate: &JumpGate,
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    let pos: &crate::types::common::Vec2 = jumpgate.get_target_gate_arrival_pos();
    teleport_to_sector(
        ctx,
        ship,
        dsl.get_sector_by_id(jumpgate.get_target_sector_id())?,
        pos.x,
        pos.y,
    )
}

/// Creates the DockedShip object plus removes the Ship and StellarObject but keeps the cargo, health, etc.
pub fn dock_to_station(
    ctx: &ReducerContext,
    ship: &Ship,
    ship_sobj: &StellarObject,
    station: &Station,
) -> Result<Ship, String> {
    let dsl = dsl(ctx);

    // Remove Ship and StellarObject
    let _ = dsl.delete_stellar_object_by_id(ship_sobj);
    let _ = dsl.delete_ship_by_id(ship.get_id());

    // Create DockedShip object
    let docked = &mut ship.clone();
    docked.set_station_id(station.get_id());
    dsl.update_ship_by_id(docked.clone());

    send_info_message(
        ctx,
        &ship.get_player_id(),
        format!(
            "Docked successfully with Station #{}: {}",
            station.get_id().value(),
            station.get_name(),
        ),
        Some("status"),
    )?;

    Ok(docked.clone())
}

pub fn undock_from_station(ctx: &ReducerContext, docked: Ship) -> Result<Ship, String> {
    let dsl = dsl(ctx);

    let station = dsl.get_station_by_id(docked.get_station_id())?;
    let station_transform = dsl.get_sobj_internal_transform_by_id(station.get_sobj_id())?;
    let ship_type = dsl.get_ship_type_definition_by_id(docked.get_shiptype_id())?;

    let sobj = create_sobj_internal(
        ctx,
        StellarObjectKinds::Ship,
        &station.get_sector_id(),
        station_transform,
    )?;

    let ship = &mut docked.clone();
    ship.set_sector_id(station.get_sector_id());
    ship.set_station_id(StationId::new(0));

    // Ensure there's still a ship status timer.
    if dsl
        .get_ship_status_timer_by_ship_id(docked.get_id())
        .is_err()
    {
        let _ = create_status_timer_for_ship(ctx, &ship.get_id(), &ship_type.get_id());
    }

    if docked.player_id != Identity::ONE {
        // There is a real player controlling this ship, so create the necessary helpers.
        let _ = create_sobj_player_window_for(ctx, docked.player_id, sobj.get_id())?;
        let _ = initialize_player_controller(ctx, &docked.get_player_id(), &sobj);
    } else {
        // There is NOT a real player controllering this ship, so error for now.
        return Err("Unsupported: There was an attempt to undock an NPC ship!".to_string());
    }

    dsl.update_ship_by_id(ship.clone())?;
    send_info_message(
        ctx,
        &ship.get_player_id(),
        format!(
            "Undocked successfully with Station #{}: {}",
            station.get_id().value(),
            station.get_name(),
        ),
        Some("status"),
    )?;

    Ok(ship.clone())
}
