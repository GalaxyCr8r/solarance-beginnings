use macroquad::{
    miniquad::date::now,
    prelude::{collections::storage, *},
};

use crate::server::bindings::*;
use crate::stdb::utils::*;

use crate::gameplay::{resources::Resources, state::GameState};

pub fn draw_mining_laser(game_state: &mut GameState<'_>, player_pose: &RenderPose) {
    if !game_state.mining_active {
        return;
    }
    // Re-query the target fresh; `get_current_target` is the row we'd otherwise
    // have cached and dereferenced. None ⇒ target gone, nothing to draw.
    let Some(target) = get_current_target(game_state.ctx, &mut game_state.current_target_sobj_id)
    else {
        return;
    };
    if target.kind != StellarObjectKinds::Asteroid {
        return;
    }
    let now_micros = now_unix_micros();
    if let Some(target_pose) = pose_for_object(game_state.ctx, &target, now_micros) {
        draw_line(
            target_pose.pos.x,
            target_pose.pos.y,
            player_pose.pos.x,
            player_pose.pos.y,
            6.0,
            Color::from_rgba(128, 0, 0, ((now() * 100.0) % 255.0) as u8),
        );
        draw_line(
            target_pose.pos.x,
            target_pose.pos.y,
            player_pose.pos.x,
            player_pose.pos.y,
            ((now() as f32) * 100.0) % 3.0,
            RED,
        );
    }
}

pub fn draw_radar(
    game_state: &mut GameState<'_>,
    local_targets: Vec<(u64, glam::Vec2, StellarObjectKinds)>,
    player_pose: &RenderPose,
    player_vec: glam::Vec2,
) {
    let radar_radius = screen_height() / 2.0 - 100.0;
    let radar_icon_size = 12.0;
    draw_circle_lines(
        player_pose.pos.x,
        player_pose.pos.y,
        radar_radius - radar_icon_size,
        radar_icon_size * 2.0,
        Color::from_rgba(255, 255, 255, 32),
    );

    // Predicted-forward velocity for the player ship — the HUD reads
    // `velocity` (scalar speed) and `rotation` from the same snapshot used
    // to draw the ship so the indicators don't lag the sprite.
    if let Some((_, snapshot)) = predicted_player_snapshot(game_state.ctx) {
        let _ = draw_hud(game_state, radar_radius, &snapshot, player_pose, &player_vec);
    }

    for (sobj_id, position, kind) in local_targets {
        // Find out where the icon should be placed on the ring.
        let angle = (position - player_vec).to_angle();
        let from =
            player_vec + (glam::Vec2::from_angle(angle) * radar_radius + radar_icon_size / 2.0);

        let is_targetted = game_state.current_target_sobj_id == Some(sobj_id);
        let thickness = if is_targetted { 2.0 } else { 1.0 };

        let dist = player_vec.distance(position);
        if dist < radar_radius {
            continue;
        }

        let distance_fade = if dist < 1000.0 {
            1.0
        } else {
            if dist < 5000.0 {
                ((6000.0 - dist) / 5000.0) * 0.75 + 0.25
            } else {
                0.25
            }
        };

        let actual_fade = if is_targetted {
            (255.0 * distance_fade) as u8
        } else {
            (192.0 * distance_fade) as u8
        };

        let radius = radar_icon_size * distance_fade;
        if is_targetted {
            draw_poly(
                from.x,
                from.y,
                polygon_points_per_kind(kind),
                radius * 2.0,
                0.0,
                Color::from_rgba(255, 255, 255, 96),
            );
        }

        draw_poly_lines(
            from.x,
            from.y,
            polygon_points_per_kind(kind),
            radius + 1.0,
            1.0,
            thickness,
            Color::from_rgba(0, 0, 0, actual_fade),
        );
        draw_poly_lines(
            from.x,
            from.y,
            polygon_points_per_kind(kind),
            radius,
            1.0,
            thickness,
            Color::from_rgba(255, 255, 255, actual_fade),
        );
    }
}

/// Draws the velocity / heading needle around the player's ship. The
/// snapshot's `rotation` and `velocity` (scalar speed) replace the legacy
/// `StellarObjectVelocity`'s (x,y) vector — multiply speed by the heading
/// unit vector to get the velocity arrow.
pub fn draw_hud(
    _game_state: &mut GameState<'_>,
    radar_radius: f32,
    snapshot: &solarance_shared::MovementState,
    pose: &RenderPose,
    _player_vec: &glam::Vec2,
) -> Result<(), String> {
    let color = Color::from_rgba(255, 255, 255, 128);
    let position = pose.pos;

    let heading = glam::Vec2::from_angle(pose.rotation_radians);
    let point_forward = position + heading * radar_radius;
    let point_forward_mid = position + heading * (radar_radius - 32.0);

    draw_line(
        point_forward_mid.x,
        point_forward_mid.y,
        point_forward.x,
        point_forward.y,
        3.0,
        color,
    );

    let velocity_speed = snapshot.velocity.max(0.0);
    let velocity_dir = heading; // ships only move forward in this model
    let point_velocity_mid = position + velocity_dir * (radar_radius - 32.0);
    let point_velocity_low =
        position + velocity_dir * (radar_radius - 32.0 - velocity_speed);

    draw_line(
        point_velocity_mid.x,
        point_velocity_mid.y,
        point_velocity_low.x,
        point_velocity_low.y,
        3.0,
        color,
    );

    Ok(())
}

pub fn draw_ship(
    ship: &Ship,
    pose: &RenderPose,
    ship_type: &ShipTypeDefinition,
    game_state: &mut GameState,
) {
    let resources = storage::get::<Resources>();
    let position = pose.pos;

    if let Some(player) = game_state.ctx.db.player().id().find(&ship.player_id) {
        let string = format!(
            "[{}] {}",
            get_faction_shortname(game_state.ctx, &player.faction_id.value),
            player.username
        );
        let dimension = measure_text(&string, None, 16, 1.0);
        draw_text_ex(
            &string,
            position.x - dimension.width / 2.0,
            position.y - 32.0,
            TextParams {
                font_size: 16,
                color: WHITE,
                ..TextParams::default()
            },
        );
    }

    let tex = &resources.ship_textures[ship_type.gfx_key.clone().unwrap().as_str()];
    draw_texture_ex(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
        DrawTextureParams {
            rotation: pose.rotation_radians,
            ..DrawTextureParams::default()
        },
    );

    if game_state.current_target_sobj_id == Some(pose.sobj_id) {
        let size = (tex.width() + tex.height()) * 0.5;
        draw_targeting_bracket(
            position,
            size,
            StellarObjectKinds::Ship,
            Color::from_rgba(255, 255, 255, 200),
        );
    }
}

/// Decorative in-sector nebula (#107). Pure flavor: no pose, no radar entry,
/// no targeting — drawn under everything else in the sector pass.
pub fn draw_nebula(nebula: &SectorNebula) {
    let resources = storage::get::<Resources>();
    let Some(tex) = resources.nebula_textures.get(nebula.gfx_key.as_str()) else {
        return; // Unknown key — decorative, so silently skip rather than panic.
    };
    let w = tex.width() * nebula.scale;
    let h = tex.height() * nebula.scale;
    let t = nebula.tint; // 0xRRGGBBAA
    draw_texture_ex(
        tex,
        nebula.position.x - w * 0.5,
        nebula.position.y - h * 0.5,
        Color::from_rgba((t >> 24) as u8, (t >> 16) as u8, (t >> 8) as u8, t as u8),
        DrawTextureParams {
            rotation: nebula.rotation_radians,
            dest_size: Some(vec2(w, h)),
            ..DrawTextureParams::default()
        },
    );
}

pub fn draw_asteroid(pose: &RenderPose, asteroid: Asteroid, game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let position = pose.pos;
    let angle = pose.rotation_radians;

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

    if game_state.current_target_sobj_id == Some(asteroid.id) {
        let size = (tex.width() + tex.height()) * 0.5;
        draw_targeting_bracket(
            position,
            size,
            StellarObjectKinds::Asteroid,
            Color::from_rgba(255, 255, 255, 200),
        );
    }
}

pub fn draw_crate(pose: &RenderPose, cargo_crate: CargoCrate, game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let position = pose.pos;
    let angle = pose.rotation_radians;

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

    if game_state.current_target_sobj_id == Some(cargo_crate.sobj_id) {
        let size = (tex.width() + tex.height()) * 0.5;
        draw_targeting_bracket(
            position,
            size,
            StellarObjectKinds::CargoCrate,
            Color::from_rgba(255, 255, 255, 200),
        );
    }
}

pub fn draw_jumpgate(pose: &RenderPose, jumpgate: JumpGate, game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let position = pose.pos;

    let tex = &resources.jumpgate_textures[jumpgate
        .gfx_key
        .unwrap_or("jumpgate_north".to_string())
        .as_str()];
    draw_texture(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
    );

    if game_state.current_target_sobj_id == Some(jumpgate.id) {
        let size = (tex.width() + tex.height()) * 0.33;
        draw_targeting_bracket(
            position,
            size,
            StellarObjectKinds::JumpGate,
            Color::from_rgba(255, 255, 255, 200),
        );
    }
}

pub fn draw_station(pose: &RenderPose, station: Station, game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let position = pose.pos;

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

    if game_state.current_target_sobj_id == Some(station.sobj_id) {
        let size = (tex.width() + tex.height()) * 0.33;
        draw_targeting_bracket(
            position,
            size * 1.1,
            StellarObjectKinds::Station,
            Color::from_rgba(255, 255, 255, 200),
        );
    }
}

pub fn draw_targeting_bracket(pos: glam::Vec2, size: f32, kind: StellarObjectKinds, color: Color) {
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

pub fn polygon_points_per_kind(kind: StellarObjectKinds) -> u8 {
    match kind {
        StellarObjectKinds::Ship => 3,
        StellarObjectKinds::Asteroid => 7,
        StellarObjectKinds::CargoCrate => 4,
        StellarObjectKinds::Station => 6,
        StellarObjectKinds::JumpGate => 5,
    }
}
