use std::{
    env,
    f32::consts::PI,
    path::PathBuf,
    thread::{self, sleep, JoinHandle},
    time::Duration,
};

use solarance_beginnings::{stdb::connector::creds_store, *};

use dotenv::dotenv;
use egui::{Align2, Color32, Frame, RichText, Shadow};
use gameplay::resources::Resources;
use macroquad::{
    math::Vec2,
    prelude::{collections::storage, coroutines::start_coroutine, *},
    time,
};

use solarance_beginnings::{module_bindings::DbConnection, stdb::connector::connect_to_spacetime};
use spacetimedb_sdk::DbContext;

struct MenuAssets {
    pub rings: Vec<Texture2D>,
    pub logo: Texture2D,
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
// Main Loop
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

/// Configures the game window properties including title, dimensions, and resizability
fn window_conf() -> Conf {
    #[cfg(not(target_os = "macos"))]
    {
        dotenv().ok();
    }
    #[cfg(target_os = "macos")]
    {
        let exe_directory = get_exe_path();
        if get_exe_path().join("../Resources/.env").exists() {
            let env_path = get_exe_path().join("../Resources/.env");
            dotenv::from_path(env_path.clone()).ok();
            info!("Env Path: {:?}", env_path.clone().to_str().unwrap());
        } else {
            info!(
                "Did not find Resources folder. Falling back to working directory's assets folder."
            );
            dotenv().ok();
        }
    }

    // Parse window dimensions from environment variables with fallback defaults
    let window_width = env::var("WINDOW_WIDTH")
        .unwrap_or_else(|_| "1600".to_string())
        .parse::<i32>()
        .unwrap_or(1600);

    let window_height = env::var("WINDOW_HEIGHT")
        .unwrap_or_else(|_| "900".to_string())
        .parse::<i32>()
        .unwrap_or(900);

    let fullscreen = env::var("FULLSCREEN")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    Conf {
        window_title: "Solarance:Beginnings".to_owned(),
        window_width,
        window_height,
        window_resizable: false,
        fullscreen,
        ..Default::default()
    }
}

/// Main entry point for the Solarance:Beginnings game
///
/// Handles environment setup, asset loading, and game flow control:
/// 1. Sets up environment variables and asset paths based on OS
/// 2. Loads menu assets
/// 3. Shows EULA screen
/// 4. Manages login flow
/// 5. Connects to SpacetimeDB
/// 6. Launches gameplay
#[macroquad::main(window_conf)]
async fn main() -> Result<(), macroquad::Error> {
    #[cfg(not(target_os = "macos"))]
    {
        dotenv().ok();
        set_pc_assets_folder("assets");
    }
    #[cfg(target_os = "macos")]
    {
        let exe_directory = get_exe_path();
        if get_exe_path().join("../Resources/.env").exists() {
            let env_path = get_exe_path().join("../Resources/.env");
            dotenv::from_path(env_path.clone()).ok();

            info!(
                "Current Directory: {:?}",
                env::current_dir().unwrap().to_str().unwrap()
            );
            info!("Env Path: {:?}", env_path.clone().to_str().unwrap());
            info!("Binary Path: {:?}", exe_directory.to_str().unwrap());

            set_pc_assets_folder(
                format!("{}/../Resources/Assets", exe_directory.to_str().unwrap()).as_str(),
            );
        } else {
            info!(
                "Did not find Resources folder. Falling back to working directory's assets folder."
            );
            dotenv().ok();
            set_pc_assets_folder("assets");
        }
    }

    clear_background(BLACK);
    next_frame().await;

    storage::store(MenuAssets {
        rings: vec![
            load_texture("Ring1.png")
                .await
                .expect("Couldn't load assets"),
            load_texture("Ring2.png")
                .await
                .expect("Couldn't load assets"),
            load_texture("Ring3.png")
                .await
                .expect("Couldn't load assets"),
        ],
        logo: load_texture("Solarance_Logo.png")
            .await
            .expect("Couldn't load assets"),
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

/// Returns the directory path of the current executable
///
/// Used for locating resources and configuration files relative to the executable location,
/// particularly important for macOS bundle structure
fn get_exe_path() -> PathBuf {
    match env::current_exe() {
        Ok(mut p) => {
            p.pop();
            p
        }
        Err(_) => PathBuf::new(),
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
// Login Screen
// TODO: Refactor out of main.rs
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

/// Displays the EULA/welcome screen and waits for user confirmation
///
/// Shows game information, project details, and legal disclaimers
/// Returns true if user accepts and wishes to continue, false otherwise
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
                .frame(
                    Frame::group(&egui_ctx.style())
                        .fill(Color32::from_rgba_unmultiplied(15, 15, 15, 245))
                        .shadow(Shadow::NONE)
                )
                .show(egui_ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.small("Welcome to");
                        ui.heading(
                            RichText::new(
                                format!("Solarance:Beginnings v{}", env!("CARGO_PKG_VERSION"))
                            ).size(48.0)
                        );
                        ui.strong(env!("CARGO_PKG_DESCRIPTION"));
                        ui.separator();
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.code(format!("Project: {}", env!("CARGO_PKG_NAME")));
                                ui.code(format!("Version: {}", env!("CARGO_PKG_VERSION")));
                                ui.code(format!("Authors: {}", env!("CARGO_PKG_AUTHORS")));
                            });
                            ui.code(format!("Database Host: {:?}", std::env::var("DATABASE_HOST")));
                        });
                        ui.separator();
                        ui.label(
                            "Hello, I'm Karl Nyborg - and this is my exhuastive test of what SpacetimeDB can currently do."
                        );
                        ui.label(
                            "Solarance is a 2D top down game idea I've had for years now. Heavily inspired by Escape Velocity: Nova, X2/X3, Freelancer, and numerous other entries in the space adventure/building genre. This is a test project to explore Rust, Macroquad, and SpacetimeDB to finally make the space MMO I've always wanted to make. You can help either by contributing code - or just by playing this test client."
                        );
                        ui.label(
                            "As far as allowed by relevant laws, there is no warantee, run this software at your own risk. I do not collect any information aside from game-related information. I use Auth0 to outsource sign ups and logins."
                        );
                        ui.label(
                            "Currently the only AI/LLM/DL usage has been for code completion, project ideation, and bug fixing. All art has (currently) been produced by me or public domain sources."
                        );
                        ui.label("Thank you for joining me on this journey!");
                        ui.separator();
                        ui.separator();
                        ui.strong(
                            "By clicking 'Continue' you acknowledge that this is early alpha and does not represent a released game."
                        );
                    });
                    ui.vertical_centered(|ui| {
                        if ui.button(RichText::new("    Continue    ").size(24.0)).clicked() {
                            result = Some(true);
                        }
                        if ui.button(RichText::new("      Exit      ").size(24.0)).clicked() {
                            result = Some(false);
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

/// Enum representing the possible exit states of the login screen
enum EndLoginScreenSelection {
    No,            // Continue showing login screen
    BreakLoop,     // Exit login screen with current token
    QuitGame,      // Exit the game entirely
    UsePriorToken, // Use previously stored authentication token
}

/// Displays the login screen and handles authentication flow
///
/// Manages authentication via Auth0, guest login, or using a previously stored token
/// Returns a tuple containing:
/// - Boolean indicating whether to continue to the game (true) or exit (false)
/// - Optional authentication token for SpacetimeDB connection
pub async fn login_screen() -> (bool, Option<String>) {
    info!("Entering login screen");

    let mut client_token_thread: Option<JoinHandle<Result<String, String>>> = None;

    let mut id_token: Option<String> = None;
    let mut error_message: Option<String> = None;

    // let mut break_the_loop = false;
    // let mut quit_game = false;
    let mut end_loop = EndLoginScreenSelection::No;

    let prior_token = {
        let tmp = creds_store().load();
        if tmp.is_ok() {
            tmp.unwrap()
        } else {
            None
        }
    };
    let has_prior_token = prior_token.is_some() && !prior_token.clone().unwrap().is_empty();

    //let menu_assets = storage::get::<MenuAssets>();
    info!("Starting login screen");

    use EndLoginScreenSelection::*;
    loop {
        if client_token_thread
            .as_ref()
            .is_some_and(|thread| thread.is_finished())
        {
            let thread = client_token_thread.take().unwrap();
            if thread.is_finished() {
                match thread.join() {
                    Ok(token_result) => match token_result {
                        Ok(token) => {
                            id_token = Some(token.to_string());
                        }
                        Err(error) => {
                            error_message = Some(error.to_string());
                        }
                    },
                    Err(join_error) => {
                        error_message =
                            Some(format!("Unexpected error during login! {:?}", join_error));
                    }
                }
            }
        }

        draw_login_screen_background();

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Solarance:Beginnings")
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .anchor(Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, 0.0))
                .frame(
                    Frame::group(&egui_ctx.style())
                        .fill(Color32::from_rgba_unmultiplied(15, 15, 15, 245))
                        .shadow(Shadow::NONE),
                )
                .show(egui_ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        if !client_token_thread.as_ref().is_none() {
                            ui.label("Waiting on handshake, check your browser.");
                        }
                        if error_message.is_some() {
                            ui.label(
                                RichText::new(format!(
                                    "ERROR: {}",
                                    error_message.as_ref().unwrap().to_string()
                                ))
                                .color(Color32::RED),
                            );
                        }
                        ui.horizontal(|ui| {
                            if client_token_thread.as_ref().is_none() {
                                if id_token.is_none() {
                                    if ui
                                        .button(
                                            RichText::new("\n    Login via Auth0    \n").size(24.0),
                                        )
                                        .clicked()
                                    {
                                        info!("CLICKED!");
                                        client_token_thread = Some(thread::spawn(|| {
                                            oidc_auth_helper::get_client_token()
                                        }));
                                    }
                                }
                                if id_token.is_some()
                                    && ui
                                        .button(
                                            RichText::new("\n    Play via Auth0    \n").size(24.0),
                                        )
                                        .clicked()
                                {
                                    info!("CLICKED!");
                                    end_loop = BreakLoop;
                                }
                            } else {
                                if ui
                                    .button(RichText::new("\n    Login via Auth0    \n").size(24.0))
                                    .clicked()
                                {
                                    info!("CLICKED again");
                                    // Ditch the value and try again.
                                    let _ = client_token_thread.take().unwrap().join();
                                    client_token_thread = Some(thread::spawn(|| {
                                        oidc_auth_helper::get_client_token()
                                    }));
                                }
                            }
                            if ui
                                .button(RichText::new("\n    Play as Guest    \n").size(24.0))
                                .clicked()
                            {
                                info!("CLICKED!");
                                end_loop = BreakLoop;
                            }
                            if has_prior_token
                                && ui
                                    .button(RichText::new("\n    Continue    \n").size(24.0))
                                    .clicked()
                            {
                                info!("CLICKED!");
                                end_loop = UsePriorToken;
                            }
                        });
                        if ui
                            .button(RichText::new("\t\tExit\t\t").size(24.0))
                            .clicked()
                        {
                            end_loop = QuitGame;
                        }
                    })
                });
        });

        egui_macroquad::draw();
        next_frame().await;

        match end_loop {
            No => {
                // Intentionally left blank
            }
            BreakLoop => break,
            UsePriorToken => break,
            QuitGame => return (false, None),
        }
    }

    match end_loop {
        UsePriorToken => {
            let _ = creds_store().save("");
            (true, prior_token)
        }
        _ => (true, id_token),
    }
}

/// Renders the animated background for the login and EULA screens
///
/// Creates a space-themed background with:
/// - Circular blue glows
/// - Rotating ring textures that respond to mouse position
/// - Game logo centered on screen
fn draw_login_screen_background() {
    let menu_assets = storage::get::<MenuAssets>();

    clear_background(BLACK);
    draw_circle(
        screen_width() / 2.0,
        screen_height() / 2.0,
        (screen_height() * 2.0) / 3.0,
        Color::from_rgba(0xbe, 0xda, 0xff, 0x11),
    );
    draw_circle(
        screen_width() / 2.0,
        screen_height() / 2.0,
        (screen_height() * 2.0) / 4.0,
        Color::from_rgba(0xbe, 0xda, 0xff, 0x11),
    );

    for i in 0..3 {
        let (x, y) = mouse_position();
        let rot = match i {
            0 => f32::cos((time::get_time() as f32) * 0.5 + x / 2048.0),
            1 => f32::sin(time::get_time() as f32),
            _ => f32::cos((-time::get_time() as f32) * 0.25 + y / 2048.0),
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
                dest_size: Some(Vec2::new(
                    menu_assets.rings[i].width() / 2.0,
                    menu_assets.rings[i].height() / 2.0,
                )),
                ..Default::default()
            },
        );
    }

    draw_texture(
        &menu_assets.logo,
        screen_width() / 2.0 - menu_assets.logo.width() / 2.0,
        screen_height() / 2.0 - menu_assets.logo.height() / 2.0,
        WHITE,
    );
}

// Loading Screen
//////////

/// Displays a loading screen while connecting to SpacetimeDB and loading game resources
///
/// Handles:
/// 1. Establishing connection to SpacetimeDB using the provided authentication token
/// 2. Waiting for identity verification
/// 3. Loading game resources asynchronously
/// 4. Displaying visual feedback during the loading process
///
/// Returns the database connection if successful, None if connection fails
async fn loading_screen(token: Option<String>) -> Option<DbConnection> {
    let menu_assets = storage::get::<MenuAssets>();

    let mut connection = None;
    let mut resources_loading: Option<coroutines::Coroutine> = None;

    while resources_loading.is_none() || !resources_loading.unwrap().is_done() {
        clear_background(Color::from_hex(0x000311));

        for i in 0..3 {
            draw_texture_ex(
                &menu_assets.rings[i],
                menu_assets.rings[i].width() / -3.0,
                screen_height() - menu_assets.rings[i].height() / 3.0,
                Color {
                    a: 0.05 + (i as f32) * 0.1,
                    ..Color::from_hex(0xbedaff)
                },
                DrawTextureParams {
                    dest_size: Some(Vec2::new(
                        menu_assets.rings[i].width() / 2.0,
                        menu_assets.rings[i].height() / 2.0,
                    )),
                    ..Default::default()
                },
            );
        }
        draw_texture(
            &menu_assets.logo,
            screen_width() / 2.0 - menu_assets.logo.width() / 2.0,
            screen_height() / 2.0 - menu_assets.logo.height() / 2.0,
            Color { a: 0.25, ..WHITE },
        );

        let text = format!(
            "Connecting to the Solarance galaxy  {}",
            ". ".repeat(((get_time() * 2.0) as usize) % 4)
        );
        draw_text(&text, 42.0, 42.0, 32.0, DARKGRAY);
        next_frame().await;

        // Do loading and connecting logic - connect first though.
        if connection.is_none() {
            connection = connect_to_spacetime(token.clone());
            // Check if it really IS None, and bail accordingly.
            if connection.is_none() {
                return None;
            }
            let cnx_time = get_time() as u64;

            while connection
                .as_ref()
                .is_some_and(|cnx| cnx.try_identity().is_none())
            {
                if cnx_time + 10 < (get_time() as u64) {
                    return None;
                }
                sleep(Duration::from_secs(1));
            }

            // TODO - if the version of this binary and the version in GlobalConfig table is different, ask the user if they want to continue.
        } else if resources_loading.is_none() {
            // Only after the connection is alive do we actually load the resources.
            resources_loading = Some(start_coroutine(async move {
                let resources = Resources::new().await.unwrap();
                storage::store(resources);
            }));
        }
    }
    return connection;
}
