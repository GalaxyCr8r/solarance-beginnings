use std::{env, path::PathBuf};

use solarance_beginnings::*;

use dotenv::dotenv;
use macroquad::prelude::{collections::storage, *};

mod login;

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

    storage::store(login::MenuAssets {
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

    if !login::confirm_eula_screen().await {
        return Ok(());
    }

    loop {
        let result = login::login_screen().await;
        if !result.0 {
            break;
        }

        let connection = login::loading_screen(result.1).await;

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
