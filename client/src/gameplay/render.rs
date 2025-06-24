use macroquad::{
    miniquad::date::now,
    prelude::{collections::storage, *},
};

use crate::{gameplay::render::star_system::render_star_system, module_bindings::*};
use spacetimedb_sdk::*;

use crate::stdb::utils::*;

use super::{resources::Resources, state::GameState};

pub mod star_system;

pub fn sector(game_state: &mut GameState) {
    let mut player_transform = None;
    let mut player_ship_type = None;
    let player_ship = get_player_ship(game_state.ctx);

    let mut local_targets: Vec<(u64, glam::Vec2, StellarObjectKinds)> = Vec::new();

    info!("6.1");
    set_camera(&game_state.bg_camera);
    info!("6.2");
    render_star_system(game_state, player_ship.clone());
    info!("6.3");
    set_camera(&game_state.camera);
    info!("6.4");

    let db = &game_state.ctx.db;
    for object in db.stellar_object().iter() {
        // Don't continue if isn't in the same sector.
        // if player_ship.as_ref().is_some_and(|ship_obj| ship_obj.sector_id != object.sector_id) {
        //     continue;
        // }

        // ONLY draw if they have hi-resolution positions.
        if let Some(transform) = db.sobj_hi_res_transform().sobj_id().find(&object.id) {
            if let Some(ship_object) = db.ship().sobj_id().find(&transform.sobj_id) {
                if let Some(ship_type) = db
                    .ship_type_definition()
                    .id()
                    .find(&ship_object.shiptype_id)
                {
                    if player_ship
                        .as_ref()
                        .is_some_and(|ship_obj| ship_obj.sobj_id == object.id)
                    {
                        // Store player transform for later use
                        player_transform = Some(transform.clone());
                        player_ship_type = Some(ship_type);
                    } else {
                        draw_ship(&ship_object, &transform, ship_type, game_state);
                    }
                }
            } else if let Some(station) = db.station().sobj_id().find(&object.id) {
                draw_station(&transform, station, game_state);
            } else if let Some(jumpgate) = db.jump_gate().sobj_id().find(&object.id) {
                draw_jumpgate(&transform, jumpgate, game_state);
            } else if let Some(asteroid) = db.asteroid().sobj_id().find(&object.id) {
                draw_asteroid(&transform, asteroid, game_state);
            } else if let Some(cargo_crate) = db.cargo_crate().sobj_id().find(&object.id) {
                draw_crate(&transform, cargo_crate, game_state);
            }
            local_targets.push((object.id, transform.clone().to_vec2(), object.kind));
        } else if let Some(transform) = db.sobj_low_res_transform().sobj_id().find(&object.id) {
            // Draw icon even if it has a low-res transform.
            local_targets.push((object.id, transform.clone().to_vec2(), object.kind));
        }
    }

    if let Some(controller) = db
        .player_ship_controller()
        .player_id()
        .find(&game_state.ctx.identity())
    {
        if player_transform.is_none() || player_ship_type.is_none() || player_ship.is_none() {
            return;
        }
        let actual_player_transform = player_transform.unwrap();
        let player_vec = actual_player_transform.to_vec2();

        draw_mining_laser(game_state, &actual_player_transform, controller);

        // Draw the controlled ship so its always on top.
        draw_ship(
            &player_ship.unwrap(),
            &actual_player_transform,
            player_ship_type.unwrap(),
            game_state,
        );

        // Draw 'radar'
        draw_radar(
            game_state,
            local_targets,
            actual_player_transform,
            player_vec,
        );
    }
}

fn draw_mining_laser(
    game_state: &mut GameState<'_>,
    actual_player_transform: &StellarObjectTransformHiRes,
    controller: PlayerShipController,
) {
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
                        Color::from_rgba(128, 0, 0, ((now() * 100.0) % 255.0) as u8),
                    );
                    draw_line(
                        transform.x,
                        transform.y,
                        actual_player_transform.x,
                        actual_player_transform.y,
                        ((now() as f32) * 100.0) % 3.0,
                        RED,
                    );
                }
            }
        }
    }
}

fn draw_radar(
    game_state: &mut GameState<'_>,
    local_targets: Vec<(u64, glam::Vec2, StellarObjectKinds)>,
    actual_player_transform: StellarObjectTransformHiRes,
    player_vec: glam::Vec2,
) {
    let radar_radius = screen_height() / 2.0 - 100.0;
    let radar_icon_size = 8.0;
    draw_circle_lines(
        actual_player_transform.x,
        actual_player_transform.y,
        radar_radius,
        radar_icon_size,
        Color::from_rgba(255, 255, 255, 32),
    );

    for (sobj_id, position, kind) in local_targets {
        // Find out where the icon should be placed on the ring.
        let angle = (position - player_vec).to_angle();
        let from =
            player_vec + (glam::Vec2::from_angle(angle) * radar_radius + radar_icon_size / 2.0);

        // Check targetted Stellar Object information (if there is any)
        let targetted_sobj = if game_state.current_target_sobj.is_some() {
            Some(game_state.current_target_sobj.as_ref().unwrap().id)
        } else {
            None
        };
        let is_targetted = targetted_sobj.is_some_and(|id| id == sobj_id);
        let thickness = if is_targetted { 2.0 } else { 1.0 };

        // Get distance to target and calculate various things based on it.
        let dist_sq = player_vec.distance_squared(position);
        if dist_sq < radar_radius * radar_radius {
            continue;
        }
        let distance_fade = if dist_sq < 1000.0 * 1000.0 {
            1.0
        } else {
            if dist_sq < 5000.0 * 5000.0 {
                ((6000.0 - f32::sqrt(dist_sq)) / 5000.0) * 0.75 + 0.25
            } else {
                0.25
            }
        };
        let actual_fade = if is_targetted {
            (255.0 * distance_fade) as u8
        } else {
            (192.0 * distance_fade) as u8
        };

        // Actually draw the icon.
        draw_poly_lines(
            from.x,
            from.y,
            polygon_points_per_kind(kind),
            radar_icon_size,
            1.0,
            thickness,
            Color::from_rgba(255, 255, 255, actual_fade),
        );
    }
}

fn draw_ship(
    ship: &Ship,
    transform: &StellarObjectTransformHiRes,
    ship_type: ShipTypeDefinition,
    game_state: &mut GameState,
) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();

    if let Some(player) = get_player(&game_state.ctx.db, &ship.player_id) {
        let string = format!("{}", player.username);
        draw_text_ex(
            &string,
            position.x,
            position.y - 32.0,
            TextParams {
                font_size: 16,
                color: WHITE,
                ..TextParams::default()
            },
        );
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
        },
    );

    if let Some(target) = &game_state.current_target_sobj {
        if target.id == transform.sobj_id {
            let size = (tex.width() + tex.height()) * 0.5;
            draw_targeting_bracket(
                position,
                size,
                target.kind,
                Color::from_rgba(255, 255, 255, 200),
            );
        }
    }
}

fn draw_asteroid(
    transform: &StellarObjectTransformHiRes,
    asteroid: Asteroid,
    game_state: &mut GameState,
) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();
    let angle = position.x + (((now() * 8.0 + (position.y as f64)) % 360.0).to_radians() as f32); // Make the rotation based on position and time

    let tex = &resources.asteroid_textures[asteroid
        .gfx_key
        .unwrap_or("asteroid.1".to_string())
        .as_str()];
    draw_texture_ex(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
        DrawTextureParams {
            rotation: angle,
            ..DrawTextureParams::default()
        },
    );

    // Targeting bracket
    if let Some(target) = &game_state.current_target_sobj {
        if target.id == asteroid.sobj_id {
            let size = (tex.width() + tex.height()) * 0.5;
            draw_targeting_bracket(
                position,
                size,
                target.kind,
                Color::from_rgba(255, 255, 255, 200),
            );
        }
    }
}

fn draw_crate(
    transform: &StellarObjectTransformHiRes,
    cargo_crate: CargoCrate,
    game_state: &mut GameState,
) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();
    let angle = position.x + (((now() * 8.0 + (position.y as f64)) % 360.0).to_radians() as f32); // Make the rotation based on position and time

    let tex = &resources.asteroid_textures[cargo_crate
        .gfx_key
        .unwrap_or("crate.0".to_string())
        .as_str()];
    draw_texture_ex(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
        DrawTextureParams {
            rotation: angle,
            ..DrawTextureParams::default()
        },
    );

    if let Some(target) = &game_state.current_target_sobj {
        if target.id == cargo_crate.sobj_id {
            let size = (tex.width() + tex.height()) * 0.5;
            draw_targeting_bracket(
                position,
                size,
                target.kind,
                Color::from_rgba(255, 255, 255, 200),
            );
        }
    }
}

fn draw_jumpgate(
    transform: &StellarObjectTransformHiRes,
    jumpgate: JumpGate,
    game_state: &mut GameState,
) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();

    let tex = &resources.jumpgate_textures[jumpgate
        .gfx_key
        .unwrap_or("jumpgate_north".to_string())
        .as_str()]; // TODO un-hardcode this
    draw_texture(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
    );

    if let Some(target) = &game_state.current_target_sobj {
        if target.id == jumpgate.sobj_id {
            let size = (tex.width() + tex.height()) * 0.33;
            draw_targeting_bracket(
                position,
                size,
                target.kind,
                Color::from_rgba(255, 255, 255, 200),
            );
        }
    }
}

fn draw_station(
    transform: &StellarObjectTransformHiRes,
    station: Station,
    game_state: &mut GameState,
) {
    let resources = storage::get::<Resources>();
    let position = transform.to_vec2();

    let gfx_key = match station.size {
        StationSize::Capital => "station.capital",
        StationSize::Large => "station.large",
        StationSize::Medium => "station.medium",
        StationSize::Small => "station.small",
        StationSize::Outpost => "station.outpost",
        StationSize::Satellite => "station.satellite",
    };
    let tex = &resources.station_textures[gfx_key];
    draw_texture(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
    );

    if let Some(target) = &game_state.current_target_sobj {
        if target.id == station.sobj_id {
            let size = (tex.width() + tex.height()) * 0.33;
            draw_targeting_bracket(
                position,
                size,
                target.kind,
                Color::from_rgba(255, 255, 255, 200),
            );
        }
    }
}

fn draw_targeting_bracket(pos: glam::Vec2, size: f32, kind: StellarObjectKinds, color: Color) {
    draw_poly_lines(
        pos.x,
        pos.y,
        polygon_points_per_kind(kind),
        size,
        1.0,
        if size < 512.0 { 1.0 } else { size / 512.0 },
        color,
    );
}

fn polygon_points_per_kind(kind: StellarObjectKinds) -> u8 {
    match kind {
        StellarObjectKinds::Ship => 3,
        StellarObjectKinds::Asteroid => 7,
        StellarObjectKinds::CargoCrate => 4,
        StellarObjectKinds::Station => 6,
        StellarObjectKinds::JumpGate => 5,
    }
}
