
use macroquad::{ math::Vec2, prelude::{collections::storage, *} };

use crate::module_bindings::*;
use spacetimedb_sdk::*;

use crate::stdb::utils::*;

use super::{resources::Resources, state::GameState};


fn draw_ship(transform: &StellarObjectTransformHiRes, _game_state: &mut GameState, ship_type: ShipTypeDefinition) {
    let resources = storage::get::<Resources>();
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

    let tex = resources.ship_textures[ship_type.gfx_key.unwrap().as_str()];
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
    let resources = storage::get::<Resources>();
    // if game_state.paused {
    //     let text = "PAUSED";
    //     let font_size = 100.0;
    //     let text_width = measure_text(text, None, font_size as u16, 1.0).width;
    //     let (x, y) = ((screen_width() - text_width) / 2.0, screen_height() / 2.0);

    //     draw_text(text, x, y, font_size, RED);

    //     return;
    // }

    // TODO: Figure out how to get the player ship's position at the beginning so we can offset everything drawn by it.

    let sun = resources.sun_texture;
    draw_texture(sun, sun.width() * -0.5, sun.height() * -0.5, WHITE);
    
    let db = &game_state.ctx.db;
    for object in db.stellar_object().iter() {
        if let Ok(transform) = get_transform(game_state.ctx, object.id) {
            if let Some(ship_object) = db.ship_object().sobj_id().find(&transform.sobj_id) {
                if let Some(ship_instance) = db.ship_instance().id().find(&ship_object.ship_id) {
                    if let Some(ship_type) = db.ship_type_definition().id().find(&ship_instance.shiptype_id) {
                        draw_ship(&transform, game_state, ship_type);
                    }
                }
            }
        }
    }

    draw_line(0.0, 0.0, game_state.camera.target.x, game_state.camera.target.y, 3.0, RED);
}

