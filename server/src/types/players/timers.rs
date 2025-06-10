use std::f32::consts::PI;

use glam::Vec2;
use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::types::{players::{get_username, PlayerController}, ships::{timers::{create_mining_timer_for_ship, DeleteShipMiningTimerRowByScheduledId, GetShipMiningTimerRowsByShipSobjId}, *}, stellarobjects::*};

use super::GetPlayerControllerRowOptionByIdentity;

#[dsl(plural_name = player_controller_timers)]
#[spacetimedb::table(name = player_controller_timer, scheduled(player_controller_upkeep))]
pub struct PlayerControllerTimer {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    pub player: Identity
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx); // Waiting for DSL implementation of timers

    // Timers are created when the Player Controller is created.

    Ok(())
}

//////////////////////////////////////////////////////////////
// Reducers
//////////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn player_controller_upkeep(ctx: &ReducerContext, timer: PlayerControllerTimer) -> Result<(), String> {
  let dsl = dsl(ctx);

  //info!("Player con upkeep!");

  let controller = dsl.get_player_controller_by_identity(&timer.player)
    .ok_or(format!("Failed to find the player's controller! ID:{}", timer.player))?;

  let ship_object = dsl.get_ship_object_by_sobj_id(StellarObjectId::new(controller.get_stellar_object_id().unwrap()))
    .ok_or(format!("Failed to find the player's ship object! ID:{}", timer.player))?;
  let mut velocity = dsl.get_sobj_velocity_by_sobj_id(ship_object.get_sobj_id())
    .ok_or(format!("Failed to find the player's ship velocity! ID:{}", timer.player))?;

  if !controller.left && !controller.right {
    velocity.rotation_radians *= 0.875; // TODO:: Milestone 10+ make these ship-type specific values.
  }
  if !controller.up && !controller.down {
    // Add inertia to the velocity
    velocity=velocity.from_vec2(velocity.to_vec2() * 0.975); // TODO:: Milestone 10+ make these ship-type specific values.
    if velocity.to_vec2().length() < 0.042 {
        velocity = velocity.from_vec2(Vec2::ZERO);
    }
  }

  try_mining(ctx, &controller, &ship_object)?;

  if !controller.left && !controller.right && !controller.up && !controller.down {
    dsl.update_sobj_velocity_by_sobj_id(velocity)?;
    return Ok(()) // Bail out early, nothing else to change!
  }
  //info!("Player changes found!");

  let ship_instance = dsl.get_ship_instance_by_id(ship_object.get_ship_id())
    .ok_or(format!("Failed to find the player's ship instance! ID:{}", timer.player))?;
  let ship_type = dsl.get_ship_type_definition_by_id(ship_instance.get_shiptype_id())
    .ok_or(format!("Failed to find the player's ship type defintion! ID:{}", timer.player))?;
  let transform = dsl.get_sobj_internal_transform_by_sobj_id(ship_object.get_sobj_id())
    .ok_or(format!("Failed to find the player's ship transform! ID:{}", timer.player))?;

  // Based on the controller's settings and the ship definition and ship status, update the velocity.
  if controller.up {
    let mut vec = (Vec2::from_angle(transform.rotation_radians) * ship_type.base_acceleration) + velocity.to_vec2();
      
    // Check if the absolute velocity is too fast for the ship.
    if vec.length() > ship_type.base_speed {
        // Set the velocity
        vec = vec.normalize() * ship_type.base_speed;
    }

    velocity = velocity.from_vec2(vec);
  }
  if controller.right {
    velocity.rotation_radians = PI * ship_type.base_turn_rate;
  }
  if controller.left {
    velocity.rotation_radians = PI * -ship_type.base_turn_rate;
  }
  if controller.down {
    let mul = 0.965f32;
    velocity.x *= mul;
    velocity.y *= mul;
  }

  dsl.update_sobj_velocity_by_sobj_id(velocity)?;

  Ok(())
}

fn try_mining(ctx: &ReducerContext, controller: &PlayerController, ship_object: &ShipObject) -> Result<(), String> {
  let dsl = dsl(ctx);

  if !controller.mining_laser_on {
    for mining_timer in dsl.get_ship_mining_timers_by_ship_sobj_id(ship_object.get_sobj_id()) {
      info!("Player {} stopped trying to mine a asteroid: {}", get_username(ctx, controller.identity), mining_timer.asteroid_sobj_id);
      dsl.delete_ship_mining_timer_by_scheduled_id(&mining_timer);
      return Ok(())
    }
  }

  // If the player is trying to mine and is targetting an asteroid, create a mining timer.
  if controller.mining_laser_on && controller.targetted_sobj_id.is_some() {
  //if controller.current_action == CurrentAction::Mining && controller.targetted_sobj_id.is_some() { // Use if we move away from targetted_sobj_id
    let asteroid_sobj = dsl.get_stellar_object_by_id(StellarObjectId::new(controller.targetted_sobj_id.unwrap()))
      .ok_or(format!("Player {} tried to mine a non-existent object: {}", get_username(ctx, controller.identity), controller.targetted_sobj_id.unwrap()))?;

    if asteroid_sobj.kind != StellarObjectKinds::Asteroid {
      return Err(format!("Player {} tried to mine a non-asteroid object: {}", get_username(ctx, controller.identity), asteroid_sobj.id));
    }

    if let Some(transform) = dsl.get_sobj_internal_transform_by_sobj_id(ship_object.get_sobj_id()) {
      if let Some(ast_transform) = dsl.get_sobj_internal_transform_by_sobj_id(asteroid_sobj.get_id()) {
        if transform.to_vec2().distance(ast_transform.to_vec2()) < 500.0 {
          // Check if the player is already mining this asteroid
          if !dsl.get_ship_mining_timers_by_ship_sobj_id(&ship_object.get_sobj_id()).any(|timer| timer.asteroid_sobj_id == asteroid_sobj.id) {
            // Only add if there is no mining timer for this ship and asteroid.
            info!("Player {} started mining asteroid {}!", get_username(ctx, controller.identity), asteroid_sobj.id);
            let _ = create_mining_timer_for_ship(ctx, &ship_object.get_sobj_id(), &asteroid_sobj.get_id())?;
          }
        } else {
          // TODO: Let player know they're too far away?
        }
      }
    }
  }

  Ok(())
}