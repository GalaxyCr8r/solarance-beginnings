use macroquad::{math::Vec2, prelude::*, ui};

use super::server::bindings::*;
use spacetimedb_sdk::{DbContext, Table};

use crate::{shader::*, stdb::utils::*};

mod gui;
mod player;
pub mod direct_server_messages;
pub mod render;
pub mod resources;
pub mod state;
pub mod visual_effects;

/// Register all the callbacks our app will use to respond to database events.
///
/// Post-#101 chat data is read directly from the STDB Views each frame, so the
/// old MPSC mirror is gone. The visual-effect callback stays — it's the only
/// side-effecting one.
pub fn register_callbacks(ctx: &DbConnection) {
    ctx.db().stellar_object().on_insert(|_ec, sobj| {
        info!("Stellar Object Inserted: {:?}", sobj);
    });

    // Register visual effect callback for client-side visual effects
    ctx.db().visual_effect().on_insert(|_ec, visual_effect| {
        info!(
            "Visual effect created: {:?} from ({}, {}) to ({}, {})",
            visual_effect.effect_type,
            visual_effect.source.x,
            visual_effect.source.y,
            visual_effect.target.x,
            visual_effect.target.y
        );
        // The visual effect will be handled by the visual_effects::update_visual_effects function
        // in the main game loop, which checks the database for new effects
    });
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Main Loop
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
pub async fn gameplay(connection: Option<DbConnection>) {
    //token : Option<String>) {
    // DB Connection & ECS World
    //let connection = connect_to_spacetime(token);
    if connection.is_none() {
        error!("Failed to connect to SpacetimeDB. Exiting...");
        return;
    }
    let ctx = connection.unwrap();

    let mut game_state = state::initialize(&ctx);
    game_state.camera.zoom.y *= -1.0;
    game_state.bg_camera.zoom.y *= -1.0;

    register_callbacks(&ctx);

    // Load starfield shader
    info!("Loading shader...");
    let sf_shader = load_starfield_shader();
    let render_target = render_target(320, 150);
    render_target.texture.set_filter(FilterMode::Linear);

    // Setup Panic Handler
    set_panic_handler(|msg, _backtrace| async move {
        loop {
            clear_background(RED);
            ui::root_ui().label(None, &msg);
            next_frame().await;
        }
    });

    loop {
        clear_background(WHITE);

        game_state.camera.target = get_player_transform_vec2(&ctx, Vec2::ZERO); // - Vec2 { x: screen_width()/4.0, y: screen_height()/4.0 };
        set_camera(&game_state.camera);

        let player_ship = get_player_ship(&ctx);

        if let Some(ship) = player_ship.clone() {
            if let Some(sector) = ctx.db().sector().id().find(&ship.sector_id) {
                game_state.bg_camera.target = game_state.camera.target;
                game_state.bg_camera.target *= 0.000_1337;
                game_state.bg_camera.target.x += sector.x * 100.0;
                game_state.bg_camera.target.y += sector.y * 100.0;
            }
        }

        apply_shader_to_screen(
            &render_target,
            &sf_shader,
            game_state.camera.target,
            game_state.camera.target * 0.000_01337,
        );

        // Update and render visual effects
        visual_effects::update_visual_effects(&mut game_state);

        render::sector(&mut game_state);

        // Render visual effects on top of everything else
        visual_effects::render_visual_effects(&game_state);

        egui_macroquad::ui(|egui_ctx| {
            if player_ship.is_none() {
                if ctx
                    .db()
                    .ship()
                    .iter()
                    .any(|ds| ds.player_id == ctx.identity())
                {
                    gui::out_of_play_screen::draw(egui_ctx, &ctx, &mut game_state);
                } else {
                    gui::creation_window::draw(egui_ctx, &ctx, &mut game_state);
                }
                return;
            }

            gui::debug_widget::draw(egui_ctx, &mut game_state);

            if player_ship.is_some() {
                // Widgets
                gui::minimap_widget::draw(egui_ctx, &mut game_state);
                gui::chat_widget::draw(egui_ctx, &game_state.ctx, &mut game_state.chat_window);
                gui::status_widget::window(egui_ctx, &ctx, &mut game_state);
                gui::menu_bar_widget::draw(egui_ctx, &ctx, &mut game_state);

                // Windows
                gui::assets_window::draw(
                    egui_ctx,
                    &game_state.ctx,
                    &mut game_state.assets_window,
                    &mut game_state.assets_window_open,
                );
                gui::ship_details_window::draw(
                    egui_ctx,
                    &game_state.ctx,
                    &mut game_state.details_window,
                    &mut game_state.details_window_open,
                );
                gui::faction_window::draw(
                    egui_ctx,
                    &game_state.ctx,
                    &mut game_state.faction_window,
                    &mut game_state.faction_window_open,
                );
                gui::map_window::draw(
                    egui_ctx,
                    &ctx,
                    &mut game_state.map_window,
                    &mut game_state.map_window_open,
                );
                gui::construction_window::draw(
                    egui_ctx,
                    &ctx,
                    &mut game_state.construction_window,
                    &mut game_state.construction_window_open,
                );
            }

            // Welcome-back panel (#100): shows once on connect regardless of
            // whether the player is piloting, docked, or still on the creation
            // screen. Self-suppresses after dismissal.
            // Renders last so it's always on top!
            gui::welcome_back_widget::draw(
                egui_ctx,
                &game_state.ctx,
                &mut game_state.welcome_back,
            );
        });

        egui_macroquad::draw();
        next_frame().await;

        let _ = player::control_player_ship(&ctx, &mut game_state); // TODO Alert player of error

        if !game_state.chat_window.has_focus && player_ship.is_some() {
            if is_key_pressed(KeyCode::E) {
                if let Ok(target) = player::target_closest_stellar_object(&ctx, &mut game_state) {
                    if game_state.current_target_sobj_id == Some(target.id) {
                        game_state.current_target_sobj_id = None;
                    } else {
                        game_state.current_target_sobj_id = Some(target.id);
                    }
                }
            }
            if is_key_pressed(KeyCode::R) {
                game_state.details_window_open = !game_state.details_window_open;
            }
            if is_key_pressed(KeyCode::F) {
                game_state.faction_window_open = !game_state.faction_window_open;
            }
            if is_key_pressed(KeyCode::T) {
                game_state.assets_window_open = !game_state.assets_window_open;
            }
            if is_key_pressed(KeyCode::M) {
                game_state.map_window_open = !game_state.map_window_open;
            }
            if is_key_pressed(KeyCode::B) {
                game_state.construction_window_open = !game_state.construction_window_open;
            }
        }

        if game_state.done {
            let _ = ctx.disconnect();
            break;
        }
    }
}
