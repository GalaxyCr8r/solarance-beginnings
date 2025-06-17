
use std::time::Duration;

use glam::Vec2;
use log::info;
use spacetimedb::{rand::Rng, TimeDuration};

use crate::types::{factions::FactionDefinitionId, ships::timers::CreateShipEnergyAndShieldTimerRow, stellarobjects::{utility::*, *}};

use super::{*};

pub fn same_sector_from_ids(ctx: &ReducerContext, id1: &StellarObjectId, id2: &StellarObjectId) -> bool {
    let dsl = dsl(ctx);

    if let Some(sobj1) = dsl.get_stellar_object_by_id(id1) {
        if let Some(sobj2) = dsl.get_stellar_object_by_id(id2) {
            return sobj1.get_sector_id() == sobj2.get_sector_id();
        }
    }
    false
}

pub fn create_ship_from_sobj(ctx: &ReducerContext, ship_type: ShipTypeDefinition, identity: Identity, sobj: StellarObject) -> Result<(Ship, ShipStatus), String> {
    let dsl = dsl(ctx);

    let ship_global = dsl.create_ship_global()?;

    let ship = match dsl.create_ship(
        &ship_global,
        ship_type.get_id(),
        &sobj,
        sobj.get_sector_id(),
        identity,
        FactionDefinitionId::new(0)) {
            Ok(ship) => {
                dsl.create_ship_energy_and_shield_timer(
                    spacetimedb::ScheduleAt::Interval(Duration::from_millis(500).into()), 
                    ship.get_id(), 
                    ship_type.get_id())?;
                Ok(ship)
            },
            Err(e) => Err(e.to_string())
        }?;

    let ship_status = dsl.create_ship_status(&ship_global, 
        sobj.get_sector_id(), 
        identity, 
        ship_type.max_health as f32,
        ship_type.max_shields as f32,
        ship_type.max_energy as f32,
        0,
        ship_type.cargo_capacity,
        None)?;

    return Ok((ship, ship_status));
}

pub fn create_ship_docked_at_station(ctx: &ReducerContext, ship_type: ShipTypeDefinition, identity: Identity, station: Station) -> Result<(DockedShip, ShipStatus), String> {
    let dsl = dsl(ctx);

    let ship_global = dsl.create_ship_global()?;

    let ship = match dsl.create_docked_ship(
        &ship_global,
        ship_type.get_id(),
        &station,
        station.get_sector_id(),
        identity,
        FactionDefinitionId::new(0)) {
            Ok(ship) => {
                dsl.create_ship_energy_and_shield_timer(
                    spacetimedb::ScheduleAt::Interval(Duration::from_millis(500).into()), 
                    ship.get_id(), 
                    ship_type.get_id())?;
                Ok(ship)
            },
            Err(e) => Err(e.to_string())
        }?;

    let ship_status = dsl.create_ship_status(&ship_global, 
        station.get_sector_id(), 
        identity, 
        ship_type.max_health as f32,
        ship_type.max_shields as f32,
        ship_type.max_energy as f32,
        0,
        ship_type.cargo_capacity,
        None)?;

    return Ok((ship, ship_status));
}

/// Docks the given Ship to the given station it is docking at and returns the new DockedShip row.
pub fn dock_ship(ctx: &ReducerContext, docking_ship: Ship, station: Station) -> Result<DockedShip, String> {
    todo!()
}

/// Undocks the given DockedShip on top of the station it was docked at and returns the new Ship row.
pub fn undock_ship(ctx: &ReducerContext, docked_ship: DockedShip) -> Result<Ship, String> {
    todo!()
}

/// Loads cargo into a ship's cargo hold, preferring existing cargo items. 
/// It creates new cargo items if necessary, but if it can't it will crate a cargo crate instead.
pub fn attempt_to_load_cargo_into_ship(ctx: &ReducerContext, 
        ship_status: &mut ShipStatus, 
        ship_object: &Ship,
        item_def: &ItemDefinition, 
        amount: u16) -> Result<(), String> {
    let dsl = dsl(ctx);
    let mut remaining_amount = amount;
    let mut overflow_amount = 0; // How many items could NEVER had fit in the ship and must be made into a crate.
    let units_per_stack = *item_def.get_units_per_stack() as u16;

    if amount == 0 {
        return Err(format!("Tried to load 0 amount of {} into ship #{:?}", item_def.name, ship_status.get_id()));
    }

    ship_status.used_cargo_capacity = ship_status.calculate_used_cargo_space(ctx); // Just go through and make sure everything is ship-shape.
    info!("Attempting to load {}x {} ({}v) into ship #{} with remaining cargo space of {}v", 
        amount, item_def.name, amount * item_def.volume_per_unit, ship_status.id, ship_status.get_remaining_cargo_space());

    // First check how many items can actually fit inside the cargo hold
    let additional_items_that_can_fit = ship_status.get_remaining_cargo_space() / item_def.volume_per_unit;
    if additional_items_that_can_fit < amount {
        overflow_amount = amount - additional_items_that_can_fit;
        remaining_amount = additional_items_that_can_fit;
        info!("WARN: We can only fit {} more items, but we've been requested to add {}. Sending {} to overflow",
            additional_items_that_can_fit, amount, overflow_amount);
        info!("Expected final used cargo capacity: {} / {}", ship_status.get_used_cargo_capacity() + (additional_items_that_can_fit * item_def.volume_per_unit), ship_status.max_cargo_capacity)
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
                    info!("Found an existing stack of {} {}, filling to max amount...", cargo_item.get_quantity(), item_def.name);
                    remaining_amount = cargo_item.get_quantity() + remaining_amount - units_per_stack;
                    units_per_stack
                } else {
                    info!("Found an existing stack of {} {}, adding {}", cargo_item.get_quantity(), item_def.name, remaining_amount);
                    let tmp_amount = remaining_amount;
                    remaining_amount = 0; // We are loading the rest of the amount!
                    cargo_item.get_quantity() + tmp_amount
                };

                // If we got this far, then we're updating the cargo item.
                cargo_item.set_quantity(new_amount);
                info!("Updating cargo item for ship #{:?}: {}x {}", ship_status.get_id(), new_amount, item_def.name);
                let _ = dsl.update_ship_cargo_item_by_id(cargo_item)?;
            }
        }
    }
    
    // If there's still remaining amount that isn't in the overflow, then that must mean we still have cargo space for them.
    if remaining_amount > 0 {
        info!("Remaining amount to load {}x {} into ship #{} with remaining cargo space of {}v", 
            remaining_amount, item_def.name, ship_status.id, ship_status.get_remaining_cargo_space());
        
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
            info!("Creating cargo item for ship #{:?}: {}x {}", ship_status.get_id(), stack_amount, item_def.name);
            if let Err(e) = dsl.create_ship_cargo_item(ship_status.get_id(), item_def, stack_amount.into()) {
                info!("Failed to create cargo item for ship {:?}, adding {} to overflow: {}", ship_status.get_id(), stack_amount, e);
                overflow_amount += stack_amount;
            }
        }
    }
    ship_status.used_cargo_capacity = ship_status.calculate_used_cargo_space(ctx); // Just go through and make sure everything is ship-shape.

    if overflow_amount > 0 {
        info!("Not enough cargo space: Remaining {} / Required {}", ship_status.get_used_cargo_capacity(), item_def.get_volume_per_unit() * overflow_amount);

        // If not enough space, create a cargo crate instead
        create_cargo_crate_nearby_ship(ctx, &ship_object.get_sobj_id(), item_def, overflow_amount)?;
    }

    ship_status.used_cargo_capacity = ship_status.calculate_used_cargo_space(ctx); // Just go through and make sure everything is ship-shape FINALLY.
    if ship_status.used_cargo_capacity > ship_status.max_cargo_capacity {
        return Err("Despite our best efforts, we ended up with more cargo used than is maximum!".to_string());
    }
    let _ = dsl.update_ship_status_by_id(ship_status.clone())?;
    Ok(())
}

/// Crates a cargo crate nearby the given stellar object if it exists, 
/// otherwise it'll place it randomly in its last known sector.
pub fn create_cargo_crate_nearby_ship(ctx: &ReducerContext, ship_sobj: &StellarObjectId, item_def: &ItemDefinition, quantity: u16) -> Result<(), String> {
    let dsl = dsl(ctx);

    let sobj = dsl.get_stellar_object_by_id(ship_sobj).ok_or(
        format!("Could not find Stellar Object #{} for a Ship!", ship_sobj.value()) // TODO error message here
    )?;
    let pos= if let Some(transform) = dsl.get_sobj_internal_transform_by_sobj_id(ship_sobj) {
        transform.to_vec2()
    } else {
        info!("Could not find ship's stellar object transform, placing randomly...");
        Vec2::new(
            ctx.rng().gen_range(-2048.0..2048.0),
            ctx.rng().gen_range(-2048.0..2048.0)
        )
    };

    let new_sobj = create_sobj_with_random_velocity(
        ctx, StellarObjectKinds::CargoCrate, &sobj.get_sector_id(), pos.x, pos.y, 0.125, Some(0.9995))?;

    info!("Created cargo crate in sector #{:?} at {}, {}!", &sobj.get_sector_id(), pos.x, pos.y);
    
    let _ = dsl.create_cargo_crate(
        sobj.get_sector_id(),
        new_sobj.get_id(),
        item_def.get_id(),
        quantity,
        ctx.timestamp.checked_add(TimeDuration::from_duration(Duration::from_secs(24 * 60 * 60))), // TODO cargo crate timer to despawn them
        None
    )?;
    Ok(())
}