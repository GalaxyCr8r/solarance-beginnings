use macroquad::prelude::*;
use spacetimedb_sdk::{DbContext, Table};

use crate::server::bindings::*;

use crate::stdb::utils::*;

use super::state::GameState;

pub fn control_player_ship(ctx: &DbConnection, game_state: &mut GameState) -> Result<(), String> {
    if game_state.chat_window.has_focus || ctx.try_identity().is_none() {
        return Ok(());
    }

    // Combat mode toggle
    if is_key_pressed(KeyCode::Q) {
        game_state.combat_mode = !game_state.combat_mode;
    }

    let forward  = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
    let backward = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);
    let left     = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
    let right    = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);

    let new_flags = (forward, backward, left, right);
    if game_state.movement_flags != new_flags {
        game_state.movement_flags = new_flags;
        let _ = ctx.reducers.update_ship_movement_controller(forward, backward, left, right);
    }

    Ok(())
}

pub fn target_closest_stellar_object(
    ctx: &DbConnection,
    game_state: &mut GameState,
) -> Result<StellarObject, String> {
    if game_state.chat_window.has_focus {
        return Err("Chat window has focus. Cannot target objects.".to_string());
    }

    //let player_id = ctx.identity();
    let player_ship_id =
        get_player_sobj_id(ctx).ok_or("Player doesn't control a stellar object yet!")?;
    let player_sobj = ctx
        .db
        .stellar_object()
        .id()
        .find(&player_ship_id)
        .ok_or("Player doesn't control a stellar object yet!")?;
    let player_transform = get_transform(ctx, player_ship_id)?.to_vec2();

    let mut closest_distance = f32::MAX;
    let mut closest_sobj = Option::None;

    for sobj in ctx.db().stellar_object().iter() {
        if sobj.id == player_ship_id || sobj.sector_id != player_sobj.sector_id {
            continue; // Skip the player's ship and non-sector objects
        }
        if let Ok(transform) = get_transform(ctx, sobj.id) {
            let distance = transform.to_vec2().distance_squared(player_transform);
            if distance < closest_distance {
                closest_distance = distance;
                closest_sobj = Some(sobj);
            }
        }
    }

    if let Some(sobj) = closest_sobj {
        match sobj.kind {
            // None => {
            //     info!("Could not find type for stellar object: {}", sobj.id);
            //     Err("Could not find type for targeted stellar object.".to_string())
            // },
            _ => {
                info!("Targeted closest {:?}: {}", sobj.kind, sobj.id);
                Ok(sobj)
            }
        }
    } else {
        info!("No stellar objects found to target.");
        Err("Could not find a stellar object to target.".to_string())
    }
}

// pub fn _control_player_ship(ctx: &DbConnection, game_state: &mut GameState) -> Result<(), String> {
//     if game_state.chat_window.has_focus {
//         return Ok(())
//     }

//     let controlled_entity_id = get_player_sobj_id(ctx);
//     if controlled_entity_id.is_none() {
//         return Err("Player doesn't control a stellar object yet!".to_string());
//     }
//     let mut velocity = ctx.db
//         .sobj_velocity()
//         .sobj_id()
//         .find(&controlled_entity_id.unwrap())
//         .ok_or("Player's controlled object doesn't have a velocity table entry!")?;

//     let ship_object = ctx.db().ship_object().sobj_id().find(&velocity.sobj_id).ok_or("control player ship_object error")?;
//     let ship_instance = ctx.db().ship_instance().id().find(&ship_object.ship_id).ok_or("control player ship_instance error")?;
//     let ship_type = ctx.db().ship_type_definition().id().find(&ship_instance.shiptype_id).ok_or("control player ship_type error")?;

//     let vel = velocity.to_vec2();
//     let mut changed = false;
//     if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
//         velocity.rotation_radians = PI * ship_type.base_turn_rate;
//         changed = true;
//     }
//     if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
//         velocity.rotation_radians = PI * -ship_type.base_turn_rate;
//         changed = true;
//     }
//     if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
//         velocity = velocity.from_vec2(vel * 0.9);
//         changed = true;
//     }
//     if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
//         info!("Orig. Velocity: {}, {}", velocity.x, velocity.y);
//         let transform = get_transform(&ctx, velocity.sobj_id)?;
//         velocity = velocity.from_vec2(Vec2::from_angle(transform.rotation_radians) * ship_type.base_speed);
//         changed = true;
//         info!("Updated Velocity: {}, {}", velocity.x, velocity.y);
//     }

//     if !changed {
//         return Ok(());
//     }

//     ctx.reducers
//         .update_sobj_velocity(velocity)
//         .or_else(|err| Err(err.to_string()))
// }
