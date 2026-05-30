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
    let player_ship_id = get_player_ship(ctx)
        .ok_or("Player doesn't control a stellar object yet!")?
        .sobj_id;
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
