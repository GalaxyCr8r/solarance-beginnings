use std::time::Duration;

use log::info;
use spacetimedb::*;
use spacetimedsl::{dsl, Wrapper};

use crate::types::{
    asteroids::*, common::utility::try_server_only, items::utility::*,
    server_messages::utility::send_info_message, ships::utility::*, stellarobjects::*,
};

use super::*;

#[dsl(plural_name = ship_status_timers)]
#[spacetimedb::table(name = ship_status_timer, scheduled(ship_status_timer_reducer))]
pub struct ShipStatusTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[unique]
    #[use_wrapper(path = ShipGlobalId)]
    /// FK to Ship
    pub ship_id: u64,

    #[use_wrapper(path = ShipTypeDefinitionId)]
    /// FK to Ship Type
    pub ship_type_id: u32,
}

#[dsl(plural_name = ship_mining_timers)]
#[spacetimedb::table(name = ship_mining_timer, scheduled(ship_mining_timer_reducer))]
pub struct ShipMiningTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[index(btree)]
    #[use_wrapper(path = StellarObjectId)]
    /// FK to StellarObject
    pub ship_sobj_id: u64,

    #[use_wrapper(path = StellarObjectId)]
    /// FK to StellarObject
    pub asteroid_sobj_id: u64,

    pub mining_progress: f32, // How much of the asteroid has been mined (0 to 1.0)
}

/// Adds a cargo item to a ship's cargo after a delay. If there isn't room, it creates a cargo crate instead.
#[dsl(plural_name = ship_add_cargo_timers)]
#[spacetimedb::table(name = ship_add_cargo_timer, scheduled(ship_add_cargo_timer_reducer))]
pub struct ShipAddCargoTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[index(btree)]
    #[use_wrapper(path = ShipGlobalId)]
    /// FK to Ship
    pub ship_id: u64,

    #[use_wrapper(path = ItemDefinitionId)]
    /// FK to Item Definition
    pub item_id: u32,

    pub amount: u16,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

pub fn create_status_timer_for_ship(
    ctx: &ReducerContext,
    ship_id: &ShipGlobalId,
    type_id: &ShipTypeDefinitionId,
) -> Result<ShipStatusTimer, String> {
    let dsl = dsl(ctx);

    let timer = dsl.create_ship_status_timer(
        spacetimedb::ScheduleAt::Interval(Duration::from_millis(500).into()),
        ship_id,
        type_id,
    )?;

    Ok(timer)
}

pub fn create_mining_timer_for_ship(
    ctx: &ReducerContext,
    ship_sobj_id: &StellarObjectId,
    asteroid_sobj_id: &StellarObjectId,
) -> Result<ShipMiningTimer, String> {
    let dsl = dsl(ctx);

    // Check if the ship is already mining and remove those timers.
    for timer in dsl.get_ship_mining_timers_by_ship_sobj_id(ship_sobj_id) {
        dsl.delete_ship_mining_timer_by_id(timer.get_id())?;
    }

    // Check if the ship and asteroid are in the same sector
    if !same_sector_from_ids(ctx, &ship_sobj_id, &asteroid_sobj_id) {
        return Err(format!(
            "What are you trying to mine? {} and {} are in different sectors",
            ship_sobj_id.clone().value(),
            asteroid_sobj_id.clone().value()
        ));
    }

    Ok(dsl.create_ship_mining_timer(
        spacetimedb::ScheduleAt::Interval(Duration::from_secs(3).into()),
        ship_sobj_id,
        asteroid_sobj_id,
        0.0,
    )?)
}

pub fn create_timer_to_add_cargo_to_ship(
    ctx: &ReducerContext,
    ship_id: ShipGlobalId,
    item_id: ItemDefinitionId,
    amount: u16,
) -> Result<ShipAddCargoTimer, String> {
    let dsl = dsl(ctx);

    Ok(dsl.create_ship_add_cargo_timer(
        spacetimedb::ScheduleAt::Interval(Duration::from_secs(1).into()),
        ship_id,
        item_id,
        amount,
    )?)
}

//////////////////////////////////////////////////////////////
// Timer Reducers
//////////////////////////////////////////////////////////////

/// Scheduled reducer that handles ship status updates like shield and energy regeneration.
/// Runs every 500ms to gradually restore shields and energy based on ship type specifications.
#[spacetimedb::reducer]
pub fn ship_status_timer_reducer(
    ctx: &ReducerContext,
    timer: ShipStatusTimer,
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    // Get ship rows
    let ship_type = dsl.get_ship_type_definition_by_id(timer.get_ship_type_id())?;
    let mut ship_status = dsl.get_ship_status_by_id(timer.get_ship_id())?;

    // TODO: Grab shield regen from attached shield modules and the current ship type
    if ship_status.shields < (ship_type.max_shields as f32) {
        ship_status.shields += 0.525175;
    }
    if ship_status.energy > (ship_type.max_shields as f32) {
        ship_status.shields = ship_type.max_shields as f32;
    }

    // TODO: Grab energy regen from attached special modules and the current ship type
    if ship_status.energy < (ship_type.max_energy as f32) {
        ship_status.energy += 0.1275;
    }
    if ship_status.energy > (ship_type.max_energy as f32) {
        ship_status.energy = ship_type.max_energy as f32;
    }

    dsl.update_ship_status_by_id(ship_status)?;

    Ok(())
}

/// Scheduled reducer that processes ship mining operations against asteroids.
/// Runs every 3 seconds to extract resources based on mining equipment and energy consumption.
#[spacetimedb::reducer]
pub fn ship_mining_timer_reducer(
    ctx: &ReducerContext,
    mut timer: ShipMiningTimer,
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    let ship_object = dsl.get_ship_by_sobj_id(timer.get_ship_sobj_id())?;
    let mut asteroid_object = dsl.get_asteroid_by_id(timer.get_asteroid_sobj_id())?;

    if *asteroid_object.get_current_resources() == 0 {
        dsl.delete_ship_mining_timer_by_id(timer.get_id())?;

        let _ = dsl.delete_stellar_object_by_id(&asteroid_object.get_id());

        let _ = send_info_message(
            ctx,
            &ship_object.get_player_id(),
            "Targetted asteroid exhausted!".to_string(),
            Some("mining"),
        );
        info!(
            "Asteroid #{:?} exhausted of resources! Timer and Asteroid deleted",
            asteroid_object.get_id()
        );
        return Ok(());
    }

    // Get the volume of the asteroid's item type
    let item_def = get_item_definition(ctx, asteroid_object.get_resource_item_id().value())?;

    // Do the logic to determine speed of mining based on mining equipment, item id, etc.
    let mut energy_consumption = 1.0f32;
    let mut mining_speed = 1.0f32;
    for item in dsl.get_ship_equipment_slots_by_ship_id(ship_object.get_id()) {
        if item.slot_type == EquipmentSlotType::MiningLaser {
            let laser_def = dsl.get_item_definition_by_id(item.get_item_id())?;
            laser_def
                .get_metadata()
                .iter()
                .for_each(|metadata| match metadata {
                    ItemMetadata::MiningSpeedMultiplier(mul) => {
                        mining_speed *= mul;
                    }
                    ItemMetadata::EnergyConsumption(consumption) => {
                        energy_consumption += consumption;
                    }
                    _ => {}
                });
        }
    }

    // Find the ship instance so we can check energy and update mining progress
    let mut ship_status =
        dsl.get_ship_status_by_id(ship_object.get_id())
            .or_else(|_stdsl_error| {
                dsl.delete_ship_mining_timer_by_id(timer.get_id())?;
                Err(format!(
                    "Failed to find ship instance object for mining timer: {:?} Removed timer.",
                    ship_object.get_id()
                ))
            })?;

    if ship_status.get_energy() < &energy_consumption {
        let _ = send_info_message(
            ctx,
            &ship_object.get_player_id(),
            format!(
                "Your ship does not have enough energy to mine. {} energy / {} required",
                ship_status.get_energy(),
                energy_consumption
            ),
            Some("mining"),
        );
        return Err(format!(
            "Ship {:?} does not have enough energy to mine. Req: {}, Current: {}",
            ship_object.get_id(),
            energy_consumption,
            ship_status.get_energy()
        ));
    }

    ship_status.set_energy(ship_status.get_energy() - energy_consumption);
    timer.set_mining_progress(timer.get_mining_progress() + mining_speed);

    let get_volume_per_unit = &(*item_def.get_volume_per_unit() as f32);
    if timer.get_mining_progress() >= get_volume_per_unit {
        let mut diff = timer.get_mining_progress() / get_volume_per_unit;

        if diff > (*asteroid_object.get_current_resources() as f32) {
            diff = *asteroid_object.get_current_resources() as f32;
            asteroid_object.set_current_resources(0);
            info!("Asteroid exhausted! Mining timer will be removed next cycle.");
        } else {
            asteroid_object
                .set_current_resources(asteroid_object.get_current_resources() - diff as u16);
        }

        let _ = dsl.update_asteroid_by_id(asteroid_object)?; // TODO handle this properly
        create_timer_to_add_cargo_to_ship(
            ctx,
            ship_object.get_id(),
            item_def.get_id(),
            diff.floor() as u16,
        )?;

        timer.set_mining_progress(0.0); //timer.get_mining_progress() - diff.floor()); // Just reset it to 0 instead of letting it roll over

        let _ = send_info_message(
            ctx,
            &ship_object.get_player_id(),
            format!(
                "Your ship has mined {}x of {}. Attempting to load...",
                diff.floor() as u16,
                item_def.get_name()
            ),
            Some("mining"),
        );
        info!(
            "Ship #{:?} mined {}x of {}. Current progress to next item: {}",
            ship_object.get_id(),
            diff.floor() as u16,
            item_def.get_name(),
            timer.get_mining_progress()
        );
    }

    dsl.update_ship_status_by_id(ship_status)?;
    dsl.update_ship_mining_timer_by_id(timer)?;
    Ok(())
}

/// Scheduled reducer that adds mined cargo to a ship's inventory after a delay.
/// If the ship's cargo bay is full, creates a cargo crate in space instead.
#[spacetimedb::reducer]
pub fn ship_add_cargo_timer_reducer(
    ctx: &ReducerContext,
    timer: ShipAddCargoTimer,
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    info!(
        "Attempting to add {}x #{:?} to ship id #{:?}",
        timer.amount,
        timer.get_item_id(),
        timer.get_ship_id()
    );

    // Either way, we don't want this to continue.
    dsl.delete_ship_add_cargo_timer_by_id(&timer)?;

    let ship_object = dsl.get_ship_by_id(timer.get_ship_id())?;
    let mut ship_status = dsl.get_ship_status_by_id(timer.get_ship_id())?;

    // Get the item definition
    let item_def = get_item_definition(ctx, timer.item_id)?;

    // Attempt to load it into the ship
    attempt_to_load_cargo_into_ship(
        ctx,
        &mut ship_status,
        &ship_object.get_id(),
        &item_def,
        timer.amount,
        true,
    )?;

    Ok(())
}
