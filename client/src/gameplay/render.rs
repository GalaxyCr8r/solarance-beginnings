use std::f32::consts::PI;

use macroquad::{ miniquad::date::now, prelude::{ collections::storage, * } };

use crate::module_bindings::*;
use spacetimedb_sdk::*;

use crate::stdb::utils::*;

use super::{ resources::Resources, state::GameState };

pub fn sector(game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let sun = &resources.sun_textures["star.1"];
    draw_texture(sun, sun.width() * -0.5, sun.height() * -0.5, WHITE);

    let mut player_transform = None;
    let mut player_ship_type = None;
    let player_ship = get_player_ship(game_state.ctx);

    let db = &game_state.ctx.db;
    for object in db.stellar_object().iter() {
        // Don't continue if isn't in the same sector.
        if player_ship.as_ref().is_some_and(|ship_obj| ship_obj.sector_id != object.sector_id) {
            continue;
        }

        if let Ok(transform) = get_transform(game_state.ctx, object.id) {
            if let Some(ship_object) = db.ship().sobj_id().find(&transform.sobj_id) {
                if let Some(ship_type) = db
                    .ship_type_definition()
                    .id()
                    .find(&ship_object.shiptype_id)
                {
                    if player_ship.as_ref().is_some_and(|ship_obj| ship_obj.sobj_id == object.id) {
                        // Store for later use
                        player_transform = Some(transform);
                        player_ship_type = Some(ship_type);
                    } else {
                        draw_ship(&transform, ship_type, game_state);
                    }
                }
            } else if let Some(jumpgate) = db.jump_gate().sobj_id().find(&object.id) {
                draw_jumpgate(&transform, jumpgate, game_state);
            } else if let Some(asteroid) = db.asteroid().sobj_id().find(&object.id) {
                draw_asteroid(&transform, asteroid, game_state);
            } else if let Some(cargo_crate) = db.cargo_crate().sobj_id().find(&object.id) {
                draw_crate(&transform, cargo_crate, game_state);
            }
        }
    }

    if let Some(controller) = db.player_ship_controller().player_id().find(&game_state.ctx.identity()) {
        if player_transform.is_none() || player_ship_type.is_none() {
            return;
        }
        let actual_player_transform = player_transform.unwrap();

        // Draw a line to your target from your ship, but only if it's far enough away.
        if let Some(target) = &game_state.current_target_sobj {
            if let Ok(transform) = get_transform(&game_state.ctx, target.id) {
                let player_vec = actual_player_transform.to_vec2();
                let target_vec = transform.to_vec2();
                if player_vec.distance(target_vec) > 300. {
                    let angle = (target_vec-player_vec).to_angle();
                    let from = player_vec + (glam::Vec2::from_angle(angle) * 150.0);
                    let to = target_vec + (glam::Vec2::from_angle(angle + PI) * 150.0);
                    draw_line(from.x, from.y, to.x, to.y, 1.337, Color::from_rgba(255, 255, 255, 200));
                }
            }
        }

        if controller.mining_laser_on {
            if let Some(target) = &game_state.current_target_sobj {
                // Draw a mining laser effect if the player is in range
                if target.kind == StellarObjectKinds::Asteroid {
                    if let Ok(transform) = get_transform(game_state.ctx, target.id) {
                        draw_line(
                            transform.x,
                            transform.y,
                            actual_player_transform.x,
                            actual_player_transform.y,
                            6.0,
                            Color::from_rgba(128, 0, 0, ((now() * 100.0) % 255.0) as u8)
                        );
                        draw_line(
                            transform.x,
                            transform.y,
                            actual_player_transform.x,
                            actual_player_transform.y,
                            (now() as f32 * 100.0) % 3.0,
                            RED
                        );
                    }
                }
            }
        }

        // Draw the controlled ship so its always on top.
        draw_ship(&actual_player_transform, player_ship_type.unwrap(), game_state);
    }
}

fn draw_ship(
    transform: &StellarObjectTransformHiRes,
    ship_type: ShipTypeDefinition,
    game_state: &mut GameState
) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();

    if let Some(player) = get_current_player(game_state.ctx) {
        let string = format!("{}", player.username);
        draw_text_ex(&string, position.x, position.y - 32.0, TextParams {
            font_size: 16,
            color: WHITE,
            ..TextParams::default()
        });
    }

    let tex = &resources.ship_textures[ship_type.gfx_key.unwrap().as_str()];
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

    if let Some(target) = &game_state.current_target_sobj {
        if target.id == transform.sobj_id {
            let size = (tex.width() + tex.height()) * 0.5;
            draw_targeting_bracket(position, size, Color::from_rgba(255, 255, 255, 200));
        }
    }
}

fn draw_asteroid(
    transform: &StellarObjectTransformHiRes,
    asteroid: Asteroid,
    game_state: &mut GameState
) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();
    let angle = position.x + (((now() * 8.0 + (position.y as f64)) % 360.0).to_radians() as f32); // Make the rotation based on position and time

    let tex =
        &resources.asteroid_textures[asteroid.gfx_key.unwrap_or("asteroid.1".to_string()).as_str()];
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

    // Targeting bracket
    if let Some(target) = &game_state.current_target_sobj {
        if target.id == asteroid.sobj_id {
            let size = (tex.width() + tex.height()) * 0.5;
            draw_targeting_bracket(position, size, Color::from_rgba(255, 255, 255, 200));
        }
    }
}

fn draw_crate(
    transform: &StellarObjectTransformHiRes,
    cargo_crate: CargoCrate,
    game_state: &mut GameState
) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();
    let angle = position.x + (((now() * 8.0 + (position.y as f64)) % 360.0).to_radians() as f32); // Make the rotation based on position and time

    let tex =
        &resources.asteroid_textures[cargo_crate.gfx_key.unwrap_or("crate.0".to_string()).as_str()];
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

    if let Some(target) = &game_state.current_target_sobj {
        if target.id == cargo_crate.sobj_id {
            let size = (tex.width() + tex.height()) * 0.5;
            draw_targeting_bracket(position, size, Color::from_rgba(255, 255, 255, 200));
        }
    }
}

fn draw_jumpgate(
    transform: &StellarObjectTransformHiRes,
    jumpgate: JumpGate,
    game_state: &mut GameState
) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();

    let tex =
        &resources.jumpgate_textures
            [jumpgate.gfx_key.unwrap_or("jumpgate_north".to_string()).as_str()];
    draw_texture(tex, position.x - tex.width() * 0.5, position.y - tex.height() * 0.5, WHITE);

    if let Some(target) = &game_state.current_target_sobj {
        if target.id == jumpgate.sobj_id {
            let size = (tex.width() + tex.height()) * 0.33;
            draw_targeting_bracket(position, size, Color::from_rgba(255, 255, 255, 200));
        }
    }
}

fn draw_targeting_bracket(pos: glam::Vec2, size: f32, color: Color) {
    draw_hexagon(
        pos.x,
        pos.y,
        size,
        if size < 512.0 {
            1.0
        } else {
            size / 512.0
        },
        true,
        color,
        Color::from_rgba(0, 0, 0, 0)
    );
}
