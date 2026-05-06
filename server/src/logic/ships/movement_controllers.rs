use std::f32::consts::PI;

use glam::Vec2;
use log::warn;
use spacetimedb::{table, ReducerContext};
use spacetimedsl::*;

use crate::tables::{players::*, ships::*, stellarobjects::*};

#[spacetimedb::reducer]
pub fn update_ship_movement_controller(
    ctx: &ReducerContext,
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let player_id = PlayerId::new(ctx.sender());
    let mut controller = dsl
        .get_ship_movement_controller_by_id(&player_id)
        .ok_or("No movement controller for player")?;
    controller.forward = forward;
    controller.backward = backward;
    controller.left = left;
    controller.right = right;
    dsl.update_ship_movement_controller_by_id(controller)?;
    Ok(())
}

pub fn initialize_controller_for_player<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player: &PlayerId,
    sobj: &StellarObject,
) -> Result<(), String> {
    dsl.create_ship_movement_controller(CreateShipMovementController {
        id: player.clone(),
        stellar_object_id: sobj.get_id(),
        forward: false,
        backward: false,
        left: false,
        right: false,
    })?;
    Ok(())
}

/////////////////////////////////////////
/// Timers
///

/// Scheduled reducer that updates player ship movement controls and physics.
/// Runs at 20 FPS to handle ship acceleration, rotation, and velocity damping based on player input.
#[dsl(plural_name = create_update_ship_movement_controllers_timers, method(update = false))]
#[table(
    accessor = create_update_ship_movement_controllers_timer,
    scheduled(timer_update_all_ship_movement_controllers)
)]
pub struct UpdateShipMovementControllers {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

//
//////////////////////////////////////////////////////////////
// Timer Reducers
//////////////////////////////////////////////////////////////

/// Scheduled reducer that updates player ship movement controls and physics.
/// Runs at 20? FPS to handle ship acceleration, rotation, and velocity damping based on player input.
#[spacetimedb::reducer]
pub fn timer_update_all_ship_movement_controllers(
    ctx: &ReducerContext,
    _timer: UpdateShipMovementControllers,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    //info!("Player con upkeep!");

    for controller in dsl.get_all_ship_movement_controllers() {
        match update_ship_movement_controller(&dsl, controller) {
            Ok(updated) => {
                dsl.update_ship_movement_controller_by_id(updated)?;
            }
            Err(err) => {
                warn!("Error: {err}");
            }
        }
    }

    Ok(())
}

fn update_ship_movement_controller(
    dsl: &DSL<ReducerContext>,
    controller: ShipMovementController,
) -> Result<ShipMovementController, String> {
    let ship_object = match dsl
        .get_ships_by_sobj_id(controller.get_stellar_object_id())
        .next()
    {
        Some(ship) => ship,
        None => {
            // Ship might be docked, clean up the timer
            dsl.delete_ship_movement_controller_by_id(&controller.get_id())?;
            return Err(format!(
                "Ship not found (likely docked), removing update timer for player ID: {}",
                controller.get_id().value()
            ));
        }
    };
    let mut velocity = dsl.get_sobj_velocity_by_id(ship_object.get_sobj_id())?;

    // If no input was given, slow down the rotation & speed
    if !controller.left && !controller.right {
        velocity.set_rotation_radians(velocity.get_rotation_radians() * 0.875); // TODO:: Milestone 10+ make these ship-type specific values.
    }
    if !controller.forward && !controller.backward {
        // Add inertia to the velocity
        velocity = velocity.from_vec2(velocity.to_vec2() * 0.975); // TODO:: Milestone 10+ make these ship-type specific values.
        if velocity.to_vec2().length() < 0.042 {
            velocity = velocity.from_vec2(Vec2::ZERO);
        }
    }

    if controller.left || controller.right || controller.forward || controller.backward {
        try_update_ship_velocity(&dsl, &mut velocity, &controller, &ship_object)?;
    }

    Ok(controller)
}

pub fn try_update_ship_velocity(
    dsl: &DSL<ReducerContext>,
    velocity: &mut StellarObjectVelocity,
    controller: &ShipMovementController,
    ship_object: &Ship,
) -> Result<(), String> {
    let ship_type = dsl.get_ship_type_definition_by_id(ship_object.get_shiptype_id())?;
    let transform = dsl.get_sobj_internal_transform_by_id(&ship_object.get_sobj_id())?;

    // Based on the controller's settings and the ship definition and ship status, update the velocity.
    if controller.forward {
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
    if controller.backward {
        let mul = 0.965f32; // TODO: Control this somehow via ship def or a global config.
        velocity.set_x(velocity.get_x() * mul);
        velocity.set_y(velocity.get_y() * mul);
    }

    Ok(())
}
