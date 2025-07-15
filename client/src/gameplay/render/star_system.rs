use std::f32::consts::PI;

use glam::Vec2;

use super::*;

pub fn render_star_system(game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let multiplier = 100.0;

    let stage_one_distance_squared = screen_height() * screen_height();
    let stage_two_distance_squared = stage_one_distance_squared * 1000.0;
    let stage_three_distance_squared = stage_one_distance_squared * 10000.0;

    let camera = game_state.bg_camera.target;

    for sso in game_state.ctx.db().star_system_object().iter() {
        let (image, secondary) = match sso.kind {
            StarSystemObjectKind::Star => (&resources.sun_textures["star.1"], None),
            StarSystemObjectKind::Planet =>
                (
                    &resources.planet_textures
                        [sso.gfx_key.clone().unwrap_or("planet.1".to_string()).as_str()],
                    Some(&resources.planet_textures["planet.shadow.1"]),
                ),
            StarSystemObjectKind::Moon =>
                (
                    &resources.planet_textures["moon.1"],
                    Some(&resources.planet_textures["planet.shadow.1"]),
                ),
            StarSystemObjectKind::AsteroidBelt => {
                continue;
            }
            StarSystemObjectKind::NebulaBelt => {
                continue;
            }
        };

        draw_star_system_object(
            multiplier,
            stage_one_distance_squared,
            stage_two_distance_squared,
            stage_three_distance_squared,
            camera,
            sso,
            image,
            secondary
        );
    }
}

fn draw_star_system_object(
    multiplier: f32,
    stage_one_distance_squared: f32,
    stage_two_distance_squared: f32,
    stage_three_distance_squared: f32,
    camera: Vec2,
    sso: StarSystemObject,
    image: &Texture2D,
    secondary: Option<&Texture2D>
) {
    let mut vec = Vec2::from_angle(sso.rotation_or_width_km) * sso.orbit_au * multiplier;
    let dist = vec.distance_squared(camera);
    let mut scale = 1.0;

    if dist < stage_one_distance_squared {
        // Stage 1: Normal
    } else if dist < stage_two_distance_squared {
        // Stage 2: Zoom out
        let angle = (vec - camera).to_angle();

        vec = camera + Vec2::from_angle(angle) * stage_one_distance_squared.sqrt();
        scale = (stage_one_distance_squared.sqrt() / dist.sqrt()) * 0.75 + 0.25;
        // if is_key_pressed(KeyCode::Space) {
        //     info!("{:?}: Stage 2 {}, {} [Angle: {}]", sso.kind, vec.x, vec.y, angle);
        // }
    } else if dist < stage_three_distance_squared {
        // TODO: Slow slide off the screen
    }

    let mut params = DrawTextureParams::default();
    params.rotation = (((now() * 0.01f64) as f32) % 2.0) * PI;
    params.dest_size = Some(Vec2::new(image.width() * scale, image.height() * scale));

    draw_texture_ex(
        &image,
        image.width() * -0.5 * scale + vec.x,
        image.height() * -0.5 * scale + vec.y,
        WHITE,
        params
    );

    if let Some(shadow) = secondary {
        let sun_angle = vec.to_angle();
        let scale_adjust = if sso.kind == StarSystemObjectKind::Planet { 0.85 } else { 0.21 };

        let params = DrawTextureParams {
            rotation: sun_angle - PI / 4.0,
            dest_size: Some(
                Vec2::new(
                    shadow.width() * scale * scale_adjust,
                    shadow.height() * scale * scale_adjust
                )
            ),
            ..Default::default()
        };

        draw_texture_ex(
            shadow,
            shadow.width() * -0.5 * scale * scale_adjust + vec.x,
            shadow.height() * -0.5 * scale * scale_adjust + vec.y,
            WHITE,
            params
        );
        //
    }
}
