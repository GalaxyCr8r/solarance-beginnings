use std::f32::consts::PI;
use std::time::Duration;

use log::info;
use solarance_shared::{MovementState, Vec2};
use spacetimedb::rand::Rng;
use spacetimedb::*;
use spacetimedsl::*;

use crate::logic::cargo_crates::attempt_to_pickup_cargo_crate;
use crate::logic::stellarobjects::stellar_object_creation::create_sobj;
use crate::tables::global_config::*;
use crate::tables::items::*;
use crate::tables::players::get_player_ship_and_sobj;
use crate::tables::players::PlayerId;
use crate::tables::messages::*;
use crate::tables::ships::*;
use crate::tables::stellarobjects::*;
use crate::utility::*;

///////////////////////////////////////////////////////////
/// Reducers
///

/// Allows a player to attempt to pickup cargo into their ship. Defaults to finding the sender's current ship.
#[spacetimedb::reducer]
pub fn try_to_pickup_crate(
    ctx: &ReducerContext,
    cargo_crate_id: CargoCrateId,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let player_id = PlayerId::new(ctx.sender());
    let (ship_object, _) = get_player_ship_and_sobj(&dsl, &player_id)?;
    let cargo_crate = dsl.get_cargo_crate_by_id(cargo_crate_id)?;

    let item_def: ItemDefinition = dsl.get_item_definition_by_id(cargo_crate.get_item_id())?;
    let status: ShipStatus = dsl.get_ship_status_by_id(&ship_object)?;

    attempt_to_pickup_cargo_crate(&dsl, &cargo_crate, &item_def, &status)
}

/// Allows a player to jettison cargo from their ship into space as a cargo crate.
/// Validates ship ownership and cargo availability before creating the crate.
#[spacetimedb::reducer]
pub fn jettison_cargo_from_ship(
    ctx: &ReducerContext,
    ship_id: u64,
    ship_cargo_id: u64,
    amount: u16,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let ship = dsl.get_ship_by_id(ShipId::new(ship_id))?;

    is_server_or_sobj_owner(&dsl, Some(ship.get_sobj_id()))?;

    let mut ship_cargo = dsl.get_ship_cargo_item_by_id(ShipCargoItemId::new(ship_cargo_id))?;
    let item_def = dsl.get_item_definition_by_id(ship_cargo.get_item_id())?;

    // Does the ship actually have that amount of item?
    if ship_cargo.get_quantity() < &amount {
        return Err(format!(
            "Failed to verify that the cargo item actually had the amount requested to yeet."
        ));
    } else if ship_cargo.get_quantity() == &amount {
        dsl.delete_ship_cargo_item_by_id(&ship_cargo)?;
    } else {
        ship_cargo.set_quantity(*ship_cargo.get_quantity() - amount);
        dsl.update_ship_cargo_item_by_id(ship_cargo)?;
    }

    // Player-initiated jettison: toss in the ship's current heading with no
    // momentum inheritance (MVP simplification per movement_system_plan).
    let toss_dir_radians = ship.get_movement().rotation;
    let origin = ship.get_movement().pos;
    create_cargo_crate_at_position(
        ctx,
        &dsl,
        &ship.get_sector_id(),
        origin,
        toss_dir_radians,
        &item_def,
        amount,
    )?;

    send_direct_server_info(
        &dsl,
        &ship.get_player_id(),
        format!("Jettioned successfully {}x {}", amount, item_def.get_name()),
    )?;

    Ok(())
}

//////////////////////////////////////////
/// Utility functions

/// Removes cargo from a ship's cargo hold. Errors if the ship doesn't have enough of the item.
pub fn remove_cargo_from_ship<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    ship_status: &mut ShipStatus,
    item_def: &ItemDefinition,
    amount: u16,
) -> Result<(), String> {
    let mut remaining_amount = amount;

    if amount == 0 {
        return Err(format!(
            "Tried to remove 0 amount of {} from ship #{:?}",
            item_def.get_name(),
            ship_status.get_id()
        ));
    }

    ship_status.set_used_cargo_capacity(ship_status.calculate_used_cargo_space(dsl)); // Just go through and make sure everything is ship-shape.
    info!(
        "Attempting to remove {}x {} ({}v) from ship #{} with remaining cargo space of {}v",
        amount,
        item_def.get_name(),
        amount * item_def.get_volume_per_unit(),
        ship_status.get_id().value(),
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

                ship_status.set_used_cargo_capacity(ship_status.calculate_used_cargo_space(dsl));
                let _ = dsl.update_ship_status_by_id(ship_status.clone())?;

                return Ok(());
            }
        }
    }
    if remaining_amount > 0 {
        return Err(format!(
            "Ship #{} does not have enough {} to remove: requested {}, available 0",
            ship_status.get_id().value(),
            item_def.get_name(),
            remaining_amount
        ));
    }

    // Update ship status after removing cargo items
    ship_status.set_used_cargo_capacity(ship_status.calculate_used_cargo_space(dsl));
    dsl.update_ship_status_by_id(ship_status.clone())?;

    Ok(())
}

/// Loads cargo into a ship's cargo hold, preferring existing cargo items.
/// It creates new cargo items if necessary, but if it can't it will create
/// a cargo crate instead if create_a_crate_if_failed is true and Ship
/// points to a Ship and not a Ship row.
pub fn attempt_to_load_cargo_into_ship<T: spacetimedsl::WriteContext>(
    ctx: &spacetimedb::ReducerContext,
    dsl: &DSL<T>,
    ship_status: &mut ShipStatus,
    ship_id: &ShipId,
    item_def: &ItemDefinition,
    amount: u16,
    create_a_crate_if_failed: bool,
) -> Result<(), String> {
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

    ship_status.set_used_cargo_capacity(ship_status.calculate_used_cargo_space(dsl)); // Just go through and make sure everything is ship-shape.
    info!(
        "Attempting to load {}x {} ({}v) into ship #{} with remaining cargo space of {}v",
        amount,
        item_def.get_name(),
        amount * item_def.get_volume_per_unit(),
        ship_status.get_id().value(),
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
            ship_status.get_max_cargo_capacity()
        );
    }

    // Update already existing stacks of the item in the ship's cargo
    if remaining_amount > 0 {
        for mut cargo_item in dsl.get_ship_cargo_items_by_ship_id(ship_status.get_id()) {
            if cargo_item.get_item_id() == item_def.get_id() {
                // If it exists, then try to fill the stack.
                if *cargo_item.get_quantity() == units_per_stack {
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
            ship_status.get_id().value(),
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
            if let Err(e) = dsl.create_ship_cargo_item(CreateShipCargoItem {
                ship_id: ship_status.get_id(),
                item_id: item_def.get_id(),
                quantity: stack_amount as u16,
            }) {
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
    ship_status.set_used_cargo_capacity(ship_status.calculate_used_cargo_space(&dsl)); // Just go through and make sure everything is ship-shape.

    if overflow_amount > 0 {
        info!(
            "Not enough cargo space: Remaining {} / Required {}",
            ship_status.get_used_cargo_capacity(),
            item_def.get_volume_per_unit() * overflow_amount
        );

        // If not enough space, check if we create a cargo crate instead
        if create_a_crate_if_failed {
            let ship_instance = dsl.get_ship_by_id(ship_id)?;
            if *ship_instance.get_location() == ShipLocation::Sector {
                // Only spawn if the ship is in a sector
                create_cargo_crate_nearby_ship(
                    ctx,
                    &dsl,
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

    ship_status.set_used_cargo_capacity(ship_status.calculate_used_cargo_space(&dsl)); // Just go through and make sure everything is ship-shape FINALLY.
    if ship_status.get_used_cargo_capacity() > ship_status.get_max_cargo_capacity() {
        return Err(
            "Despite our best efforts, we ended up with more cargo used than is maximum!"
                .to_string(),
        );
    }
    dsl.update_ship_status_by_id(ship_status.clone())?;

    send_direct_server_info(
        dsl,
        &ship_status.get_player_id(),
        format!("Loaded successfully {}x {}", amount, item_def.get_name()),
    )?;

    Ok(())
}

/// Spawns a cargo crate at `origin` with a single, well-defined toss vector.
/// Reads `cargo_crate_toss_speed`, `cargo_crate_brake_rate`, etc. from
/// `global_config` so designers can retune crate physics without touching code.
///
/// This is the single sanctioned creation point for crates — callers like
/// `jettison_cargo_from_ship` (toss direction = ship's heading) and the
/// mining-overflow path in `attempt_to_load_cargo_into_ship` (random direction)
/// both funnel through it.
pub fn create_cargo_crate_at_position<T: spacetimedsl::WriteContext>(
    ctx: &spacetimedb::ReducerContext,
    dsl: &DSL<T>,
    sector_id: &crate::tables::sectors::SectorId,
    origin: Vec2,
    toss_dir_radians: f32,
    item_def: &ItemDefinition,
    quantity: u16,
) -> Result<(), String> {
    let config = dsl.get_global_config_by_id(GlobalConfigId::new(0))?;

    // Sample toss speed and brake rate from the configured ranges. The
    // variance is symmetric (±variance) so designers can keep the mean tied
    // to `*_speed` / `*_rate` directly.
    let toss_speed_variance = *config.get_cargo_crate_toss_speed_variance();
    let toss_speed = *config.get_cargo_crate_toss_speed()
        + ctx
            .rng()
            .gen_range(-toss_speed_variance..=toss_speed_variance);
    let brake_variance = *config.get_cargo_crate_brake_rate_variance();
    let brake_rate = *config.get_cargo_crate_brake_rate()
        + ctx.rng().gen_range(-brake_variance..=brake_variance);
    // A small random spin for visual flavor — the always-on angular damping
    // inside predict_movement bleeds it to zero on its own.
    let spin = ctx.rng().gen_range(-PI..=PI);

    let now_micros = dsl.ctx().timestamp()?.to_micros_since_unix_epoch();
    let movement = MovementState {
        pos: origin,
        velocity: toss_speed.max(0.0),
        rotation: toss_dir_radians,
        angular_velocity: spin,
        last_update_time: now_micros,
        acceleration: -brake_rate.abs(), // braking → negative
        angular_acceleration: 0.0,
        // Cap at the chosen toss speed (velocity only decreases, so this is
        // a free guard against bad input).
        max_speed: toss_speed.max(0.0),
        max_turn_rate: *config.get_cargo_crate_max_turn_rate(),
    };

    let new_sobj = create_sobj(dsl, StellarObjectKinds::CargoCrate, sector_id)?;

    dsl.create_cargo_crate(CreateCargoCrate {
        sobj_id: new_sobj.get_id(),
        current_sector_id: sector_id.clone(),
        item_id: item_def.get_id(),
        quantity,
        despawn_ts: Some(
            dsl.ctx()
                .timestamp()?
                .checked_add(TimeDuration::from_duration(Duration::from_secs(
                    *config.get_cargo_crate_ttl_secs(),
                )))
                .unwrap(),
        ),
        gfx_key: None,
        movement,
    })?;
    Ok(())
}

/// Overflow-path crate spawn: when mined cargo can't fit, we drop a crate at
/// the ship's last-known position with a random toss direction (per
/// movement_system_plan §"Cargo jettison from asteroid").
pub fn create_cargo_crate_nearby_ship<T: spacetimedsl::WriteContext>(
    ctx: &spacetimedb::ReducerContext,
    dsl: &DSL<T>,
    ship_sobj: &StellarObjectId,
    item_def: &ItemDefinition,
    quantity: u16,
) -> Result<(), String> {
    let ship = dsl
        .get_ships_by_sobj_id(ship_sobj)
        .next()
        .ok_or_else(|| {
            format!(
                "Couldn't find a Ship for sobj_id {} when spawning overflow crate",
                ship_sobj.value()
            )
        })?;

    let origin = ship.get_movement().pos;
    let random_dir = ctx.rng().gen_range(-PI..=PI);

    info!(
        "Spawning overflow crate from ship #{} at ({:.0},{:.0})",
        ship.get_id().value(),
        origin.x,
        origin.y
    );

    create_cargo_crate_at_position(
        ctx,
        dsl,
        &ship.get_sector_id(),
        origin,
        random_dir,
        item_def,
        quantity,
    )
}
