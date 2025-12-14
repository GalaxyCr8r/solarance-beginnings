use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::{
    logic::ships::{mining::*, station_interactions::dock_to_station},
    tables::{
        common_types::Vec2, jumpgates::*, players::*, sectors::GetSectorRowOptionById,
        server_messages::send_info_message, ships::*, stations::*, stellarobjects::*,
    },
};

/////////////////////////////////////////
/// Object-type Logic

pub fn try_mining_asteroid(
    dsl: &DSL,
    controller: &PlayerShipController,
    ship_object: &Ship,
    ship_sobj: &StellarObject,
    asteroid_sobj: &StellarObject,
) -> Result<(), String> {
    if !controller.get_mining_laser_on() {
        return Ok(());
    }

    // If the player is trying to mine and is targetting an asteroid, create a mining timer.
    if ship_sobj
        .distance_squared(dsl, asteroid_sobj)
        .is_ok_and(|d| d < (300.0_f32).powi(2))
    {
        // Check if the player is already mining this asteroid
        if !dsl
            .get_ship_mining_timers_by_ship_sobj_id(&ship_object.get_sobj_id())
            .any(|timer| timer.get_asteroid_sobj_id().value() == asteroid_sobj.get_id().value())
        {
            // Only add if there is no mining timer for this ship and asteroid.
            let _ = send_info_message(
                dsl,
                &controller.get_id(),
                format!(
                    "Player {} started mining asteroid #{}!",
                    get_username(dsl, controller.get_id().value()),
                    asteroid_sobj.get_id().value()
                ),
                Some("mining"),
            ); // Should this Error really be suppressed?
            info!(
                "Player {} started mining asteroid #{}!",
                get_username(dsl, controller.get_id().value()),
                asteroid_sobj.get_id().value()
            );
            let _ = create_mining_timer_for_ship(
                dsl,
                &ship_object.get_sobj_id(),
                &asteroid_sobj.get_id(),
            )?;
        }
    }

    Ok(())
}

pub fn try_to_dock_to_station(
    dsl: &DSL,
    player_ship_obj: &Ship,
    ship_sobj: &StellarObject,
    station: &Station,
) -> Result<(), String> {
    // Check if same faction

    // If not, check faction standing

    info!("Trying to dock to station!");
    dock_to_station(dsl, player_ship_obj, ship_sobj, station)?;

    Ok(())
}

pub fn try_to_use_jumpgate(
    dsl: &DSL,
    player_ship_obj: &Ship,
    jumpgate: &JumpGate,
) -> Result<(), String> {
    let mut ship_status = dsl.get_ship_status_by_id(player_ship_obj.get_id())?;

    // Jump once they have more than 100 energy
    if *ship_status.get_energy() > 100.0 {
        ship_status.set_energy(ship_status.get_energy() - 100.0);
        dsl.update_ship_status_by_id(ship_status)?;
        //teleport_via_jumpgate(dsl, player_ship_obj.clone(), jumpgate)?;
        let mut ship = player_ship_obj.clone();

        let pos: &Vec2 = jumpgate.get_target_gate_arrival_pos();
        let destination_sector = dsl.get_sector_by_id(jumpgate.get_target_sector_id())?;

        ship.set_sector_id(&destination_sector);
        if let Ok(mut sobj) = dsl.get_stellar_object_by_id(&ship.get_sobj_id()) {
            sobj.set_sector_id(&destination_sector);
            if let Ok(mut transform) = dsl.get_sobj_internal_transform_by_id(&sobj.get_id()) {
                transform.set_x(pos.x);
                transform.set_y(pos.y);
                dsl.update_sobj_internal_transform_by_id(transform)?;
            }
            dsl.update_stellar_object_by_id(sobj)?;
        }
        if let Ok(mut ship_status) = dsl.get_ship_status_by_id(&ship.get_id()) {
            ship_status.set_sector_id(&destination_sector);
            dsl.update_ship_status_by_id(ship_status)?;
        }

        send_info_message(
            dsl,
            &ship.get_player_id(),
            format!(
                "Jumped successfully via jumpgate to sector #{}: {}",
                destination_sector.get_id().value(),
                destination_sector.get_name()
            ),
            Some("status"),
        )?;

        dsl.update_ship_by_id(ship)?;
    }

    Ok(())
} // try_to_use_jumpgate
