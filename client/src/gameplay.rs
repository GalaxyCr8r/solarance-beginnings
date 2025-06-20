use std:: sync::mpsc::{self, Sender} ;

use macroquad::{ math::Vec2, prelude::*, ui };

use super::module_bindings::*;
use spacetimedb_sdk::{ DbContext, Table };

use crate::{shader::*, stdb::utils::*};

mod state;
mod gui;
mod player;
mod render;
pub mod resources;

/// Register all the callbacks our app will use to respond to database events.
pub fn register_callbacks(ctx: &DbConnection, global_chat_channel: Sender<GlobalChatMessage>, sector_chat_channel: Sender<SectorChatMessage>) {
    ctx.db.stellar_object().on_insert( |_ec, sobj| {
        info!("Stellar Object Inserted: {:?}", sobj);
    });

    ctx.db.global_chat_message().on_insert(move |_ec, message| {
        info!("G{}: {}", message.player_id.to_abbreviated_hex().to_string(), message.message);
        let _ = global_chat_channel.send(message.clone());
    });

    ctx.db.sector_chat_message().on_insert(move |_ec, message| {
        info!("S{}: {}", message.player_id.to_abbreviated_hex().to_string(), message.message);
        let _ = sector_chat_channel.send(message.clone());
    });
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Main Loop
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn gameplay(connection: Option<DbConnection>) {//token : Option<String>) {
    // DB Connection & ECS World
    //let connection = connect_to_spacetime(token);
    if connection.is_none() {
        error!("Failed to connect to SpacetimeDB. Exiting...");
        return;
    }
    let ctx = connection.unwrap();

    let (global_chat_transmitter, global_chat_receiver) = mpsc::channel::<GlobalChatMessage>();
    let (sector_chat_transmitter, sector_chat_receiver) = mpsc::channel::<SectorChatMessage>();

    let mut game_state = state::initialize(&ctx);
    game_state.camera.zoom.y *= -1.0;

    let _receiver = register_callbacks(&ctx, global_chat_transmitter, sector_chat_transmitter);

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

    set_camera(&game_state.camera);

    loop {
        clear_background(WHITE);

        game_state.camera.target = get_player_transform_vec2(&ctx, Vec2::ZERO);// - Vec2 { x: screen_width()/4.0, y: screen_height()/4.0 };
        let player_ship = get_player_ship(&ctx);
        set_camera(&game_state.camera);
        
        apply_shader_to_screen(
            &render_target,
            &sf_shader,
            game_state.camera.target,
            game_state.camera.target * 0.0001337
        );

        render::sector(&mut game_state);

        egui_macroquad::ui(|egui_ctx| {
            if player_ship.is_none() {
                gui::creation_window::draw(egui_ctx, &ctx, &mut game_state);
                return;
            }

            gui::debug_widget::draw(egui_ctx, &mut game_state);

            if get_player(&ctx.db, &ctx.identity()).is_some() {
                // Widgets
                gui::minimap_widget::draw(egui_ctx, &mut game_state);
                gui::chat_widget::draw(egui_ctx, &game_state.ctx, &mut game_state.chat_window);
                gui::status_widget::window(egui_ctx, &ctx, &mut game_state);
                gui::menu_bar_widget::draw(egui_ctx, &ctx, &mut game_state);
                
                // Windows
                gui::ship_details_window::draw(egui_ctx, &game_state.ctx, &mut game_state.details_window, &mut game_state.details_window_open);
                gui::map_window::draw(egui_ctx, &ctx, &mut game_state.map_window, &mut game_state.map_window_open);
            }
        });

        egui_macroquad::draw();
        next_frame().await;

        let _ = player::control_player_ship(&ctx, &mut game_state); // TODO Alert player of error

        if !game_state.chat_window.has_focus {
            if is_key_pressed(KeyCode::E) {
                if let Ok(target) = player::target_closest_stellar_object(&ctx, &mut game_state) {
                    if let Some(mut controller) = ctx.db.player_ship_controller().player_id().find(&ctx.identity()) {
                        // Deselect target if it's already selected
                        if controller.targetted_sobj_id.is_some() && controller.targetted_sobj_id.unwrap() == target.id {
                            controller.targetted_sobj_id = None;
                            game_state.current_target_sobj = None;
                        } else {
                            controller.targetted_sobj_id = Some(target.id);
                            game_state.current_target_sobj = Some(target);
                        }
                        let _ = ctx.reducers.update_player_controller(controller); // TODO Alert player of error
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
        }

        // Handle callbacks
        if let Ok(message) = global_chat_receiver.try_recv() {
            game_state.chat_window.global_chat_channel.push(message);
            game_state.chat_window.global_chat_channel.sort_by_key(|chat| chat.created_at);
        }
        if player_ship.is_some() {
            if let Ok(message) = sector_chat_receiver.try_recv() {
                let sector_id = player_ship.unwrap().sector_id;
                if game_state.chat_window.sector_chat_channel.iter().any(|msg| msg.sector_id != sector_id) {
                    // Just dump prior sector messages.
                    game_state.chat_window.sector_chat_channel.retain(|msg| msg.sector_id == sector_id);
                }
                game_state.chat_window.sector_chat_channel.push(message);
                game_state.chat_window.sector_chat_channel.sort_by_key(|chat| chat.created_at);
            }
        }

        if game_state.done {
            let _ = ctx.disconnect();
            break;
        }
    }
}
