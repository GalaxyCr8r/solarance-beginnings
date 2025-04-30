use std::{ collections::HashMap, f32::consts::PI };

use egui::{Align2, RichText};
use macroquad::{ math::Vec2, prelude::*, time};
use secs::World;
use macroquad::miniquad::conf::Conf;

mod module_bindings;
use module_bindings::*;
use spacetimedb_sdk::{ Table };
//use stdb_client_helper::get_transform;
mod stdb_client_helper;
mod shader;
pub mod oidc_auth_helper;

struct GameState<'a> {
    paused: bool,
    done: bool,
    ctx: &'a DbConnection,
    textures: HashMap<&'static str, Texture2D>,
}

#[derive(Default)]
struct SolShip {
    sobj_id: u64,
}

#[derive(Default)]
struct Transform {
    x: f32,
    y: f32,
    rotation_radians: f32,
}

fn draw_ship(transform: &StellarObjectTransform) {
    let position = Vec2::new(transform.x, transform.y);
    let forward = Vec2::from_angle(transform.rotation_radians) * 16.0;
    let right = Vec2::from_angle(transform.rotation_radians + PI * 0.75) * 16.0;
    let left = Vec2::from_angle(transform.rotation_radians - PI * 0.75) * 16.0;

    let forward_pos = position + forward * 2.0;

    draw_line(position.x, position.y, forward_pos.x, forward_pos.y, 2.0, RED);
    draw_triangle(position + forward, position + right, position + left, WHITE);

    let string = format!("Sobj{}", transform.sobj_id.to_string());
    draw_text_ex(&string, transform.x, transform.y, TextParams {
        font_size: 16,
        rotation: transform.rotation_radians,
        color: WHITE,
        ..TextParams::default()
    });
}

fn render_system(_world: &World, game_state: &mut GameState) {
    if game_state.paused {
        let text = "PAUSED";
        let font_size = 100.0;
        let text_width = measure_text(text, None, font_size as u16, 1.0).width;
        let (x, y) = ((screen_width() - text_width) / 2.0, screen_height() / 2.0);

        draw_text(text, x, y, font_size, RED);

        return;
    }

    for object in game_state.ctx.db.stellar_object().iter() {
        match game_state.ctx.db.stellar_object_hi_res().sobj_id().find(&object.id) {
            Some(hirez) => {
                draw_ship(&hirez);
                let tex = game_state.textures["lc/phalanx"];
                draw_texture_ex(
                    tex,
                    hirez.x - tex.width() * 0.5,
                    hirez.y - tex.height() * 0.5,
                    WHITE,
                    DrawTextureParams {
                        rotation: hirez.rotation_radians,
                        ..DrawTextureParams::default()
                    }
                );
            }
            None => {
                match game_state.ctx.db.stellar_object_low_res().sobj_id().find(&object.id) {
                    Some(lorez) => draw_ship(&lorez),
                    None => (),
                }
            }
        }
    }

    draw_text("Solarance", 0.0, 64.0, 150.0, RED);
    draw_line(0.0, 0.0, 32.0, 150.0, 3.0, GOLD);
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Main Loop
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
fn _window_conf() -> Conf {
    Conf {
        window_title: "egui with macroquad & secs".to_owned(),
        //high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main("secs_macroquad")]
async fn main() {
    
    set_pc_assets_folder("assets");
    let rings: Vec<Texture2D> =
        vec![load_texture("Ring1.png").await.expect("Couldn't load file"),
             load_texture("Ring2.png").await.expect("Couldn't load file"),
             load_texture("Ring3.png").await.expect("Couldn't load file")];

    loop {
        clear_background(BLACK);

        for i in 0 .. 3 {
            let rot = match i {
                0 => {f32::cos((time::get_time() as f32) * 0.5)},
                1 => {f32::sin(time::get_time() as f32)},
                _ => {f32::cos((-time::get_time() as f32) * 0.25)}
            };
            draw_texture_ex(rings[i], screen_width()/2.0 - rings[i].width()/4.0, screen_height()/2.0  - rings[i].height()/4.0,
                Color {
                    a: 0.5,
                    ..Color::from_hex(0xBEDAFF)
                },
                DrawTextureParams {
                    rotation: rot * PI,
                    dest_size: Some(Vec2::new(rings[i].width()/2.0, rings[i].height()/2.0)),
                    ..Default::default()
                });
        }

        egui_macroquad::ui(|egui_ctx| {
            egui::Window
                ::new("Solarance:Beginnings")
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .anchor(Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
                .show(egui_ctx, |ui| {
                    if ui.button(RichText::new("\n      Login      \n").size(32.0)).clicked() {
                        info!("CLICKED!");
                        oidc_auth_helper::begin_connection();
                    }
                });
        });

        egui_macroquad::draw();
        next_frame().await;
    }
}
