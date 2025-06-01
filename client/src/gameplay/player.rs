use std::f32::consts::PI;

use macroquad::{ math::Vec2, prelude::* };
use spacetimedb_sdk::DbContext;

use crate::module_bindings::*;

use crate::stdb::utils::*;

use super::state::GameState;

pub fn control_player_ship(ctx: &DbConnection, game_state: &mut GameState) -> Result<(), String> {
    if game_state.chat_window.has_focus {
        return Ok(())
    }
    let mut changed = false; // ONLY request an update if there's actually been a change!
    if let Some(mut controller) = ctx.db.player_controller().identity().find(&ctx.identity()) {
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            controller.right = true;
            controller.left = false;
            changed = true;
        } else {
            if controller.right {
                controller.right = false;
                changed = true;
            }
        }
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            controller.right = false;
            controller.left = true;
            changed = true;
        } else {
            if controller.left {
                controller.left = false;
                changed = true;
            }
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            controller.up = false;
            controller.down = true;
            changed = true;
        } else {
            if controller.down {
                controller.down = false;
                changed = true;
            }
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            controller.up = true;
            controller.down = false;
            changed = true;
        } else {
            if controller.up {
                controller.up = false;
                changed = true;
            }
        }

        if changed {
            ctx.reducers.update_player_controller(controller).or_else(|err| Err(err.to_string()))?;
        }
    } 

    Ok(())
}

pub fn _control_player_ship(ctx: &DbConnection, game_state: &mut GameState) -> Result<(), String> {
    if game_state.chat_window.has_focus {
        return Ok(())
    }

    let controlled_entity_id = get_player_sobj_id(ctx);
    if controlled_entity_id.is_none() {
        return Err("Player doesn't control a stellar object yet!".to_string());
    }
    let mut velocity = ctx.db
        .sobj_velocity()
        .sobj_id()
        .find(&controlled_entity_id.unwrap())
        .ok_or("Player's controlled object doesn't have a velocity table entry!")?;

    let ship_object = ctx.db.ship_object().sobj_id().find(&velocity.sobj_id).ok_or("control player ship_object error")?;
    let ship_instance = ctx.db.ship_instance().id().find(&ship_object.ship_id).ok_or("control player ship_instance error")?;
    let ship_type = ctx.db.ship_type_definition().id().find(&ship_instance.shiptype_id).ok_or("control player ship_type error")?;

    let vel = velocity.to_vec2();
    let mut changed = false;
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        velocity.rotation_radians = PI * ship_type.base_turn_rate;
        changed = true;
    }
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        velocity.rotation_radians = PI * -ship_type.base_turn_rate;
        changed = true;
    }
    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
        velocity = velocity.from_vec2(vel * 0.9);
        changed = true;
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
        info!("Orig. Velocity: {}, {}", velocity.x, velocity.y);
        let transform = get_transform(&ctx, velocity.sobj_id)?;
        velocity = velocity.from_vec2(Vec2::from_angle(transform.rotation_radians) * ship_type.base_speed);
        changed = true;
        info!("Updated Velocity: {}, {}", velocity.x, velocity.y);
    }

    if !changed {
        return Ok(());
    }

    ctx.reducers
        .update_sobj_velocity(velocity)
        .or_else(|err| Err(err.to_string()))
}
