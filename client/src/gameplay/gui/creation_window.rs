use egui::{Color32, Context, FontId, RichText};
use macroquad::prelude::*;
use spacetimedb_sdk::DbContext;

use crate::{gameplay::state::GameState, module_bindings::*, stdb::utils::*};

// #[derive(PartialEq)]
// enum CurrentTab {
//     Ship,
//     Cargo
// }

pub struct State {
    pub text: String,
    pub error: Option<String>
}

impl State {
    pub fn new() -> Self {
        State {
            text: "".to_string(),
            error: None
        }
    }
}

pub fn draw(egui_ctx: &Context, ctx: &DbConnection, game_state: &mut GameState) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window
        ::new("Account Creation")
        .title_bar(true)
        .resizable(false)
        .collapsible(false)
        .movable(true)
        .vscroll(false)
        .show(egui_ctx, |ui| {
            ui.vertical_centered(|ui| {
                if let Some(player) = get_player(&ctx.db, &ctx.identity()) {
                    create_ship(ctx, game_state, ui, player);
                } else if let Some(player_ship) = get_player_ship(&ctx) {
                    // You're ready to go!
                } else {
                    create_player(ctx, game_state, ui);
                }
                if game_state.creation_window.error.is_some() {
                    ui.label(
                        RichText::new(
                            format!("ERROR: {}", game_state.creation_window.error.as_ref().unwrap().to_string())
                        ).strong().color(Color32::RED)
                    );
                }
            });
        })
}

fn create_ship(ctx: &DbConnection, game_state: &mut GameState<'_>, ui: &mut egui::Ui, player: Player) {
    // Create a ship
    ui.heading(format!("Welcome Captain {}!", player.username));
    ui.separator();
    ui.label("In the future, the step before this will be selecting your faction which will influence what ship(s) you can pick.");
    ui.strong("For now, just click the button to enter the Solarance universe.");
    ui.separator();
    ui.heading("Basic Instructions");
    ui.label("As of 0.2.x, you can only mine asteroids and travel to different sectors via jump gates. There is no combat or NPC ships yet.");
    ui.label("The next milestone will bring space stations where you'll be able to sell ore at and perhaps other things!");
    ui.strong("Use WASD or the Arrow keys to move. 'Down' or 'S' will slow your ship.");
    ui.label("To use jump gates, engage auto-docking with its hotkey and get to its exact center. Wait for it to drain all your energy.");
    ui.label("To mine asteroids, target a asteroid and use mining on it. The hotkeys should be obvious via the GUI.");
    ui.separator();
    if ui.button(RichText::new("Create Your Ship").strong()).clicked() {
        match ctx.reducers.create_player_controlled_ship(ctx.identity(), game_state.chat_window.text.clone()) {
            Ok(_) => game_state.creation_window.error = None,
            Err(e) => game_state.creation_window.error = Some(format!("{:?}", e)),
        }
    }
}

fn create_player(ctx: &DbConnection, game_state: &mut GameState<'_>, ui: &mut egui::Ui) {
    // Create an account
    ui.heading("Player Creation");
    ui.separator();
    ui.small("This will be seen by every player.");
    ui.separator();
    ui.horizontal(|ui| {
        ui.strong("Username:");
    
        ui.text_edit_singleline(&mut game_state.creation_window.text);
        if ui.button("Create").clicked() || 
           ui.input(|i| i.key_pressed(egui::Key::Enter))
        {
            if !game_state.creation_window.text.is_empty() {
                // Create Player
                match ctx.reducers.register_playername(ctx.identity(), game_state.creation_window.text.clone()) {
                    Ok(_) => game_state.creation_window.error = None,
                    Err(e) => game_state.creation_window.error = Some(format!("{:?}", e)),
                }
            }
        }
    });
}