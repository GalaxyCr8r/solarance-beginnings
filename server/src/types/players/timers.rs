use std::{ f32::consts::PI, time::Duration };

use glam::Vec2;
use log::info;
use spacetimedb::*;
use spacetimedsl::{ dsl, Wrapper };

use crate::types::{
    items::*,
    jumpgates::{ GetJumpGateRowOptionBySobjId, JumpGate },
    players::{utility::get_username, *},
    ships::{ reducers::*, timers::* },
};


#[dsl(plural_name = player_ship_controller_update_timers)]
#[spacetimedb::table(
    name = player_ship_controller_update_timer,
    scheduled(player_ship_controller_update_upkeep)
)]
pub struct PlayerControllerUpdateTimer {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    pub player: Identity,
}

#[dsl(plural_name = player_ship_controller_logic_timers)]
#[spacetimedb::table(
    name = player_ship_controller_logic_timer,
    scheduled(player_ship_controller_logic_upkeep)
)]
pub struct PlayerControllerLogicTimer {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    pub player: Identity,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx); // Waiting for DSL implementation of timers

    // Timers are created when the Player Controller is created.

    Ok(())
}

pub fn init_timers(
    ctx: &ReducerContext,
    identity: Identity,
    sobj: &StellarObject
) -> Result<(), String> {
    let dsl = dsl(ctx);

    let _controller = dsl.create_player_ship_controller(
        identity,
        sobj.get_id(),
        false,
        false,
        false,
        false,
        CurrentAction::Idle,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        None
    )?;
    let _update = dsl.create_player_ship_controller_update_timer(
        spacetimedb::ScheduleAt::Interval(Duration::from_millis(1000 / 20).into()),
        identity
    )?;
    let _logic = dsl.create_player_ship_controller_logic_timer(
        spacetimedb::ScheduleAt::Interval(Duration::from_millis(1000 / 2).into()),
        identity
    )?;
    Ok(())
}

//////////////////////////////////////////////////////////////
// Reducers
//////////////////////////////////////////////////////////////

/// Update the movement-related controls.
#[spacetimedb::reducer]
pub fn player_ship_controller_update_upkeep(
    ctx: &ReducerContext,
    timer: PlayerControllerUpdateTimer
) -> Result<(), String> {
    let dsl = dsl(ctx);

    //info!("Player con upkeep!");

    let controller = dsl
        .get_player_ship_controller_by_player_id(&timer.player)
        .ok_or(format!("Failed to find the player's controller! ID:{}", timer.player))?;

    let ship_object = dsl
        .get_ship_by_sobj_id(controller.get_stellar_object_id())
        .ok_or(format!("Failed to find the player's ship object! ID:{}", timer.player))?;
    let mut velocity = dsl
        .get_sobj_velocity_by_sobj_id(ship_object.get_sobj_id())
        .ok_or(format!("Failed to find the player's ship velocity! ID:{}", timer.player))?;

    // If no input was given, slow down the rotation & speed
    if !controller.left && !controller.right {
        velocity.rotation_radians *= 0.875; // TODO:: Milestone 10+ make these ship-type specific values.
    }
    if !controller.up && !controller.down {
        // Add inertia to the velocity
        velocity = velocity.from_vec2(velocity.to_vec2() * 0.975); // TODO:: Milestone 10+ make these ship-type specific values.
        if velocity.to_vec2().length() < 0.042 {
            velocity = velocity.from_vec2(Vec2::ZERO);
        }
    }

    if controller.left || controller.right || controller.up || controller.down {
        try_update_ship_velocity(ctx, &mut velocity, &controller, &ship_object)?;
    }

    dsl.update_sobj_velocity_by_sobj_id(velocity)?;

    Ok(())
}

/// Update the logical features that players control that aren't as time sensitive.
#[spacetimedb::reducer]
pub fn player_ship_controller_logic_upkeep(
    ctx: &ReducerContext,
    timer: PlayerControllerLogicTimer
) -> Result<(), String> {
    let dsl = dsl(ctx);

    let controller = dsl
        .get_player_ship_controller_by_player_id(&timer.player)
        .ok_or(format!("Failed to find the player's controller! ID:{}", timer.player))?;
    let ship_object = dsl
        .get_ship_by_sobj_id(controller.get_stellar_object_id())
        .ok_or(format!("Failed to find the player's ship object! ID:{}", timer.player))?;

    try_mining(ctx, &controller, &ship_object)?;
    try_update_target_specific_functions(ctx, &mut controller.clone(), &ship_object)?;

    Ok(())
}

//////////////////////////////////////////////////////////////
// Helpers
//////////////////////////////////////////////////////////////

fn try_update_target_specific_functions(
    ctx: &ReducerContext,
    controller: &mut PlayerShipController,
    ship_object: &Ship
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Check target-specific things.
    if controller.targetted_sobj_id.is_none() {
        return Ok(());
    }

    let player_sobj = dsl
        .get_stellar_object_by_id(ship_object.get_sobj_id())
        .ok_or(
            format!(
                "ERROR: Player {} has stellar object #{} that does not exist!",
                get_username(ctx, controller.player_id),
                controller.stellar_object_id
            )
        )?;

    match verify_target(ctx, &controller, &player_sobj) {
        // These "Do things if nearby target" should be in their own timer. As-is things will ONLY happen if you are updating your controller when nearby!!!
        Ok(target_sobj) => {
            match target_sobj.kind {
                // StellarObjectKinds::Ship => {
                //     // Nothing to do.. yet

                //     // Maybe implement ship scanning?
                // }
                // StellarObjectKinds::Asteroid => {
                //     // Nothing to do.. yet

                //     // Maybe implement asteroid scanning?
                // }
                StellarObjectKinds::CargoCrate => {
                    if controller.cargo_bay_open &&
                       player_sobj
                            .distance_squared(ctx, &target_sobj)
                            .is_some_and(|d| d < 1000.0) &&
                        attempt_to_pickup_cargo_crate(ctx, &ship_object, &target_sobj)
                    {
                        // Picking up the crate!
                        controller.targetted_sobj_id = None;
                        target_sobj.delete(ctx, true);
                    }
                }
                // StellarObjectKinds::Station => {
                //     // Nothing to do.. yet
                // }
                StellarObjectKinds::JumpGate => {
                    if
                        controller.dock &&
                        player_sobj.distance_squared(ctx, &target_sobj).is_some_and(|d| d < 1000.0)
                    {
                        if let Some(jumpgate) = dsl.get_jump_gate_by_sobj_id(&target_sobj) {
                            attempt_to_jump(ctx, ship_object, &jumpgate)?;
                        }
                    }
                }
                _ => {
                    // Do nothing
                }
            }
        }
        Err(error) => {
            info!("WARNING: {}", error);
            controller.targetted_sobj_id = None;
            dsl.update_player_ship_controller_by_player_id(controller.clone())?;
        }
    }
    Ok(())
}

/// Verifies the controller's targetted stellar object exists and retrieves it.
fn verify_target(
    ctx: &ReducerContext,
    controller: &PlayerShipController,
    player_sobj: &StellarObject
) -> Result<StellarObject, String> {
    let dsl = dsl(ctx);

    if
        let Some(target_sobj) = dsl.get_stellar_object_by_id(
            StellarObjectId::new(controller.targetted_sobj_id.unwrap())
        )
    {
        if player_sobj.sector_id != target_sobj.sector_id {
            Err(
                format!(
                    "Player {} tried to target a stellar object in a different sector! Player SOBJ ID: {}, Target SOBJ ID: {}",
                    get_username(ctx, controller.player_id),
                    player_sobj.id,
                    target_sobj.id
                )
            )
        } else {
            Ok(target_sobj)
        }
    } else {
        Err(
            format!(
                "Player {} tried targetting a non-existant stellar object #{}!",
                get_username(ctx, controller.player_id),
                controller.targetted_sobj_id.unwrap()
            )
        )
    }
}

fn attempt_to_pickup_cargo_crate(
    ctx: &ReducerContext,
    player_ship_obj: &Ship,
    crate_sobj: &StellarObject
) -> bool {
    let dsl = dsl(ctx);

    if let Some(cargo_crate) = dsl.get_cargo_crate_by_sobj_id(crate_sobj) {
        if let Some(item_def) = dsl.get_item_definition_by_id(cargo_crate.get_item_id()) {
            if let Some(ship) = dsl.get_ship_status_by_id(player_ship_obj.get_id()) {
                if item_def.can_any_of_this_fit_inside_this_ship(&ship) {
                    return create_timer_to_add_cargo_to_ship(
                        ctx,
                        ship.get_id(),
                        item_def.get_id(),
                        cargo_crate.quantity
                    )
                        .inspect_err(|e| info!("WARNING: Couldn't add cargo crate timer: {}", e))
                        .is_ok();
                }
            }
        }
    }

    false
}

fn attempt_to_jump(
    ctx: &ReducerContext,
    player_ship_obj: &Ship,
    jumpgate: &JumpGate
) -> Result<(), String> {
    let dsl = dsl(ctx);

    let ship = dsl.get_ship_by_id(player_ship_obj.get_id())
        .ok_or("Failed to find Ship object.")?;

    if let Some(mut ship_status) = dsl.get_ship_status_by_id(player_ship_obj.get_id()) {
        // Subtract energy
        ship_status.energy -= 13.37; // TODO: Make this not a flat value. If some ship had crazy energy regen then.. they wouldn't be able to use jump gates.
        if ship_status.energy < 0.0 {
            ship_status.energy = 0.0;
        }

        // Jump once less than 10 energy
        if ship_status.energy < 10.0 {
            teleport_via_jumpgate(ctx, ship.clone(), jumpgate)?;
        } else {
            // Teleporting will update the ship instance, otherwise we will need to do it ourselves
            dsl.update_ship_status_by_id(ship_status)?;
        }
    }
    Ok(())
}

fn try_update_ship_velocity(
    ctx: &ReducerContext,
    velocity: &mut StellarObjectVelocity,
    controller: &PlayerShipController,
    ship_object: &Ship
) -> Result<(), String> {
    let dsl = dsl(ctx);
    
    let ship_type = dsl
        .get_ship_type_definition_by_id(ship_object.get_shiptype_id())
        .ok_or(
            format!(
                "Failed to find the player's ship type defintion! ID:{:?}",
                ship_object.get_shiptype_id()
            )
        )?;
    let transform = dsl
        .get_sobj_internal_transform_by_sobj_id(ship_object.get_sobj_id())
        .ok_or(
            format!(
                "Failed to find the player's ship transform! ID:{:?}",
                ship_object.get_sobj_id()
            )
        )?;

    // Based on the controller's settings and the ship definition and ship status, update the velocity.
    if controller.up {
        let mut vec =
            Vec2::from_angle(transform.rotation_radians) * ship_type.base_acceleration +
            velocity.to_vec2();

        // Check if the absolute velocity is too fast for the ship.
        if vec.length() > ship_type.base_speed {
            // Set the velocity
            vec = vec.normalize() * ship_type.base_speed;
        }

        *velocity = velocity.from_vec2(vec);
    }
    if controller.right {
        velocity.rotation_radians = PI * ship_type.base_turn_rate;
    }
    if controller.left {
        velocity.rotation_radians = PI * -ship_type.base_turn_rate;
    }
    if controller.down {
        let mul = 0.965f32; // TODO: Control this somehow via ship def or a global config.
        velocity.x *= mul;
        velocity.y *= mul;
    }

    Ok(())
}

fn try_mining(
    ctx: &ReducerContext,
    controller: &PlayerShipController,
    ship_object: &Ship
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if !controller.mining_laser_on {
        for mining_timer in dsl.get_ship_mining_timers_by_ship_sobj_id(ship_object.get_sobj_id()) {
            info!(
                "Player {} stopped trying to mine a asteroid: {}",
                get_username(ctx, controller.player_id),
                mining_timer.asteroid_sobj_id
            );
            dsl.delete_ship_mining_timer_by_scheduled_id(&mining_timer);
            return Ok(());
        }
    }

    // If the player is trying to mine and is targetting an asteroid, create a mining timer.
    if controller.mining_laser_on && controller.targetted_sobj_id.is_some() {
        let asteroid_sobj = dsl
            .get_stellar_object_by_id(StellarObjectId::new(controller.targetted_sobj_id.unwrap()))
            .ok_or(
                format!(
                    "Player {} tried to mine a non-existent object: {}",
                    get_username(ctx, controller.player_id),
                    controller.targetted_sobj_id.unwrap()
                )
            )?;

        if asteroid_sobj.kind != StellarObjectKinds::Asteroid {
            return Err(
                format!(
                    "Player {} tried to mine a non-asteroid object: {}",
                    get_username(ctx, controller.player_id),
                    asteroid_sobj.id
                )
            );
        }

        if let Some(transform) = 
            dsl.get_sobj_internal_transform_by_sobj_id( ship_object.get_sobj_id() )
        {
            if let Some(ast_transform) = 
                dsl.get_sobj_internal_transform_by_sobj_id( asteroid_sobj.get_id() )
            {
                if transform.to_vec2().distance(ast_transform.to_vec2()) < 500.0 {
                    // Check if the player is already mining this asteroid
                    if !dsl
                        .get_ship_mining_timers_by_ship_sobj_id(&ship_object.get_sobj_id())
                        .any(|timer| timer.asteroid_sobj_id == asteroid_sobj.id)
                    {
                        // Only add if there is no mining timer for this ship and asteroid.
                        info!(
                            "Player {} started mining asteroid {}!",
                            get_username(ctx, controller.player_id),
                            asteroid_sobj.id
                        );
                        let _ = create_mining_timer_for_ship(
                            ctx,
                            &ship_object.get_sobj_id(),
                            &asteroid_sobj.get_id()
                        )?;
                    }
                } else {
                    // TODO: Let player know they're too far away?
                }
            }
        }
    }

    Ok(())
}
