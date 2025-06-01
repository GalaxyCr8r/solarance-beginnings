use std::{f32::consts::PI, time::Duration};

use glam::Vec2;
use log::info;
use spacetimedb::{rand::Rng, *};
use spacetimedsl::*;

use crate::types::{ships::*, stellarobjects::*};

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

  info!("Player con upkeep!");

  let controller = dsl.get_player_controller_by_identity(&timer.player)
    .ok_or(format!("Failed to find the player's controller! ID:{}", timer.player))?;

  if !controller.left && !controller.right && !controller.up && !controller.up && !controller.down {
    return Ok(()) // Bail out early, nothing to change!
  }
  info!("Player changes found!");

  let ship_object = dsl.get_ship_object_by_sobj_id(StellarObjectId::new(controller.get_stellar_object_id().unwrap()))
    .ok_or(format!("Failed to find the player's ship object! ID:{}", timer.player))?;
  let ship_instance = dsl.get_ship_instance_by_id(ship_object.get_ship_id())
    .ok_or(format!("Failed to find the player's ship instance! ID:{}", timer.player))?;
  let ship_type = dsl.get_ship_type_definition_by_id(ship_instance.get_shiptype_id())
    .ok_or(format!("Failed to find the player's ship type defintion! ID:{}", timer.player))?;
  let mut velocity = dsl.get_sobj_velocity_by_sobj_id(ship_object.get_sobj_id())
    .ok_or(format!("Failed to find the player's ship velocity! ID:{}", timer.player))?;
  let transform = dsl.get_sobj_internal_transform_by_sobj_id(ship_object.get_sobj_id())
    .ok_or(format!("Failed to find the player's ship transform! ID:{}", timer.player))?;

  // Based on the controller's settings and the ship definition and ship status, update the velocity.
  if controller.up {
      velocity = velocity.from_vec2(Vec2::from_angle(transform.rotation_radians) * ship_type.base_speed);
  }
  if controller.right {
      velocity.rotation_radians = PI * ship_type.base_turn_rate;
  }
  if controller.left {
      velocity.rotation_radians = PI * -ship_type.base_turn_rate;
  }
  if controller.down {
      velocity = velocity.from_vec2(velocity.to_vec2() * 0.9);
  }

  dsl.update_sobj_velocity_by_sobj_id(velocity)?;

  Ok(())
}