
use macroquad::{miniquad::date::now, prelude::{collections::storage, *}};

use crate::module_bindings::*;
use spacetimedb_sdk::*;

use crate::stdb::utils::*;

use super::{resources::Resources, state::GameState};

pub fn sector(game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let sun = resources.sun_textures["star.1"];
    draw_texture(sun, sun.width() * -0.5, sun.height() * -0.5, WHITE);
    
    let db = &game_state.ctx.db;
    for object in db.stellar_object().iter() {
        if object.sector_id != 0 {
            continue;
        }
        if let Ok(transform) = get_transform(game_state.ctx, object.id) {
            if let Some(ship_object) = db.ship_object().sobj_id().find(&transform.sobj_id) {
                if let Some(ship_instance) = db.ship_instance().id().find(&ship_object.ship_id) {
                    if let Some(ship_type) = db.ship_type_definition().id().find(&ship_instance.shiptype_id) {
                        draw_ship(&transform, game_state, ship_type);
                    }
                }
            } else if let Some(jumpgate) = db.jump_gate().sobj_id().find(&object.id) {
                draw_jumpgate(&transform, jumpgate);
            } else if let Some(asteroid) = db.asteroid().sobj_id().find(&object.id) {
                draw_asteroid(&transform, asteroid);
            } else if let Some(cargo_crate) = db.cargo_crate().sobj_id().find(&object.id) {
                draw_crate(&transform, cargo_crate);
            }
        }
    }
}

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

fn draw_asteroid(transform: &StellarObjectTransformHiRes, asteroid: Asteroid) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();
    let angle = position.x + ((((now() * 8.0) + position.y as f64) % 360.0).to_radians() as f32);  // Make the rotation based on position and time

    let tex = resources.asteroid_textures[asteroid.gfx_key.unwrap_or("asteroid.1".to_string()).as_str()];
    draw_texture_ex(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
        DrawTextureParams {
            rotation: angle,
            ..DrawTextureParams::default()
        }
    );
}

fn draw_crate(transform: &StellarObjectTransformHiRes, cargo_crate: CargoCrate) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();
    let angle = position.x + ((((now() * 8.0) + position.y as f64) % 360.0).to_radians() as f32); // Make the rotation based on position and time

    let tex = resources.asteroid_textures[cargo_crate.gfx_key.unwrap_or("crate.0".to_string()).as_str()];
    draw_texture_ex(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
        DrawTextureParams {
            rotation: angle,
            ..DrawTextureParams::default()
        }
    );
}

fn draw_jumpgate(transform: &StellarObjectTransformHiRes, jumpgate: JumpGate) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();

    let tex = resources.jumpgate_textures[jumpgate.gfx_key.unwrap_or("jumpgate_north".to_string()).as_str()];
    draw_texture(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE
    );
}
