use egui::{ Align2, Context };
use macroquad::prelude::*;

use crate::module_bindings::*;
use spacetimedb_sdk::{ DbContext, Table };

use crate::stdb::utils::*;

use super::state::GameState;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Debug Gameplay Window
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

pub fn debug_window(egui_ctx: &Context, game_state: &mut GameState) -> Option<egui::InnerResponse<Option<()>>> {
    let ctx = game_state.ctx;

    egui::Window
        ::new("Solarance:Beginnings")
        .resizable(false)
        .collapsible(false)
        .movable(false)
        .anchor(Align2::LEFT_TOP, egui::Vec2::new(-5.0, 5.0))
        .show(egui_ctx, |ui| {
            match ctx.db.player().identity().find(&ctx.identity()) {
                Some(player) => {
                    ui.heading(format!("Player: {}", player.username));
                    if let Some(controlled) = player.get_controlled_stellar_object(&ctx) {
                        match get_transform(&ctx, controlled)
                        {
                            Ok(transform) => {
                                ui.label(
                                    format!(
                                        "Ship: {}, {}",
                                        transform.x.to_string(),
                                        transform.y.to_string()
                                    )
                                );
                            }
                            _ => {
                                ui.label("Ship: unknown");
                            }
                        }
                    } else {
                        ui.label("Ship: None");
                    }
                }
                None => {
                    ui.heading("Player: MISSING");
                    ui.label(format!("ID: {}", ctx.identity()));

                    ui.horizontal(|ui| {
                        ui.label("Username: ");
                        ui.text_edit_singleline(&mut game_state.chat_window.text);
                    });

                    if ui.button("Create Player & Ship").clicked() && game_state.chat_window.text.len() > 1{
                        info!("Creating player and ship");
                        let _ = ctx.reducers.create_player_controlled_ship(ctx.identity(), game_state.chat_window.text.clone());
                        game_state.chat_window.text.clear();
                    }
                }
            }

            for object in ctx.db.stellar_object().iter() {
                ui.horizontal(|ui| {
                    ui.label(format!("- Ship #{}", object.id));

                    match get_transform(&ctx, object.id) {
                        Ok(transform) => {
                            let string = format!(
                                "Position: {}, {}",
                                transform.x.to_string(),
                                transform.y.to_string()
                            );
                            ui.label(string);
                            return;
                        }
                        _ => {
                            ui.label("Position: n/a");
                        }
                    }
                });
            }

            for player_controlled in ctx.db.player_controlled_stellar_object().iter() {
                ui.label(format!(" - Player Controlled Obj #{} in Sec#{}", player_controlled.sobj_id, player_controlled.sector_id));
            }

            ui.label(
                format!(
                    "Camera: {}, {}",
                    game_state.camera.target.x.to_string(),
                    game_state.camera.target.y.to_string()
                )
            );

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("  Quit  ").clicked() {
                    game_state.done = true;
                }
            });
        })
}