use std::{ env, f32::consts::PI, path::PathBuf, thread::{ self, JoinHandle } };

use dotenv::dotenv;
use egui::{ Align2, Button, Color32, Frame, RichText, Shadow };
use macroquad::{
    math::Vec2,
    prelude::{ collections::storage, coroutines::start_coroutine, * },
    time,
};
use spacetimedb_sdk::*;

use solarance_beginnings::{
    gameplay::{ self, render::star_system::*, resources::* },
    module_bindings::*,
    stdb::connector::*,
};

/// The purpose of this star system test is to test the faux-parallax effect of star systems for when
/// you move to different sectors. The background star, planets, asteroid/nebula belts, etc. will move
/// in proportion to where you go, based on their relative sizes like parallax, but have additional
/// caveats that normal parallax does not.
///
/// It breaks down into three modes:
/// 1. When the sector is on top or next to a SSO (star system object), it slides around at full scale until it reaches the screen edge.
/// 2. When at the screen edge, the further you go (for a set rate per SSO kind, e.g. slower for stars)
/// the smaller it will get before reaching a set scale (like 10%)
/// 3. Once it is the smallest it will get, it now moves slowly off screen, and when it is safely away, stopped rendering entirely.
#[macroquad::main("S:B Star System Test")]
async fn main() {
    request_new_screen_size(1280.0, 720.0);

    clear_background(BLACK);
    next_frame().await;

    set_pc_assets_folder("assets");

    clear_background(BLACK);
    next_frame().await;

    let ctx = connect_to_spacetime(None).ok_or("Could not connect in time.").unwrap();

    let resources = Resources::new().await.unwrap();
    storage::store(resources);

    let mut game_state = gameplay::state::initialize(&ctx);
    game_state.camera.zoom.y *= -1.0;
    let multiplier = 100.0;

    loop {
        clear_background(BLACK);
        set_camera(&game_state.camera);
        render_star_system(&mut game_state);

        {
            // Position Grid
            let mut target = game_state.camera.target.clone();
            target.x = (((target.x as i32) / 128) as f32) * 128.0;
            target.y = (((target.y as i32) / 128) as f32) * 128.0;
            draw_line(
                target.x - 256.0,
                target.y,
                target.x + 256.0,
                target.y,
                1.0,
                Color::from_rgba(128, 192, 255, 128)
            );
            draw_line(
                target.x,
                target.y - 256.0,
                target.x,
                target.y + 256.0,
                1.0,
                Color::from_rgba(128, 192, 255, 128)
            );
        }

        egui_macroquad::ui(|egui_ctx| {
            egui::Window
                ::new("Star System Test")
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .anchor(Align2::LEFT_CENTER, egui::Vec2::new(0.0, 0.0))
                .frame(
                    Frame::group(&egui_ctx.style())
                        .fill(Color32::from_rgba_unmultiplied(15, 15, 15, 245))
                        .shadow(Shadow::NONE)
                )
                .show(egui_ctx, |ui| {
                    ui.label(
                        format!(
                            "Current Position: {}, {}",
                            game_state.camera.target.x,
                            game_state.camera.target.y
                        )
                    );
                    ui.heading("Star System Objects");
                    for sso in game_state.ctx.db().star_system_object().iter() {
                        ui.horizontal(|ui| {
                            ui.label(format!(" - {:?}    ", sso.kind));
                            ui.label(
                                format!(
                                    "({}au, {}°)    ",
                                    sso.orbit_au,
                                    sso.rotation_or_width_km.to_degrees()
                                )
                            );
                            let vec =
                                glam::Vec2::from_angle(sso.rotation_or_width_km) *
                                sso.orbit_au *
                                multiplier;
                            ui.label(format!("({}, {})", vec.x, vec.y));
                            ui.label(
                                format!("Angle: {}", (game_state.camera.target - vec).to_angle())
                            );
                        });
                    }
                });
        });

        egui_macroquad::draw();
        next_frame().await;

        let speed = if is_key_down(KeyCode::LeftShift) { 50.0 } else { 10.0 };

        if is_key_down(KeyCode::Down) {
            game_state.camera.target.y += speed;
        }
        if is_key_down(KeyCode::Up) {
            game_state.camera.target.y -= speed;
        }
        if is_key_down(KeyCode::Right) {
            game_state.camera.target.x += speed;
        }
        if is_key_down(KeyCode::Left) {
            game_state.camera.target.x -= speed;
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }
    }

    //Ok(())
}
