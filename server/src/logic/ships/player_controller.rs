use std::f32::consts::PI;
use std::time::Duration;

use glam::Vec2;
use log::info;
use spacetimedb::{table, Identity, ReducerContext};
use spacetimedsl::*;

use crate::{
    logic::{
        cargo_crates::attempt_to_pickup_cargo_crate,
        combat::actions::*,
        ships::{controller_interactions::*, mining::*},
    },
    tables::{
        common_types::CurrentAction, jumpgates::*, players::*, ships::*, stations::*,
        stellarobjects::*,
    },
};

/// Main Table

#[dsl(plural_name = player_ship_controllers, method(update = true))]
#[table(name = player_ship_controller, public)]
pub struct PlayerShipController {
    #[primary_key]
    #[use_wrapper(PlayerId)]
    #[foreign_key(path = crate::tables::players, table = player, column = id, on_delete = Delete)]
    id: Identity,

    #[index(btree)]
    #[use_wrapper(StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    pub stellar_object_id: u64,

    // Movement
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    /// Currently selected Autopilot Action
    pub current_action: CurrentAction,

    // Equipment
    pub activate_jump_drive: bool,
    pub tractor_beam_on: bool,
    pub mining_laser_on: bool,
    pub cargo_bay_open: bool,

    // Actions
    pub dock: bool,
    pub undock: bool,
    pub shield_boost: bool,
    pub fire_weapons: bool,
    pub fire_missles: bool,

    // Misc
    /// FK to StellarObject
    pub targetted_sobj_id: Option<u64>,
}

pub fn initialize_player_controller<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player: &PlayerId,
    sobj: &StellarObject,
) -> Result<(), String> {
    dsl.create_player_ship_controller(CreatePlayerShipController {
        id: *player,
        stellar_object_id: sobj.get_id(),
        up: false,
        down: false,
        left: false,
        right: false,
        current_action: CurrentAction::Idle,
        activate_jump_drive: false,
        tractor_beam_on: false,
        mining_laser_on: false,
        cargo_bay_open: false,
        dock: false,
        undock: false,
        shield_boost: false,
        fire_weapons: false,
        fire_missles: false,
        targetted_sobj_id: None,
    })?;
    dsl.create_player_ship_controller_update_timer(CreatePlayerControllerMovementTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_millis(1000 / 20).into()),
        player: *player,
    })?;
    dsl.create_player_ship_controller_logic_timer(CreatePlayerControllerInteractionTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_millis(1000 / 2).into()),
        player: *player,
    })?;
    Ok(())
}

/// Called by players to update their own ship's controls.
#[spacetimedb::reducer]
pub fn update_player_controller(
    ctx: &ReducerContext,
    controller: PlayerShipController,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let controller_id = controller.get_id().value();

    if ctx.sender != controller_id {
        info!(
            "SECURITY ERROR: ID {} is trying to change player controller for ID {}!!! Username: {}",
            ctx.sender,
            controller_id,
            get_username(&dsl, controller_id)
        );
        return Err("ID Mismatch. This was reported to the system admin.".to_string());
    }

    // Clean up player's mining timers.
    let previous_controller = dsl.get_player_ship_controller_by_id(&controller.get_id())?;
    // TODO: Create a controller if one doesn't exist? IFF there isn't a controller for that player.

    // Check if the player had been trying to mine, if so, remove the mining timers.
    if *previous_controller.get_mining_laser_on() && !controller.get_mining_laser_on() {
        info!(
            "Player {} no longer mining, removing mining timers.",
            get_username(&dsl, controller_id)
        );
        for mining_timer in
            dsl.get_ship_mining_timers_by_ship_sobj_id(previous_controller.get_stellar_object_id())
        {
            dsl.delete_ship_mining_timer_by_id(&mining_timer)?;
        }
    }

    // Process combat actions if fire_weapons or fire_missiles are set
    let mut controller_updated = controller.clone();
    let source_sobj_id = controller.get_stellar_object_id().value();

    // Process weapon firing
    if *controller.get_fire_weapons() {
        if let Some(target_sobj_id) = controller.get_targetted_sobj_id() {
            match process_weapon_combat_action(&dsl, source_sobj_id, *target_sobj_id) {
                Ok(_) => {
                    info!(
                        "Weapon fired successfully: {} -> {} by player {}",
                        source_sobj_id,
                        target_sobj_id,
                        get_username(&dsl, controller_id)
                    );
                }
                Err(e) => {
                    info!(
                        "Weapon fire failed for ship {} (player {}): {}",
                        source_sobj_id,
                        get_username(&dsl, controller_id),
                        e
                    );
                }
            }
        } else {
            info!(
                "Player {} attempted to fire weapons without a target",
                get_username(&dsl, controller_id)
            );
        }

        // Reset fire_weapons flag
        controller_updated.set_fire_weapons(false);
    }

    // Process missile firing
    if *controller.get_fire_missles() {
        if let Some(target_sobj_id) = controller.get_targetted_sobj_id() {
            match process_missile_combat_action(&dsl, source_sobj_id, *target_sobj_id) {
                Ok(_) => {
                    info!(
                        "Missile fired successfully: {} -> {} by player {}",
                        source_sobj_id,
                        target_sobj_id,
                        get_username(&dsl, controller_id)
                    );
                }
                Err(e) => {
                    info!(
                        "Missile fire failed for ship {} (player {}): {}",
                        source_sobj_id,
                        get_username(&dsl, controller_id),
                        e
                    );
                }
            }
        } else {
            info!(
                "Player {} attempted to fire missiles without a target",
                get_username(&dsl, controller_id)
            );
        }

        // Reset fire_missles flag
        controller_updated.set_fire_missles(false);
    }

    dsl.update_player_ship_controller_by_id(controller_updated)?;

    Ok(())
}

/////////////////////////////////////////
/// Timers
///

/// Scheduled reducer that updates player ship movement controls and physics.
/// Runs at 20 FPS to handle ship acceleration, rotation, and velocity damping based on player input.
#[dsl(plural_name = player_ship_controller_update_timers, method(update = true))]
#[table(
    name = player_ship_controller_update_timer,
    scheduled(timer_player_controller_movement_update)
)]
pub struct PlayerControllerMovementTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[use_wrapper(crate::players::PlayerId)]
    pub player: Identity,
}

/// Scheduled reducer that handles player ship logic operations like mining, docking, and cargo pickup.
/// Runs at 2 FPS to process less time-sensitive actions based on player targets and proximity.
#[dsl(plural_name = player_ship_controller_logic_timers, method(update = true))]
#[table(
    name = player_ship_controller_logic_timer,
    scheduled(timer_player_controller_interaction_update)
)]
pub struct PlayerControllerInteractionTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[use_wrapper(crate::players::PlayerId)]
    pub player: Identity,
}

//////////////////////////////////////////////////////////////
// Timer Reducers
//////////////////////////////////////////////////////////////

/// Scheduled reducer that updates player ship movement controls and physics.
/// Runs at 20 FPS to handle ship acceleration, rotation, and velocity damping based on player input.
#[spacetimedb::reducer]
pub fn timer_player_controller_movement_update(
    ctx: &ReducerContext,
    timer: PlayerControllerMovementTimer,
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

    let ship_object = match dsl
        .get_ships_by_sobj_id(controller.get_stellar_object_id())
        .next()
    {
        Some(ship) => ship,
        None => {
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
        try_update_ship_velocity(&dsl, &mut velocity, &controller, &ship_object)?;
    }

    dsl.update_sobj_velocity_by_id(velocity)?;

    Ok(())
}

/// Scheduled reducer that handles player ship logic operations like mining, docking, and cargo pickup.
/// Runs at 2 FPS to process less time-sensitive actions based on player targets and proximity.
#[spacetimedb::reducer]
pub fn timer_player_controller_interaction_update(
    ctx: &ReducerContext,
    timer: PlayerControllerInteractionTimer,
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
    let ship_object = match dsl
        .get_ships_by_sobj_id(controller.get_stellar_object_id())
        .next()
    {
        Some(ship) => ship,
        None => {
            // Ship might be docked, clean up the timer
            dsl.delete_player_ship_controller_logic_timer_by_id(&timer)?;
            info!(
                "Ship not found (likely docked), removing logic timer for player {}",
                timer.player
            );
            return Ok(());
        }
    };

    remove_old_timers(&dsl, &controller, &ship_object)?;

    // Check target-specific things.
    if controller.targetted_sobj_id.is_none() {
        return Ok(());
    }

    let player_sobj = dsl.get_stellar_object_by_id(ship_object.get_sobj_id())?;

    match get_targetted_sobj(&dsl, &controller, &player_sobj) {
        // These "Do things if nearby target" should be in their own timer. As-is things will ONLY happen if you are updating your controller when nearby!!!
        Ok(target_sobj) => {
            match target_sobj.get_kind() {
                StellarObjectKinds::Ship => {
                    // Nothing to do.. yet

                    // TODO: Maybe implement ship scanning? Combat?
                }
                StellarObjectKinds::Asteroid => {
                    try_mining_asteroid(
                        &dsl,
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
                            .distance_squared(&dsl, &target_sobj)
                            .is_ok_and(|d| d < 1000.0)
                    {
                        // Picking up the crate!
                        attempt_to_pickup_cargo_crate(&dsl, &ship_object, &target_sobj)?;
                        controller.targetted_sobj_id = None;
                        let _ = dsl.delete_stellar_object_by_id(&target_sobj);
                    }
                }
                StellarObjectKinds::Station => {
                    if controller.dock
                        && player_sobj
                            .distance_squared(&dsl, &target_sobj)
                            .is_ok_and(|d| d < (100.0_f32).powi(2))
                    {
                        let station = dsl.get_station_by_sobj_id(&target_sobj)?;
                        try_to_dock_to_station(&dsl, &ship_object, &player_sobj, &station)?;
                    }
                }
                StellarObjectKinds::JumpGate => {
                    if controller.dock
                        && player_sobj
                            .distance_squared(&dsl, &target_sobj)
                            .is_ok_and(|d| d < (100.0_f32).powi(2))
                    {
                        let jumpgate = dsl.get_jump_gate_by_id(&target_sobj)?;
                        try_to_use_jumpgate(&dsl, &ship_object, &jumpgate)?;
                    }
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

/// Verifies the controller's targetted stellar object exists and retrieves it.
pub fn get_targetted_sobj<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    controller: &PlayerShipController,
    player_sobj: &StellarObject,
) -> Result<StellarObject, String> {
    let target_sobj =
        dsl.get_stellar_object_by_id(StellarObjectId::new(controller.targetted_sobj_id.unwrap()))?;
    if player_sobj.get_sector_id() != target_sobj.get_sector_id() {
        Err(
            format!(
                "Player {} tried to target a stellar object in a different sector! Player SOBJ ID: {:?}, Target SOBJ ID: {:?}",
                get_username(&dsl, controller.id),
                player_sobj.get_id(),
                target_sobj.get_id()
            )
        )
    } else {
        Ok(target_sobj)
    }
}

pub fn try_update_ship_velocity<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    velocity: &mut StellarObjectVelocity,
    controller: &PlayerShipController,
    ship_object: &Ship,
) -> Result<(), String> {
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

pub fn remove_old_timers<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    controller: &PlayerShipController,
    ship_object: &Ship,
) -> Result<(), String> {
    if !controller.mining_laser_on {
        for mining_timer in dsl.get_ship_mining_timers_by_ship_sobj_id(ship_object.get_sobj_id()) {
            info!(
                "Player {} stopped trying to mine a asteroid: {}",
                get_username(&dsl, controller.id),
                mining_timer.get_asteroid_sobj_id()
            );
            dsl.delete_ship_mining_timer_by_id(&mining_timer)?;
            return Ok(());
        }
    }

    Ok(())
}
