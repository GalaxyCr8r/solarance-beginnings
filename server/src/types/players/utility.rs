use std::f32::consts::PI;

use glam::Vec2;
use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::types::{
    items::*,
    jumpgates::*,
    ships::{timers::*, utility::*},
    stations::*,
};

use super::*;

pub fn get_username(ctx: &ReducerContext, id: Identity) -> String {
    if let Some(player) = ctx.db().player().id().find(id) {
        player.username
    } else {
        if ctx.sender == ctx.identity() {
            "SERVER".to_string()
        } else {
            id.to_abbreviated_hex().to_string()
        }
    }
}

/// Verifies the controller's targetted stellar object exists and retrieves it.
pub fn get_targetted_sobj(
    ctx: &ReducerContext,
    controller: &PlayerShipController,
    player_sobj: &StellarObject,
) -> Result<StellarObject, String> {
    let dsl = dsl(ctx);

    let target_sobj =
        dsl.get_stellar_object_by_id(StellarObjectId::new(controller.targetted_sobj_id.unwrap()))?;
    if player_sobj.get_sector_id() != target_sobj.get_sector_id() {
        Err(
            format!(
                "Player {} tried to target a stellar object in a different sector! Player SOBJ ID: {:?}, Target SOBJ ID: {:?}",
                get_username(ctx, controller.id),
                player_sobj.get_id(),
                target_sobj.get_id()
            )
        )
    } else {
        Ok(target_sobj)
    }
}

pub fn try_update_ship_velocity(
    ctx: &ReducerContext,
    velocity: &mut StellarObjectVelocity,
    controller: &PlayerShipController,
    ship_object: &Ship,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    let ship_type = dsl.get_ship_type_definition_by_id(ship_object.get_shiptype_id())?;
    let transform = dsl.get_sobj_internal_transform_by_id(&ship_object.get_sobj_id())?;

    // Based on the controller's settings and the ship definition and ship status, update the velocity.
    if controller.up {
        let mut vec = Vec2::from_angle(*transform.get_rotation_radians())
            * ship_type.get_base_acceleration()
            + velocity.to_vec2();

        // Check if the absolute velocity is too fast for the ship.
        if vec.length() > *ship_type.get_base_speed() {
            // Set the velocity
            vec = vec.normalize() * ship_type.get_base_speed();
        }

        *velocity = velocity.from_vec2(vec);
    }
    if controller.right {
        velocity.set_rotation_radians(PI * ship_type.get_base_turn_rate());
    }
    if controller.left {
        velocity.set_rotation_radians(PI * -ship_type.get_base_turn_rate());
    }
    if controller.down {
        let mul = 0.965f32; // TODO: Control this somehow via ship def or a global config.
        velocity.set_x(velocity.get_x() * mul);
        velocity.set_y(velocity.get_y() * mul);
    }

    Ok(())
}

pub fn remove_old_timers(
    ctx: &ReducerContext,
    controller: &PlayerShipController,
    ship_object: &Ship,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if !controller.mining_laser_on {
        for mining_timer in dsl.get_ship_mining_timers_by_ship_sobj_id(ship_object.get_sobj_id()) {
            info!(
                "Player {} stopped trying to mine a asteroid: {}",
                get_username(ctx, controller.id),
                mining_timer.get_asteroid_sobj_id()
            );
            dsl.delete_ship_mining_timer_by_id(&mining_timer)?;
            return Ok(());
        }
    }

    Ok(())
}

pub fn try_mining_asteroid(
    ctx: &ReducerContext,
    controller: &PlayerShipController,
    ship_object: &Ship,
    ship_sobj: &StellarObject,
    asteroid_sobj: &StellarObject,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if !controller.mining_laser_on {
        return Ok(());
    }

    // If the player is trying to mine and is targetting an asteroid, create a mining timer.
    if ship_sobj
        .distance_squared(ctx, asteroid_sobj)
        .is_ok_and(|d| d < (300.0_f32).powi(2))
    {
        // Check if the player is already mining this asteroid
        if !dsl
            .get_ship_mining_timers_by_ship_sobj_id(&ship_object.get_sobj_id())
            .any(|timer| timer.get_asteroid_sobj_id().value() == asteroid_sobj.get_id().value())
        {
            // Only add if there is no mining timer for this ship and asteroid.
            info!(
                "Player {} started mining asteroid {:?}!",
                get_username(ctx, controller.id),
                asteroid_sobj.get_id()
            );
            let _ = create_mining_timer_for_ship(
                ctx,
                &ship_object.get_sobj_id(),
                &asteroid_sobj.get_id(),
            )?;
        }
    }

    Ok(())
}

pub fn attempt_to_pickup_cargo_crate(
    ctx: &ReducerContext,
    player_ship_obj: &Ship,
    crate_sobj: &StellarObject,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    let cargo_crate = dsl.get_cargo_crate_by_sobj_id(crate_sobj)?;
    let item_def = dsl.get_item_definition_by_id(cargo_crate.get_item_id())?;
    let ship = dsl.get_ship_status_by_id(player_ship_obj.get_id())?;

    if item_def.can_any_of_this_fit_inside_this_ship(&ship) {
        match create_timer_to_add_cargo_to_ship(
            // Do the actual thing
            ctx,
            ship.get_id(),
            item_def.get_id(),
            *cargo_crate.get_quantity(),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "ERROR {} : Ship {:?} could not fit {}x #{:?} items",
                e,
                ship.get_id(),
                *cargo_crate.get_quantity(),
                item_def.get_id()
            )),
        }
    } else {
        Err(format!(
            "Ship {:?} could not fit {}x #{:?} items",
            ship.get_id(),
            *cargo_crate.get_quantity(),
            item_def.get_id()
        ))
    }
}

pub fn try_to_dock_to_station(
    ctx: &ReducerContext,
    player_ship_obj: &Ship,
    ship_sobj: &StellarObject,
    station: &Station,
) -> Result<(), String> {
    // Check if same faction

    // If not, check faction standing

    info!("Trying to dock to station!");
    dock_to_station(ctx, player_ship_obj, ship_sobj, station)?;

    Ok(())
}

pub fn try_to_use_jumpgate(
    ctx: &ReducerContext,
    player_ship_obj: &Ship,
    jumpgate: &JumpGate,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    let mut ship_status = dsl.get_ship_status_by_id(player_ship_obj.get_id())?;

    // Jump once they have more than 100 energy
    if *ship_status.get_energy() > 100.0 {
        ship_status.set_energy(ship_status.get_energy() - 100.0);
        dsl.update_ship_status_by_id(ship_status)?;
        teleport_via_jumpgate(ctx, player_ship_obj.clone(), jumpgate)?;
    }

    Ok(())
} // try_to_dock_to_station
