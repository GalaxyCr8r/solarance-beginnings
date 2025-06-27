use std::time::Duration;

use glam::Vec2;
use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::types::{ jumpgates::*, players::{ utility::*, * }, stations::* };

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

pub fn initialize_player_controller(
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
// Timer Reducers
//////////////////////////////////////////////////////////////

/// Update the movement-related controls.
#[spacetimedb::reducer]
pub fn player_ship_controller_update_upkeep(
    ctx: &ReducerContext,
    timer: PlayerControllerUpdateTimer
) -> Result<(), String> {
    let dsl = dsl(ctx);

    //info!("Player con upkeep!");

    let mut controller = match dsl.get_player_ship_controller_by_player_id(&timer.player) {
        Some(con) => con,
        None => {
            dsl.delete_player_ship_controller_update_timer_by_scheduled_id(&timer);
            info!("Failed to find the player's controller! ID:{} Removing timer.", timer.player);
            return Ok(());
        }
    };

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

    let mut controller = match dsl.get_player_ship_controller_by_player_id(&timer.player) {
        Some(con) => con,
        None => {
            dsl.delete_player_ship_controller_logic_timer_by_scheduled_id(&timer);
            info!("Failed to find the player's controller! ID:{} Removing timer.", timer.player);
            return Ok(());
        }
    };
    let ship_object = dsl
        .get_ship_by_sobj_id(controller.get_stellar_object_id())
        .ok_or(format!("Failed to find the player's ship object! ID:{}", timer.player))?;

    remove_old_timers(ctx, &controller, &ship_object)?;

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

    match get_targetted_sobj(ctx, &controller, &player_sobj) {
        // These "Do things if nearby target" should be in their own timer. As-is things will ONLY happen if you are updating your controller when nearby!!!
        Ok(target_sobj) => {
            match target_sobj.kind {
                // StellarObjectKinds::Ship => {
                //     // Nothing to do.. yet

                //     // Maybe implement ship scanning?
                // }
                StellarObjectKinds::Asteroid => {
                    try_mining_asteroid(
                        ctx,
                        &controller,
                        &ship_object,
                        &player_sobj,
                        &target_sobj
                    )?;

                    // Maybe implement asteroid scanning?
                }
                StellarObjectKinds::CargoCrate => {
                    if
                        controller.cargo_bay_open &&
                        player_sobj.distance_squared(ctx, &target_sobj).is_some_and(|d| d < 1000.0)
                    {
                        // Picking up the crate!
                        attempt_to_pickup_cargo_crate(ctx, &ship_object, &target_sobj)?;
                        controller.targetted_sobj_id = None;
                        target_sobj.delete(ctx, true);
                    }
                }
                StellarObjectKinds::Station => {
                    if
                        controller.dock &&
                        player_sobj
                            .distance_squared(ctx, &target_sobj)
                            .is_some_and(|d| d < (100.0_f32).powi(2))
                    {
                        if let Some(station) = dsl.get_station_by_sobj_id(&target_sobj) {
                            try_to_dock_to_station(ctx, &ship_object, &player_sobj, &station)?;
                        }
                    }
                }
                StellarObjectKinds::JumpGate => {
                    if
                        controller.dock &&
                        player_sobj
                            .distance_squared(ctx, &target_sobj)
                            .is_some_and(|d| d < (100.0_f32).powi(2))
                    {
                        if let Some(jumpgate) = dsl.get_jump_gate_by_sobj_id(&target_sobj) {
                            try_to_use_jumpgate(ctx, &ship_object, &jumpgate)?;
                        }
                    }
                }
                _ => {
                    // Do nothing
                }
            }
        }
        Err(error) => {
            info!(
                "WARNING: {} - Untargetting the sobj #{}",
                error,
                controller.targetted_sobj_id.unwrap()
            );
            controller.targetted_sobj_id = None;
            dsl.update_player_ship_controller_by_player_id(controller.clone())?;
        }
    }

    Ok(())
}
