use std::{ collections::HashMap, sync::mpsc::{self, Sender} };

use macroquad::{ math::Vec2, prelude::*, ui };

use crate::module_bindings::*;
use spacetimedb_sdk::{ DbContext, Table };
use crate::stdb::connector::connect_to_spacetime;

use crate::{shader::*, stdb::utils::*};

use super::state::GameState;


fn draw_ship(transform: &StellarObjectTransform, game_state: &mut GameState) {
    let position = transform.to_vec2();
    let forward = Vec2::from_angle(transform.rotation_radians) * 16.0;

    let forward_pos = position + forward * 2.0;

    draw_line(position.x, position.y, forward_pos.x, forward_pos.y, 2.0, RED);

    let string = format!("Sobj{}", transform.sobj_id.to_string());
    draw_text_ex(&string, position.x, position.y, TextParams {
        font_size: 16,
        rotation: transform.rotation_radians,
        color: WHITE,
        ..TextParams::default()
    });

    let tex = game_state.textures["lc.phalanx"];
    draw_texture_ex(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
        DrawTextureParams {
            rotation: transform.rotation_radians,
            ..DrawTextureParams::default()
        }
    );
}

pub fn sector(game_state: &mut GameState) {
    // if game_state.paused {
    //     let text = "PAUSED";
    //     let font_size = 100.0;
    //     let text_width = measure_text(text, None, font_size as u16, 1.0).width;
    //     let (x, y) = ((screen_width() - text_width) / 2.0, screen_height() / 2.0);

    //     draw_text(text, x, y, font_size, RED);

    //     return;
    // }

    // TODO: Figure out how to get the player ship's position at the beginning so we can offset everything drawn by it.

    let sun = game_state.textures["star"];
    draw_texture(sun, sun.width() * -0.5, sun.height() * -0.5, WHITE);

    for object in game_state.ctx.db.stellar_object().iter() {
        if let Ok(transform) = get_transform(game_state.ctx, object.id) {
            draw_ship(&transform, game_state);
        }
    }

    draw_line(0.0, 0.0, game_state.camera.target.x, game_state.camera.target.y, 3.0, RED);
}

