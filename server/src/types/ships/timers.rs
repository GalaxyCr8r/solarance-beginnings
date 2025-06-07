use std::time::Duration;

use spacetimedb::*;
use spacetimedsl::{dsl, Wrapper};

use crate::types::{asteroids::GetAsteroidRowOptionBySobjId, items::{utility::get_item_definition, CreateCargoCrateRow, GetItemDefinitionRowOptionById, ItemMetadata}, ships::utility::{load_cargo_into_ship, same_sector_from_ids}, utility::try_server_only};

use super::{*};

#[dsl(plural_name = ship_mining_timers)]
#[spacetimedb::table(name = ship_mining_timer, scheduled(ship_mining_timer_reducer))]
pub struct ShipMiningTimer {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[index(btree)]
    #[wrapped(path = StellarObjectId)]
    pub ship_sobj_id: u64, // FK: StellarObject

    #[wrapped(path = StellarObjectId)]
    pub asteroid_sobj_id: u64, // FK: StellarObject

    pub mining_progress: f32, // How much of the asteroid has been mined (0 to 1.0)
}

/// Adds a cargo item to a ship's cargo after a delay. If there isn't room, it creates a cargo crate instead.
#[dsl(plural_name = ship_add_cargo_timers)]
#[spacetimedb::table(name = ship_add_cargo_timer, scheduled(ship_add_cargo_timer_reducer))]
pub struct ShipAddCargoTimer {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[index(btree)]
    #[wrapped(path = ShipInstanceId)]
    pub ship_inst_id: u64, // FK: ShipInstance

    #[wrapped(path = ItemDefinitionId)]
    pub item_id: u32, // FK: Item Definition

    pub amount: u16,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

pub fn create_mining_timer_for_ship(ctx: &ReducerContext, ship_sobj_id: &StellarObjectId, asteroid_sobj_id: &StellarObjectId) -> Result<ShipMiningTimer, String> {
    let dsl = dsl(ctx);

    // Check if the ship is already mining and remove those timers.
    dsl.get_ship_mining_timers_by_ship_sobj_id(ship_sobj_id).for_each(|timer| {
        dsl.delete_ship_mining_timer_by_scheduled_id(timer.get_scheduled_id());
    });

    // Check if the ship and asteroid are in the same sector
    if !same_sector_from_ids(ctx, &ship_sobj_id, &asteroid_sobj_id) {
        return Err(format!("What are you trying to mine? {} and {} are in different sectors", ship_sobj_id.clone().value(), asteroid_sobj_id.clone().value()));
    }

    Ok(dsl.create_ship_mining_timer(
        spacetimedb::ScheduleAt::Interval(Duration::from_secs(1).into()), 
        ship_sobj_id, 
        asteroid_sobj_id,
        0.0
    )?)
}

pub fn create_timer_to_add_cargo_to_ship(ctx: &ReducerContext, ship_inst_id: ShipInstanceId, item_id: ItemDefinitionId, amount: u16) -> Result<ShipAddCargoTimer, String> {
    let dsl = dsl(ctx);

    Ok(dsl.create_ship_add_cargo_timer(
        spacetimedb::ScheduleAt::Interval(Duration::from_secs(1).into()), 
        ship_inst_id, 
        item_id,
        amount
    )?)
}

//////////////////////////////////////////////////////////////
// Reducers
//////////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn ship_mining_timer_reducer(ctx: &ReducerContext, mut timer: ShipMiningTimer) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    if let Some(ship_object) = dsl.get_ship_object_by_sobj_id(timer.get_ship_sobj_id()) {
        if let Some(asteroid_object) = dsl.get_asteroid_by_sobj_id(timer.get_asteroid_sobj_id()) {
            // Get the volume of the asteroid's item type
            let item_def = get_item_definition(ctx, asteroid_object.resource_item_id).ok_or("Failed to get item definition")?;

            // Do the logic to determine speed of mining based on mining equipment, item id, etc.
            let mut energy_consumption = 0.0f32;
            let mut mining_speed = 1.0f32;
            for item in dsl.get_ship_equipment_slots_by_ship_id(ship_object.get_ship_id()) {
                if item.slot_type == EquipmentSlotType::MiningLaser {
                    if let Some(laser_def) = dsl.get_item_definition_by_id(item.get_item_id()) {
                        laser_def.get_metadata().iter().for_each(|metadata| {
                            match metadata {
                                ItemMetadata::MiningSpeedMultiplier(mul) => {
                                    mining_speed *= mul;
                                },
                                ItemMetadata::EnergyConsumption(consumption) => {
                                    energy_consumption += consumption;
                                }
                                _ => {}
                            }
                        });
                    }
                }
            }

            // Find the ship instance so we can check energy and update mining progress
            if let Some(mut ship_instance) = dsl.get_ship_instance_by_id(ship_object.get_ship_id()) {
                if ship_instance.get_energy() < &energy_consumption {
                    return Err(format!("Ship {:?} does not have enough energy to mine. Req: {}, Current: {}", ship_object.get_sobj_id(), energy_consumption, ship_instance.get_energy()));
                }

                ship_instance.set_energy(ship_instance.get_energy() - energy_consumption);
                timer.set_mining_progress(timer.get_mining_progress() + mining_speed);
                
                let get_volume_per_unit = &(*item_def.get_volume_per_unit() as f32);
                if timer.get_mining_progress() >= get_volume_per_unit {
                    let diff = timer.get_mining_progress() / get_volume_per_unit;
                    create_timer_to_add_cargo_to_ship(ctx,ship_instance.get_id(), item_def.get_id(), diff.floor() as u16)?;
                
                    timer.set_mining_progress(timer.get_mining_progress() - diff.floor());
                }

                dsl.update_ship_instance_by_id(ship_instance)?;
                dsl.update_ship_mining_timer_by_scheduled_id(timer)?;
                Ok(())
            } else {
                dsl.delete_ship_mining_timer_by_scheduled_id(timer.get_scheduled_id());
                Err(format!("Failed to find ship instance object for mining timer: {:?} Removed timer.", ship_object.get_ship_id()))
            }
        } else {
            dsl.delete_ship_mining_timer_by_scheduled_id(timer.get_scheduled_id());
            Err(format!("Failed to find asteroid object for mining timer: {:?} Removed timer.", timer.get_ship_sobj_id()))
        }
    } else {
        dsl.delete_ship_mining_timer_by_scheduled_id(timer.get_scheduled_id());
        Err(format!("Failed to find ship object for mining timer: {:?} Removed timer.", timer.get_ship_sobj_id()))
    }
}

#[spacetimedb::reducer]
pub fn ship_add_cargo_timer_reducer(ctx: &ReducerContext, timer: ShipAddCargoTimer) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    if let Some(mut ship_instance) = dsl.get_ship_instance_by_id(timer.get_ship_inst_id()) {
        // Get the item definition
        let item_def = get_item_definition(ctx, timer.item_id)
            .ok_or(format!("Failed to get item def for {}", timer.item_id))?;

        // Check if the ship has enough cargo space
        if *ship_instance.get_cargo_capacity() >= *item_def.get_volume_per_unit() * timer.amount {
            load_cargo_into_ship(ctx, &mut ship_instance, &item_def, timer.amount)?;
        } else {
            // If not enough space, create a cargo crate instead
            let _ = dsl.create_cargo_crate(
                    ship_instance.get_current_sector_id(),
                    ship_instance.get_sobj_id().unwrap(), // TODO: We can't assume this here, but we do for now.
                    item_def.get_id(),
                    timer.amount,
                    None, // TODO: Set a despawn duration
                    None
            )?;
        }
        
        Ok(())
    } else {
        Err(format!("Failed to find ship instance for cargo timer: {:?}. Removing timer thus destroying the item(s).", timer.get_ship_inst_id()))
    }
}