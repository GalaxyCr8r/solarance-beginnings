use std::time::Duration;

use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::{
    logic::ships::add_cargo_timer::*,
    tables::{asteroids::*, items::*, players::*, server_messages::*, ships::*, stellarobjects::*},
    utility::try_server_only,
};

#[dsl(plural_name = ship_mining_timers, method(update = true))]
#[spacetimedb::table(name = ship_mining_timer, scheduled(ship_mining_timer_reducer))]
pub struct ShipMiningTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[index(btree)]
    #[use_wrapper(StellarObjectId)]
    /// FK to StellarObject
    ship_sobj_id: u64,

    #[use_wrapper(StellarObjectId)]
    /// FK to StellarObject
    asteroid_sobj_id: u64,

    pub mining_progress: f32, // How much of the asteroid has been mined (0 to 1.0)
}

pub fn create_mining_timer_for_ship<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    ship_sobj_id: &StellarObjectId,
    asteroid_sobj_id: &StellarObjectId,
) -> Result<ShipMiningTimer, String> {
    // Check if the ship is already mining and remove those timers.
    for timer in dsl.get_ship_mining_timers_by_ship_sobj_id(ship_sobj_id) {
        dsl.delete_ship_mining_timer_by_id(timer.get_id())?;
    }

    // Check if the ship and asteroid are in the same sector
    if !same_sector_from_ids(dsl, &ship_sobj_id, &asteroid_sobj_id) {
        return Err(format!(
            "What are you trying to mine? {} and {} are in different sectors",
            ship_sobj_id.clone().value(),
            asteroid_sobj_id.clone().value()
        ));
    }

    Ok(dsl.create_ship_mining_timer(CreateShipMiningTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_secs(3).into()),
        ship_sobj_id: ship_sobj_id.clone(),
        asteroid_sobj_id: asteroid_sobj_id.clone(),
        mining_progress: 0.0,
    })?)
}

/// Scheduled reducer that processes ship mining operations against asteroids.
/// Runs every 3 seconds to extract resources based on mining equipment and energy consumption.
#[spacetimedb::reducer]
pub fn ship_mining_timer_reducer(
    ctx: &ReducerContext,
    mut timer: ShipMiningTimer,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    let ship_object = dsl
        .get_ships_by_sobj_id(timer.get_ship_sobj_id())
        .next()
        .ok_or("Couldn't find ship.".to_string())?;
    let mut asteroid_object = dsl.get_asteroid_by_id(timer.get_asteroid_sobj_id())?;

    if *asteroid_object.get_current_resources() == 0 {
        dsl.delete_ship_mining_timer_by_id(timer.get_id())?;

        let _ = dsl.delete_stellar_object_by_id(&asteroid_object.get_id());

        let _ = send_info_message(
            &dsl,
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
    let item_def = dsl.get_item_definition_by_id(asteroid_object.get_resource_item_id())?;

    // Do the logic to determine speed of mining based on mining equipment, item id, etc.
    let mut energy_consumption = 1.0f32;
    let mut mining_speed = 1.0f32;
    for item in dsl.get_ship_equipment_slots_by_ship_id(ship_object.get_id()) {
        if *item.get_slot_type() == EquipmentSlotType::MiningLaser {
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
            &dsl,
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
                .set_current_resources(asteroid_object.get_current_resources() - (diff as u16));
        }

        let _ = dsl.update_asteroid_by_id(asteroid_object)?; // TODO handle this properly
        create_timer_to_add_cargo_to_ship(
            &dsl,
            ship_object.get_id(),
            item_def.get_id(),
            diff.floor() as u16,
        )?;

        timer.set_mining_progress(0.0); //timer.get_mining_progress() - diff.floor()); // Just reset it to 0 instead of letting it roll over

        let _ = send_info_message(
            &dsl,
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

/// Tries to begin mining the given asteroid. Uses the `ctx.sender` to try to find the player ship.
#[reducer]
pub fn try_mining_asteroid(
    ctx: &ReducerContext,
    asteroid_sobj_id: StellarObjectId,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Find the player's ship by assuming they have a movement controller on their current ship
    let player_id = PlayerId::new(ctx.sender);
    let (ship_object, ship_sobj) = get_player_ship_and_sobj(&dsl, &player_id)?;

    // TODO: Check if the player ship has a mining laser installed.

    let asteroid_sobj = dsl.get_stellar_object_by_id(asteroid_sobj_id)?;

    // If the player is trying to mine and is targetting an asteroid, create a mining timer.
    if ship_sobj
        .distance_squared(&dsl, &asteroid_sobj)
        .is_ok_and(|d| d < (300.0_f32).powi(2))
    // TODO: Make mining range a config const
    {
        // Check if the player is already mining this asteroid
        if !dsl
            .get_ship_mining_timers_by_ship_sobj_id(&ship_object.get_sobj_id())
            .any(|timer| timer.get_asteroid_sobj_id().value() == asteroid_sobj.get_id().value())
        {
            // Only add if there is no mining timer for this ship and asteroid.
            let _ = send_info_message(
                &dsl,
                &player_id,
                format!(
                    "Player {} started mining asteroid #{}!",
                    get_username(&dsl, player_id.value()),
                    asteroid_sobj.get_id().value()
                ),
                Some("mining"),
            ); // Should this Error really be suppressed?
            info!(
                "Player {} started mining asteroid #{}!",
                get_username(&dsl, player_id.value()),
                asteroid_sobj.get_id().value()
            );
            let _ =
                create_mining_timer_for_ship(&dsl, &ship_sobj.get_id(), &asteroid_sobj.get_id())?;
        }

        Ok(())
    } else {
        Err("Asteroid is outside mining range!".to_string())
    }
}
