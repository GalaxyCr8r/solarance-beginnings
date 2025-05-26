use egui::{ Align2, Context, ScrollArea };
use macroquad::miniquad::date::now;
use macroquad::prelude::*;

use crate::module_bindings::*;
use spacetimedb_sdk::{ DbContext, Table };

use crate::stdb::utils::*;
use crate::gameplay::state::GameState;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
// Debug Gameplay Window
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

pub fn window(egui_ctx: &Context, game_state: &mut GameState) -> Option<egui::InnerResponse<Option<()>>> {
    let ctx = game_state.ctx;

    egui::Window
        ::new("Debug")
        .title_bar(true)
        .resizable(false)
        .collapsible(true)
        .movable(false)
        .anchor(Align2::LEFT_BOTTOM, egui::Vec2::new(-5.0, 5.0))
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

            ui.collapsing("Stellar Objects", |ui| {
                ScrollArea::vertical()
                .auto_shrink([false, true])
                .stick_to_bottom(true)
                .max_height(screen_height()/4.0)
                .show(ui, |ui| {
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
                                }
                                _ => {
                                    ui.label("Position: n/a");
                                }
                            }
                            ui.label(format!("- Sector #{}", object.sector_id));
                        });
                    }
                });
            });

            // for player_controlled in ctx.db.ship_object().iter() {
            //     ui.label(format!(" - Player Controlled Obj #{} in Sec#{}", player_controlled.sobj_id, player_controlled.sector_id));
            // }

            // ui.label(
            //     format!(
            //         "Camera: {}, {}",
            //         game_state.camera.target.x.to_string(),
            //         game_state.camera.target.y.to_string()
            //     )
            // );

            ui.label(format!("Now: {}", now()));

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("  Quit  ").clicked() {
                    game_state.done = true;
                }
                if ui.button("Ship Details").clicked() {
                    game_state.details_window_open = !game_state.details_window_open;
                }
            });
        })
}