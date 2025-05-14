use std::f32::consts::PI;

use macroquad::{ math::Vec2, prelude::*, ui };

use crate::module_bindings::*;

use crate::stdb::utils::*;

use super::state::GameState;

pub fn control_player_ship(ctx: &DbConnection, game_state: &mut GameState) -> Result<(), String> {
    if game_state.chat_window.has_focus {
        return Ok(())
    }

    let controlled_entity_id = get_player_sobj_id(ctx);
    if controlled_entity_id.is_none() {
        return Err("Player doesn't control a stellar object yet!".to_string());
    }
    let mut velocity = ctx.db
        .stellar_object_velocity()
        .sobj_id()
        .find(&controlled_entity_id.unwrap())
        .ok_or("Player's controlled object doesn't have a velocity table entry!")?;

    let vel = velocity.to_vec2();
    let mut changed = false;
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        velocity.rotation_radians += PI * 0.01337;
        changed = true;
    }
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        velocity.rotation_radians -= PI * 0.01337;
        changed = true;
    }
    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
        velocity = velocity.from_vec2(vel * 0.75);
        changed = true;
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
        info!("Orig. Velocity: {}, {}", velocity.x, velocity.y);
        let transform = get_transform(&ctx, velocity.sobj_id)?;
        velocity = velocity.from_vec2(Vec2::from_angle(transform.rotation_radians) * 200.0);
        changed = true;
        info!("Updated Velocity: {}, {}", velocity.x, velocity.y);
    }

    if !changed {
        return Ok(());
    }

    ctx.reducers
        .update_stellar_object_velocity(velocity)
        .or_else(|err| Err(err.to_string()))
}
