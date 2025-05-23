
use macroquad::{ math::Vec2, prelude::{collections::storage, *} };

use crate::module_bindings::*;
use spacetimedb_sdk::*;

use crate::stdb::utils::*;

use super::{resources::Resources, state::GameState};

fn draw_ship(transform: &StellarObjectTransformHiRes, game_state: &mut GameState, ship_type: ShipTypeDefinition) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();

    if let Some(player) = get_current_player(game_state.ctx) {
        let string = format!("{}", player.username);
        draw_text_ex(&string, position.x, position.y-32.0, TextParams {
            font_size: 16,
            color: WHITE,
            ..TextParams::default()
        });
    }

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
}

