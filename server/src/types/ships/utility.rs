
use std::time::Duration;

use glam::Vec2;
use log::info;
use spacetimedb::rand::Rng;

use crate::types::{items::*, ships::timers::CreateShipEnergyAndShieldTimerRow, stellarobjects::{utility::*, *}};

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

pub fn create_ship_instance(ctx: &ReducerContext, ship_type: ShipTypeDefinition, identity: Identity, sobj: StellarObject) -> Result<ShipInstance, String> {
    let dsl = dsl(ctx);

    match dsl.create_ship_instance(
            ship_type.get_id(),
            Some(identity), None, 
            Some(sobj.get_id()), 
            sobj.get_sector_id(), 
            ship_type.max_health.into(),
            ship_type.max_shield.into(),
            ship_type.max_energy.into(),
            0,
            ship_type.cargo_capacity, 
            None, None,
            ctx.timestamp) {
                Ok(ship) => {
                    dsl.create_ship_energy_and_shield_timer(
                        spacetimedb::ScheduleAt::Interval(Duration::from_millis(500).into()), ship.get_id())?;
                    Ok(ship)
                },
                Err(e) => Err(e.to_string())
            }
}

/// Loads cargo into a ship's cargo hold, preferring existing cargo items. It creates new cargo items if necessary, but if it can't it will crate a cargo crate instead.
pub fn load_cargo_into_ship(ctx: &ReducerContext, ship: &mut ShipInstance, item_def: &ItemDefinition, amount: u16) -> Result<(), String> {
    let dsl = dsl(ctx);
    let mut remaining_amount = amount;
    let mut overflow_amount = 0; // How many items could NEVER had fit in the ship and must be made into a crate.
    let units_per_stack = *item_def.get_units_per_stack() as u16;

    if amount == 0 {
        return Err(format!("Tried to load 0 amount of {} into ship #{:?}", item_def.name, ship.get_id()));
    }

    ship.used_cargo_capacity = ship.calculate_used_cargo_space(ctx); // Just go through and make sure everything is ship-shape.
    info!("Attempting to load {}x {} into ship #{} with remaining cargo space of {}v", 
        amount, item_def.name, ship.id, ship.get_remaining_cargo_space());

    // First check how many items can actually fit inside the cargo hold
    let additional_items_that_can_fit = ship.get_remaining_cargo_space() / item_def.volume_per_unit;
    if additional_items_that_can_fit < amount {
        overflow_amount = amount - additional_items_that_can_fit;
        remaining_amount = additional_items_that_can_fit;
        info!("Houston, we have a problem. We can only fit {} more items, but we've been requested to add {}. Sending {} to overflow",
            additional_items_that_can_fit, amount, overflow_amount);
    }

    // Update already existing stacks of the item in the ship's cargo
    if remaining_amount > 0 {
        for mut cargo_item in dsl.get_ship_cargo_items_by_ship_id(ship.get_id()) {
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
                    info!("Found an existing stack of {} {}, filling to up by {}", cargo_item.get_quantity(), item_def.name, remaining_amount);
                    let tmp_amount = remaining_amount;
                    remaining_amount = 0; // We are loading the rest of the amount!
                    cargo_item.get_quantity() + tmp_amount
                };

                // If we got this far, then we're updating the cargo item.
                cargo_item.set_quantity(new_amount);
                info!("Updating cargo item for ship #{:?}: {}x {}", ship.get_id(), new_amount, item_def.name);
                let _ = dsl.update_ship_cargo_item_by_id(cargo_item)?;
            }
        }
    }
    
    // If there's still remaining amount that isn't in the overflow, then that must mean we still have cargo space for them.
    if remaining_amount > 0 {
        info!("Remaining amount to load {}x {} into ship #{} with remaining cargo space of {}v", 
            remaining_amount, item_def.name, ship.id, ship.get_remaining_cargo_space());
        
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
            info!("Creating cargo item for ship #{:?}: {}x {}", ship.get_id(), stack_amount, item_def.name);
            if let Err(e) = dsl.create_ship_cargo_item(ship.get_id(), item_def, stack_amount.into()) {
                info!("Failed to create cargo item for ship {:?}, adding {} to overflow: {}", ship.get_id(), stack_amount, e);
                overflow_amount += stack_amount;
            }
        }
    }
    ship.used_cargo_capacity = ship.calculate_used_cargo_space(ctx); // Just go through and make sure everything is ship-shape.

    if overflow_amount > 0 {
        info!("Not enough cargo space: Remaining {} / Required {}", ship.get_used_cargo_capacity(), item_def.get_volume_per_unit() * overflow_amount);

        // If not enough space, create a cargo crate instead
        create_cargo_crate_nearby_ship(ctx, ship, item_def, overflow_amount)?;
    }

    ship.used_cargo_capacity = ship.calculate_used_cargo_space(ctx); // Just go through and make sure everything is ship-shape FINALLY.
    if ship.used_cargo_capacity > ship.max_cargo_capacity {
        return Err("Despite our best efforts, we ended up with more cargo used than is maximum!".to_string());
    }
    let _ = dsl.update_ship_instance_by_id(ship.clone())?;
    Ok(())
}

/// Crates a cargo crate nearby the given ship instance. If the ship instance doesn't have an stellarObject, it'll place it randomly in its last known sector.
fn create_cargo_crate_nearby_ship(ctx: &ReducerContext, ship: &ShipInstance, item_def: &ItemDefinition, quantity: u16) -> Result<(), String> {
    let dsl = dsl(ctx);

    let mut pos = Vec2::ZERO;
    if let Some(ship_sobj) = ship.get_sobj_id() {
        if let Some(transform) = dsl.get_sobj_internal_transform_by_sobj_id(ship_sobj) {
            pos = transform.to_vec2();
        }
    }
    if pos == Vec2::ZERO {
        pos = Vec2::new(
            ctx.rng().gen_range(-2048.0..2048.0),
            ctx.rng().gen_range(-2048.0..2048.0)
        );
    }
    let sobj = create_sobj_with_random_velocity(
        ctx, StellarObjectKinds::CargoCrate, &ship.get_current_sector_id(), pos.x, pos.y, 0.125);
    if sobj.is_err() {
        return Err(format!("Failed to create cargo crate for ship {:?}: {}", ship.get_id(), sobj.unwrap_err()));
    }
    let _ = dsl.create_cargo_crate(
        ship.get_current_sector_id(),
        sobj.unwrap().get_id(),
        item_def.get_id(),
        quantity,
        None, // TODO: Set a despawn duration
        None
    )?;
    Ok(())
}