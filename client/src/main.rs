use std::{ f32::consts::PI, thread::{ self, JoinHandle } };

use dotenv::dotenv;
use egui::{ Align2, Button, Color32, Frame, RichText, Shadow };
use gameplay::resources::Resources;
use macroquad::{math::Vec2, prelude::{collections::storage, coroutines::start_coroutine, *}, time};

use crate::{module_bindings::DbConnection, stdb::connector::connect_to_spacetime};

mod module_bindings;
pub mod oidc_auth_helper;
pub mod stdb;
mod shader;
pub mod gameplay;

struct MenuAssets {
    pub rings: Vec<Texture2D>,
    pub logo: Texture2D
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Main Loop
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

#[macroquad::main("Solarance:Beginnings")]
async fn main() -> Result<(), macroquad::Error> {
    dotenv().ok();

    //request_new_screen_size(720.0, 480.0);
    request_new_screen_size(1280.0, 720.0);

    clear_background(BLACK);
    next_frame().await;

    set_pc_assets_folder("assets");
    storage::store(MenuAssets{
        rings: vec![
            load_texture("Ring1.png").await.expect("Couldn't load assets"),
            load_texture("Ring2.png").await.expect("Couldn't load assets"),
            load_texture("Ring3.png").await.expect("Couldn't load assets")
        ],
        logo: load_texture("Solarance_Logo.png").await.expect("Couldn't load assets")
    });

    if !confirm_eula_screen().await {
        return Ok(());
    }

    loop {
        let result = login_screen().await;
        if !result.0 {
            break;
        }
        
        let connection = loading_screen(result.1).await;
    
        info!("Calling gameplay from main");
        gameplay::gameplay(connection).await;
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Login Screen
/// TODO: Refactor out of main.rs
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

async fn confirm_eula_screen() -> bool {
    let mut result: Option<bool> = None;

    loop {
        draw_login_screen_background();

        egui_macroquad::ui(|egui_ctx| {
            egui::Window
                ::new("Welcome")
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .anchor(Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
                .frame(Frame::group(&egui_ctx.style()).fill(Color32::from_rgba_unmultiplied(15, 15, 15, 245)).shadow(Shadow::NONE))
                .show(egui_ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.small("Welcome to");
                        ui.heading(RichText::new(format!("Solarance:Beginnings v{}", env!("CARGO_PKG_VERSION"))).size(48.0));
                        ui.strong(env!("CARGO_PKG_DESCRIPTION"));
                        ui.separator();
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.code(format!("Project: {}", env!("CARGO_PKG_NAME")));
                                ui.code(format!("Version: {}", env!("CARGO_PKG_VERSION")));
                                ui.code(format!("Authors: {}", env!("CARGO_PKG_AUTHORS")));
                            });
                        });
                        ui.separator();
                        ui.label("Hello, I'm Karl Nyborg - and this is my exhuastive test of what SpacetimeDB can currently do.");
                        ui.label("Solarance is a 2D top down game idea I've had for years now. Heavily inspired by Escape Velocity: Nova, X2/X3, Freelancer, and numerous other entries in the space adventure/building genre. This is a test project to explore Rust, Macroquad, and SpacetimeDB to finally make the space MMO I've always wanted to make. You can help either by contributing code - or just by playing this test client.");
                        ui.label("As far as allowed by relevant laws, there is no warantee, run this software at your own risk. I do not collect any information aside from game-related information. I use Auth0 to outsource sign ups and logins.");
                        ui.label("Currently the only AI/LLM/DL usage has been for code completion, project ideation, and bug fixing. All art has (currently) been produced by me or public domain sources.");
                        ui.label("Thank you for joining me on this journey!");
                        ui.separator();
                        ui.separator();
                        ui.strong("By clicking 'Continue' you acknowledge that this is early alpha and does not represent a released game.");
                    });
                    ui.vertical_centered(|ui| {
                        if ui.button(RichText::new("    Continue    ").size(24.0)).clicked() {
                            result = Some(true)
                        }
                        if ui.button(RichText::new("      Exit      ").size(24.0)).clicked() {
                            result = Some(false)
                        }
                    });
                });
            });


        egui_macroquad::draw();
        next_frame().await;

        if result.is_some() {
            break;
        }
    }
    return result.is_some_and(|continue_| continue_);
}

pub async fn login_screen() -> (bool, Option<String>) {
    info!("Entering login screen");

    let mut client_token_thread: Option<JoinHandle<Result<String, String>>> = None;

    let mut id_token: Option<String> = None;
    let mut error_message: Option<String> = None;

    let mut break_the_loop = false;
    let mut quit_game = false;

    //let menu_assets = storage::get::<MenuAssets>();
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
                    Err(join_error) => {
                        error_message = Some(format!("Unexpected error during login! {:?}", join_error));
                    }
                }
            }
        }

        draw_login_screen_background();

        egui_macroquad::ui(|egui_ctx| {
            egui::Window
                ::new("Solarance:Beginnings")
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .anchor(Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, 0.0))
                .frame(Frame::group(&egui_ctx.style()).fill(Color32::from_rgba_unmultiplied(15, 15, 15, 245)).shadow(Shadow::NONE))
                .show(egui_ctx, |ui| { ui.vertical_centered(|ui| {
                    if !client_token_thread.as_ref().is_none() {
                        ui.label("Waiting on handshake, check your browser.");
                    }
                    if error_message.is_some() {
                        ui.label(
                            RichText::new(
                                format!("ERROR: {}", error_message.as_ref().unwrap().to_string())
                            ).color(Color32::RED)
                        );
                    }
                    ui.horizontal(|ui| {
                        if client_token_thread.as_ref().is_none() {
                            if !id_token.is_some() && ui.button(RichText::new("\n    Login via Auth0    \n").size(24.0)).clicked() {
                                info!("CLICKED!");
                                client_token_thread = Some(
                                    thread::spawn(|| { oidc_auth_helper::get_client_token() })
                                );
                            } else if id_token.is_none() && client_token_thread.is_some() {
                                ui.add_enabled(false, Button::new("\n    Login via Auth0    \n"));
                            }
                            if id_token.is_some() && ui.button(RichText::new("\n    Play via Auth0    \n").size(24.0)).clicked() {
                                info!("CLICKED!");
                                break_the_loop = true;
                            }
                        }
                        if ui.button(RichText::new("\n    Play as Guest    \n").size(24.0)).clicked() {
                            info!("CLICKED!");
                            break_the_loop = true;
                        }
                    });
                    if ui.button(RichText::new("\n\t\tExit\t\t\n").size(24.0)).clicked() {
                        quit_game = true;
                    }
                })
                });
        });

        egui_macroquad::draw();
        next_frame().await;

        if quit_game {
            return (false, None);
        }
        if break_the_loop {
            break;
        }
    }

    (true, id_token)
}

fn draw_login_screen_background() {
    let menu_assets = storage::get::<MenuAssets>();

    clear_background(BLACK);
    draw_circle(screen_width() / 2.0, screen_height() / 2.0, screen_height() * 2.0 / 3.0, Color::from_rgba(0xBE, 0xDA, 0xFF, 0x11));
    draw_circle(screen_width() / 2.0, screen_height() / 2.0, screen_height() * 2.0 / 4.0, Color::from_rgba(0xBE, 0xDA, 0xFF, 0x11));

    for i in 0..3 {
        let (x, y) = mouse_position();
        let rot = match i {
            0 => { f32::cos((time::get_time() as f32) * 0.5 + (x / 2048.0)) }
            1 => { f32::sin(time::get_time() as f32) }
            _ => { f32::cos((-time::get_time() as f32) * 0.25 + (y / 2048.0)) }
        };
        draw_texture_ex(
            &menu_assets.rings[i],
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

    draw_texture(&menu_assets.logo,
        screen_width() / 2.0 - menu_assets.logo.width() / 2.0,
        screen_height() / 2.0 - menu_assets.logo.height() / 2.0,
        WHITE);
}

// Loading Screen
////////// 

async fn loading_screen(token : Option<String>) -> Option<DbConnection> {
    let menu_assets = storage::get::<MenuAssets>();
    
    let connection = connect_to_spacetime(token);
    let mut resources_loading: Option<coroutines::Coroutine> = None;
    
    while resources_loading.is_none() || !resources_loading.unwrap().is_done() { 
        clear_background(Color::from_hex(0x000311));

        for i in 0..3 {
            draw_texture_ex(
                &menu_assets.rings[i],
                menu_assets.rings[i].width() / -3.0,
                screen_height() - menu_assets.rings[i].height() / 3.0,
                Color {
                    a: 0.05 + (i as f32 * 0.1),
                    ..Color::from_hex(0xbedaff)
                },
                DrawTextureParams {
                    dest_size: Some(Vec2::new(menu_assets.rings[i].width() / 2.0, menu_assets.rings[i].height() / 2.0)),
                    ..Default::default()
                }
            );
        }
        draw_texture(&menu_assets.logo,
            screen_width() / 2.0 - menu_assets.logo.width() / 2.0,
            screen_height() / 2.0 - menu_assets.logo.height() / 2.0,
            Color {
                a: 0.25,
                ..WHITE
            });

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
    return connection;
}

