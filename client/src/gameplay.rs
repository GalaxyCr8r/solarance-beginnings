use std:: sync::mpsc::{self, Sender} ;

use macroquad::{ math::Vec2, prelude::*, ui };

use super::module_bindings::*;
use spacetimedb_sdk::{ DbContext, Table };
use super::stdb::connector::connect_to_spacetime;

use crate::{shader::*, stdb::utils::*};

mod state;
mod gui;
mod player;
mod render;
pub mod resources;

/// Register all the callbacks our app will use to respond to database events.
pub fn register_callbacks(ctx: &DbConnection, global_chat_channel: Sender<GlobalChatMessage>) {
    ctx.db.stellar_object().on_insert( |_ec, sobj| {
        info!("Stellar Object Inserted: {:?}", sobj);
    });

    ctx.db.global_chat_message().on_insert(move |_ec, message| {
        info!("{}: {}", message.identity.to_abbreviated_hex().to_string(), message.message);
        let _ = global_chat_channel.send(message.clone());
    });
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Main Loop
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn gameplay(token : Option<String>) {
    // DB Connection & ECS World
    let ctx = connect_to_spacetime(token);

    let (global_chat_transmitter, global_chat_receiver) = mpsc::channel::<GlobalChatMessage>();

    let mut game_state = state::initialize(&ctx);

    let _receiver = register_callbacks(&ctx, global_chat_transmitter);

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
        set_camera(&game_state.camera);
        
        apply_shader_to_screen(
            render_target,
            sf_shader,
            game_state.camera.target,
            game_state.camera.target * 0.0001337
        );

        render::sector(&mut game_state);

        egui_macroquad::ui(|egui_ctx| {  
            gui::debug::window(&egui_ctx, &mut game_state);

            if get_player(&ctx.db, &ctx.identity()).is_some() {
                gui::chat::window(&egui_ctx, &game_state.ctx, &mut game_state.chat_window);
                gui::status::window(&egui_ctx, &ctx, &mut gui::status::WindowState::default());
                gui::ship_details::window(&egui_ctx, &game_state.ctx, &mut game_state.details_window, &mut game_state.details_window_open);
                gui::menu_bar::window(&egui_ctx, &ctx, &mut game_state);
            }
        });

        egui_macroquad::draw();
        next_frame().await;

        let _ = player::control_player_ship(&ctx, &mut game_state);

        if is_key_down(KeyCode::R) {
            game_state.details_window_open = !game_state.details_window_open;
        }

        if let Ok(message) = global_chat_receiver.try_recv() {
            game_state.chat_window.global_chat_channel.push(message);
            game_state.chat_window.global_chat_channel.sort_by_key(|chat| chat.id);
        }

        if game_state.done {
            let _ = ctx.disconnect();
            break;
        }
    }
}
