use std::{ env, f32::consts::PI, path::PathBuf, thread::{ self, JoinHandle } };

use dotenv::dotenv;
use egui::{ Align2, Button, Color32, Frame, RichText, Shadow };
use macroquad::{
    math::Vec2,
    prelude::{ collections::storage, coroutines::start_coroutine, * },
    time,
};

use solarance_beginnings::gameplay::resources::*;

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
async fn main() -> Result<(), macroquad::Error> {
    set_pc_assets_folder("assets");
    request_new_screen_size(1280.0, 720.0);

    let resources = Resources::new().await.unwrap();
    storage::store(resources);

    loop {
        clear_background(BLACK);

        egui_macroquad::ui(|egui_ctx| {
            egui::Window
                ::new("Star System Test")
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .anchor(Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, 0.0))
                .frame(
                    Frame::group(&egui_ctx.style())
                        .fill(Color32::from_rgba_unmultiplied(15, 15, 15, 245))
                        .shadow(Shadow::NONE)
                )
                .show(egui_ctx, |ui| {
                    ui.heading("Henlo!");
                });
        });

        egui_macroquad::draw();
        next_frame().await;

        // if quit_game {
        //     return (false, None);
        // }
        // if break_the_loop {
        //     break;
        // }
    }

    Ok(())
}
