use egui::{Align2, Color32, Context, RichText};
use spacetimedb_sdk::{DbContext, Table};

use crate::{gameplay::state::GameState, module_bindings::*, stdb::utils::*};

// #[derive(PartialEq)]
// enum CurrentTab {
//     Ship,
//     Cargo
// }

pub struct State {
    pub text: String,
    pub error: Option<String>,
    pub selected_faction_id: Option<u32>,
}

impl State {
    pub fn new() -> Self {
        State {
            text: "".to_string(),
            error: None,
            selected_faction_id: None,
        }
    }
}

pub fn draw(
    egui_ctx: &Context,
    ctx: &DbConnection,
    game_state: &mut GameState,
) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window::new("Account Creation")
        .title_bar(true)
        .resizable(false)
        .collapsible(false)
        .movable(true)
        .vscroll(false)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(egui_ctx, |ui| {
            ui.vertical_centered(|ui| {
                if let Some(player) = get_player(&ctx.db, &ctx.identity()) {
                    create_ship(ctx, game_state, ui, player);
                } else if let Some(_player_ship) = get_player_ship(&ctx) {
                    // You're ready to go!
                } else {
                    create_player(ctx, game_state, ui);
                }
                if game_state.creation_window.error.is_some() {
                    ui.label(
                        RichText::new(format!(
                            "ERROR: {}",
                            game_state
                                .creation_window
                                .error
                                .as_ref()
                                .unwrap()
                                .to_string()
                        ))
                        .strong()
                        .color(Color32::RED),
                    );
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    game_state.done = true;
                }
            });
        })
}

fn create_ship(
    ctx: &DbConnection,
    game_state: &mut GameState<'_>,
    ui: &mut egui::Ui,
    player: Player,
) {
    // Create a ship
    ui.heading(format!("Welcome Captain {}!", player.username));
    ui.separator();
    ui.heading("Basic Instructions");
    ui.label(
        "Currently you can only mine asteroids, buy/sell goods at stations, and travel to different sectors via jump gates. There is no combat or NPC ships yet."
    );
    ui.label("The next milestone will begin NPCs and make the universe feel more alive!");
    ui.strong("Use WASD or the Arrow keys to move. 'Down' or 'S' will slow your ship.");
    ui.label(
        "To dock with stations or to use jump gates, engage auto-docking, target the station/gate and get to its exact center. Jump gates drain half your energy currently."
    );
    ui.label(
        "To mine asteroids, target a asteroid with '[E]' or the minimap and use '[X]' mining on it. Mining takes energy that will slowly refill."
    );
    ui.separator();
    if ui
        .button(RichText::new("> Create a 'Column-class' Mining Corvette < ").strong())
        .clicked()
    {
        match ctx
            .reducers
            .create_player_controlled_ship(ctx.identity(), game_state.chat_window.text.clone())
        {
            Ok(_) => {
                game_state.creation_window.error = None;
            }
            Err(e) => {
                game_state.creation_window.error = Some(format!("{:?}", e));
            }
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
    });

    ui.separator();

    // Faction selection
    ui.strong("Select your faction:");
    ui.label("Choose wisely - this will determine your starting relationships and opportunities.");
    ui.small("Eventually your faction will determine your starting/replacement ship.");

    // Get all factions and filter to known joinable ones
    let joinable_factions: Vec<_> = ctx
        .db
        .faction()
        .iter()
        .filter(|faction| faction.joinable)
        .collect();

    if joinable_factions.is_empty() {
        ui.label("No factions available for selection.");
    } else {
        for faction in &joinable_factions {
            let is_selected = game_state.creation_window.selected_faction_id == Some(faction.id);

            if ui.selectable_label(is_selected, &faction.name).clicked() {
                game_state.creation_window.selected_faction_id = Some(faction.id);
            }

            if is_selected {
                ui.small(&faction.description);
            }
        }
    }

    ui.separator();

    // Button to actually create the player.
    let can_create = !game_state.creation_window.text.is_empty()
        && game_state.creation_window.selected_faction_id.is_some();

    ui.add_enabled_ui(can_create, |ui| {
        if ui.button("Create Player Account").clicked()
            || (can_create && ui.input(|i| i.key_pressed(egui::Key::Enter)))
        {
            if let Some(faction_id) = game_state.creation_window.selected_faction_id {
                // Create Player
                match ctx.reducers.register_playername(
                    ctx.identity(),
                    game_state.creation_window.text.clone(),
                    faction_id,
                ) {
                    Ok(_) => {
                        game_state.creation_window.error = None;
                    }
                    Err(e) => {
                        game_state.creation_window.error = Some(format!("{:?}", e));
                    }
                }
            }
        }
    });

    if !can_create {
        if game_state.creation_window.text.is_empty() {
            ui.small("Please enter a username.");
        } else if game_state.creation_window.selected_faction_id.is_none() {
            ui.small("Please select a faction.");
        }
    }
}
