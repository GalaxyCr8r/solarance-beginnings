use std::{ f32::consts::PI, thread::{ self, JoinHandle } };

use dotenv::dotenv;
use egui::{ Align2, Color32, RichText };
use gameplay::resources::Resources;
use macroquad::{math::Vec2, prelude::{collections::storage, coroutines::start_coroutine, *}, time};

mod module_bindings;
pub mod oidc_auth_helper;
pub mod stdb;
mod shader;
pub mod gameplay;

struct MenuAssets {
    pub rings: Vec<Texture2D>
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Main Loop
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

#[macroquad::main("Solarance:Beginnings")]
async fn main() -> Result<(), FileError> {
    dotenv().ok();

    set_pc_assets_folder("assets");
    storage::store(MenuAssets{
        rings: vec![
            load_texture("Ring1.png").await.expect("Couldn't load file"),
            load_texture("Ring2.png").await.expect("Couldn't load file"),
            load_texture("Ring3.png").await.expect("Couldn't load file")
        ]
    });

    loop {
        let id_token: Option<String> = login_screen().await;

        if id_token.is_none() {
            break;
        }
        
        loading_screen().await;
    
        info!("Calling gameplay from main");
        gameplay::gameplay(id_token).await;
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Login Screen
/// TODO: Refactor out of main.rs
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) async fn login_screen() -> Option<String> {
    info!("Entering login screen");

    let mut client_token_thread: Option<JoinHandle<Result<String, String>>> = None;

    let mut id_token: Option<String> = None;
    let mut error_message: Option<String> = None;

    let mut break_the_loop = false;

    let menu_assets = storage::get::<MenuAssets>();
    info!("Starting login screen");

    loop { 
        if client_token_thread.as_ref().is_some_and(|thread| { thread.is_finished() }) {
            let thread = client_token_thread.take().unwrap();
            if thread.is_finished() {
                match thread.join() {
                    Ok(token_result) =>
                        match token_result {
                            Ok(token) => {
                                id_token = Some(token.to_string());
                            }
                            Err(error) => {
                                error_message = Some(error.to_string());
                            }
                        }
                    Err(_join_error) => {
                        error_message = Some("Unknown join error of login thread!".to_string());
                    }
                }
            }
        }

        clear_background(DARKGRAY);
        draw_circle(screen_width() / 2.0, screen_height() / 2.0, screen_height() * 2.0 / 3.0, BLACK);

        for i in 0..3 {
            let (x, y) = mouse_position();
            let rot = match i {
                0 => { f32::cos((time::get_time() as f32) * 0.5 + (x / 2048.0)) }
                1 => { f32::sin(time::get_time() as f32) }
                _ => { f32::cos((-time::get_time() as f32) * 0.25 + (y / 2048.0)) }
            };
            draw_texture_ex(
                menu_assets.rings[i],
                screen_width() / 2.0 - menu_assets.rings[i].width() / 4.0,
                screen_height() / 2.0 - menu_assets.rings[i].height() / 4.0,
                Color {
                    a: 0.5,
                    ..Color::from_hex(0xbedaff)
                },
                DrawTextureParams {
                    rotation: rot * PI,
                    dest_size: Some(Vec2::new(menu_assets.rings[i].width() / 2.0, menu_assets.rings[i].height() / 2.0)),
                    ..Default::default()
                }
            );
        }

        egui_macroquad::ui(|egui_ctx| {
            egui::Window
                ::new("Solarance:Beginnings")
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .anchor(Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
                .show(egui_ctx, |ui| {
                    if !client_token_thread.as_ref().is_none() {
                        ui.label("Waiting on handshake...");
                    }
                    else {
                        if error_message.is_some() {
                            ui.label(
                                RichText::new(
                                    format!("ERROR: {}", error_message.as_ref().unwrap().to_string())
                                ).color(Color32::RED)
                            );
                        }
    
                        if !id_token.is_some() && ui.button(RichText::new("\n      Login      \n").size(32.0)).clicked() {
                            info!("CLICKED!");
                            client_token_thread = Some(
                                thread::spawn(|| { oidc_auth_helper::get_client_token() })
                            );
                        }
                        if id_token.is_some() && ui.button(RichText::new("\n      PLAY!      \n").size(32.0)).clicked() {
                            info!("CLICKED!");
                            break_the_loop = true;
                        }
                    }
                    if id_token.is_none() && ui.button("Quit").clicked() {
                        break_the_loop = true;
                    }
                });
        });

        egui_macroquad::draw();
        next_frame().await;

        if break_the_loop {
            break;
        }
    }

    id_token
}

//// Loading Screen
/// 

async fn loading_screen() {
    let menu_assets = storage::get::<MenuAssets>();
    
    let mut resources_loading: Option<coroutines::Coroutine> = None;
    
    while resources_loading.is_none() || !resources_loading.unwrap().is_done() { 
        clear_background(BLACK);

        for i in 0..3 {
            draw_texture_ex(
                menu_assets.rings[i],
                menu_assets.rings[i].width() / -3.0,
                screen_height() - menu_assets.rings[i].height() / 3.0,
                Color {
                    a: 0.5,
                    ..Color::from_hex(0xbedaff)
                },
                DrawTextureParams {
                    dest_size: Some(Vec2::new(menu_assets.rings[i].width() / 2.0, menu_assets.rings[i].height() / 2.0)),
                    ..Default::default()
                }
            );
        }

        let text = format!(
            "Connecting to the Solarance galaxy  {}",
            ". ".repeat(((get_time() * 2.) as usize) % 4)
        );
        draw_text(&text, 42.0, 42.0, 32.0, DARKGRAY);
        next_frame().await;

        if resources_loading.is_none() {
            resources_loading = Some(start_coroutine(async move {
                let resources = Resources::new().await.unwrap();
                storage::store(resources);
            }));
        }
    }
}

