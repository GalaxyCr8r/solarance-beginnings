use std::time::Duration;

use glam::Vec2;
use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::types::{
    jumpgates::*,
    players::{utility::*, *},
    stations::*,
};

#[dsl(plural_name = player_ship_controller_update_timers)]
#[spacetimedb::table(
    name = player_ship_controller_update_timer,
    scheduled(player_ship_controller_update_upkeep)
)]
pub struct PlayerControllerUpdateTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[use_wrapper(path = crate::players::PlayerId)]
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
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[use_wrapper(path = crate::players::PlayerId)]
    pub player: Identity,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {
    // Timers are created when the Player Controller is created.

    Ok(())
}

pub fn initialize_player_controller(
    ctx: &ReducerContext,
    player: &PlayerId,
    sobj: &StellarObject,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    let _controller = dsl.create_player_ship_controller(
        player,
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
        None,
    )?;
    let _update = dsl.create_player_ship_controller_update_timer(
        spacetimedb::ScheduleAt::Interval(Duration::from_millis(1000 / 20).into()),
        player,
    )?;
    let _logic = dsl.create_player_ship_controller_logic_timer(
        spacetimedb::ScheduleAt::Interval(Duration::from_millis(1000 / 2).into()),
        player,
    )?;
    Ok(())
}

//////////////////////////////////////////////////////////////
// Timer Reducers
//////////////////////////////////////////////////////////////

/// Scheduled reducer that updates player ship movement controls and physics.
/// Runs at 20 FPS to handle ship acceleration, rotation, and velocity damping based on player input.
#[spacetimedb::reducer]
pub fn player_ship_controller_update_upkeep(
    ctx: &ReducerContext,
    timer: PlayerControllerUpdateTimer,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    //info!("Player con upkeep!");

    let controller = match dsl.get_player_ship_controller_by_id(&timer.get_player()) {
        Ok(con) => con,
        Err(e) => {
            dsl.delete_player_ship_controller_update_timer_by_id(&timer)?;
            info!(
                "Failed to find the player's controller! Error: [{}] PID:{} Removing timer.",
                e, timer.player
            );
            return Ok(());
        }
    };

    let ship_object = match dsl.get_ship_by_sobj_id(controller.get_stellar_object_id()) {
        Ok(ship) => ship,
        Err(_) => {
            // Ship might be docked, clean up the timer
            dsl.delete_player_ship_controller_update_timer_by_id(&timer)?;
            info!(
                "Ship not found (likely docked), removing update timer for player {}",
                timer.player
            );
            return Ok(());
        }
    };
    let mut velocity = dsl.get_sobj_velocity_by_id(ship_object.get_sobj_id())?;

    // If no input was given, slow down the rotation & speed
    if !controller.left && !controller.right {
        velocity.set_rotation_radians(velocity.get_rotation_radians() * 0.875); // TODO:: Milestone 10+ make these ship-type specific values.
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

    dsl.update_sobj_velocity_by_id(velocity)?;

    Ok(())
}

/// Scheduled reducer that handles player ship logic operations like mining, docking, and cargo pickup.
/// Runs at 2 FPS to process less time-sensitive actions based on player targets and proximity.
#[spacetimedb::reducer]
pub fn player_ship_controller_logic_upkeep(
    ctx: &ReducerContext,
    timer: PlayerControllerLogicTimer,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    let mut controller = match dsl.get_player_ship_controller_by_id(&timer.get_player()) {
        Ok(con) => con,
        Err(e) => {
            dsl.delete_player_ship_controller_logic_timer_by_id(&timer)?;
            info!(
                "Failed to find the player's controller! Error: [{}] ID:{} Removing timer.",
                e, timer.player
            );
            return Ok(());
        }
    };
    let ship_object = match dsl.get_ship_by_sobj_id(controller.get_stellar_object_id()) {
        Ok(ship) => ship,
        Err(_) => {
            // Ship might be docked, clean up the timer
            dsl.delete_player_ship_controller_logic_timer_by_id(&timer)?;
            info!(
                "Ship not found (likely docked), removing logic timer for player {}",
                timer.player
            );
            return Ok(());
        }
    };

    remove_old_timers(ctx, &controller, &ship_object)?;

    // Check target-specific things.
    if controller.targetted_sobj_id.is_none() {
        return Ok(());
    }

    let player_sobj = dsl.get_stellar_object_by_id(ship_object.get_sobj_id())?;

    match get_targetted_sobj(ctx, &controller, &player_sobj) {
        // These "Do things if nearby target" should be in their own timer. As-is things will ONLY happen if you are updating your controller when nearby!!!
        Ok(target_sobj) => {
            match target_sobj.get_kind() {
                // StellarObjectKinds::Ship => {
                //     // Nothing to do.. yet

                // TODO: Maybe implement ship scanning? Combat?
                // }
                StellarObjectKinds::Asteroid => {
                    try_mining_asteroid(
                        ctx,
                        &controller,
                        &ship_object,
                        &player_sobj,
                        &target_sobj,
                    )?;

                    // Maybe implement asteroid scanning?
                }
                StellarObjectKinds::CargoCrate => {
                    if controller.cargo_bay_open
                        && player_sobj
                            .distance_squared(ctx, &target_sobj)
                            .is_ok_and(|d| d < 1000.0)
                    {
                        // Picking up the crate!
                        attempt_to_pickup_cargo_crate(ctx, &ship_object, &target_sobj)?;
                        controller.targetted_sobj_id = None;
                        let _ = dsl.delete_stellar_object_by_id(&target_sobj);
                    }
                }
                StellarObjectKinds::Station => {
                    if controller.dock
                        && player_sobj
                            .distance_squared(ctx, &target_sobj)
                            .is_ok_and(|d| d < (100.0_f32).powi(2))
                    {
                        let station = dsl.get_station_by_sobj_id(&target_sobj)?;
                        try_to_dock_to_station(ctx, &ship_object, &player_sobj, &station)?;
                    }
                }
                StellarObjectKinds::JumpGate => {
                    if controller.dock
                        && player_sobj
                            .distance_squared(ctx, &target_sobj)
                            .is_ok_and(|d| d < (100.0_f32).powi(2))
                    {
                        let jumpgate = dsl.get_jump_gate_by_id(&target_sobj)?;
                        try_to_use_jumpgate(ctx, &ship_object, &jumpgate)?;
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
            dsl.update_player_ship_controller_by_id(controller.clone())?;
        }
    }

    Ok(())
}
