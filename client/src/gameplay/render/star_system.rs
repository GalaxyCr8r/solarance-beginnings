use egui::Color32;
use glam::Vec2;

use super::*;

pub fn render_star_system(game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let multiplier = 100.0;

    let stage_one_distance_squared = screen_height() * screen_height();
    let stage_two_distance_squared = stage_one_distance_squared * 1000.0;
    let stage_three_distance_squared = stage_one_distance_squared * 10000.0;

    let camera = game_state.camera.target;

    for sso in game_state.ctx.db().star_system_object().iter() {
        let image = match sso.kind {
            StarSystemObjectKind::Star => { &resources.sun_textures["star.1"] }
            StarSystemObjectKind::Planet => { &resources.planet_textures["planet.1"] }
            StarSystemObjectKind::Moon => { &resources.planet_textures["moon.1"] }
            StarSystemObjectKind::AsteroidBelt => {
                continue;
            }
            StarSystemObjectKind::NebulaBelt => {
                continue;
            }
        };

        let mut vec = Vec2::from_angle(sso.rotation_or_width_km) * sso.orbit_au * multiplier;
        let dist = vec.distance_squared(camera);
        let mut scale = 1.0;

        if dist < stage_one_distance_squared {
            // Stage 1: Normal
        } else if dist < stage_two_distance_squared {
            // Stage 2: Zoom out
            let angle = (vec - camera).to_angle();
            //let extra_dist = dist - stage_one_distance_squared;
            vec = camera + Vec2::from_angle(angle) * stage_one_distance_squared.sqrt();
            scale = (stage_one_distance_squared.sqrt() / dist.sqrt()) * 0.75 + 0.25;
            if is_key_pressed(KeyCode::Space) {
                info!("{:?}: Stage 2 {}, {} [Angle: {}]", sso.kind, vec.x, vec.y, angle);
            }
        } else if dist < stage_three_distance_squared {
            // Slow slide
        }

        let mut params = DrawTextureParams::default();
        params.dest_size = Some(Vec2::new(image.width() * scale, image.height() * scale));

        draw_texture_ex(
            &image,
            image.width() * -0.5 * scale + vec.x,
            image.height() * -0.5 * scale + vec.y,
            WHITE,
            params
        );

        draw_text(format!("Dist: {} - Scale: {}", dist, scale).as_str(), vec.x, vec.y, 32.0, BLACK);
    }
}
