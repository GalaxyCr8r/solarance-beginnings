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

pub fn draw(
    egui_ctx: &Context,
    game_state: &mut GameState
) -> Option<egui::InnerResponse<Option<()>>> {
    let ctx = game_state.ctx;

    egui::Window
        ::new("Debug")
        .title_bar(true)
        .resizable(false)
        .collapsible(true)
        .movable(false)
        .anchor(Align2::LEFT_BOTTOM, egui::Vec2::new(-5.0, 5.0))
        .show(egui_ctx, |ui| {
            match ctx.db().player().id().find(&ctx.identity()) {
                Some(player) => {
                    ui.heading(format!("Player: {}", player.username));
                    if let Some(controlled) = player.get_controlled_stellar_object_id(&ctx) {
                        match get_transform(&ctx, controlled) {
                            Ok(transform) => {
                                ui.label(
                                    format!(
                                        "SObj: {}, {}",
                                        transform.x.to_string(),
                                        transform.y.to_string()
                                    )
                                );
                            }
                            _ => {
                                ui.label("SObj: unknown");
                            }
                        }
                    } else {
                        ui.label("WARNING - The player doesn't have a SObj!");

                        if ui.button("Create Ship").clicked() {
                            info!("Creating ship");
                            let _ = ctx.reducers.create_player_controlled_ship(
                                ctx.identity(),
                                game_state.chat_window.text.clone()
                            );
                            game_state.chat_window.text.clear();
                        }
                    }
                }
                None => {
                    ui.heading("Player: MISSING");
                    ui.label(format!("ID: {}", ctx.identity()));

                    ui.horizontal(|ui| {
                        ui.label("Username: ");
                        ui.text_edit_singleline(&mut game_state.chat_window.text);
                    });

                    if
                        ui.button("Create Player").clicked() &&
                        game_state.chat_window.text.len() > 1
                    {
                        info!("Creating player");
                        let _ = ctx.reducers.register_playername(
                            ctx.identity(),
                            game_state.chat_window.text.clone()
                        );
                        //game_state.chat_window.text.clear(); // Replace later maybe?
                    }
                }
            }

            ui.collapsing("Stellar Objects", |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .stick_to_bottom(true)
                    .max_height(screen_height() / 4.0)
                    .show(ui, |ui| {
                        let player_transform = get_player_transform_vec2(ctx, glam::Vec2::ZERO);
                        for object in ctx.db().stellar_object().iter() {
                            let obj_type = format!("{:?}", object.kind);

                            ui.horizontal(|ui| {
                                ui.label(format!("{} #{}", obj_type, object.id));

                                match get_transform(&ctx, object.id) {
                                    Ok(transform) => {
                                        let string = format!(
                                            "Position: {}, {} Distance: {}",
                                            transform.x.to_string(),
                                            transform.y.to_string(),
                                            player_transform.distance(transform.to_vec2())
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

            ui.collapsing("Players", |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .stick_to_bottom(true)
                    .max_height(screen_height() / 4.0)
                    .show(ui, |ui| {
                        for player in ctx.db().player().iter() {
                            ui.horizontal(|ui| {
                                ui.label(
                                    format!("[{}] Credits: {}", player.username, player.credits)
                                );
                                if
                                    let Some(window) = ctx
                                        .db()
                                        .sobj_player_window()
                                        .id()
                                        .find(&player.id)
                                {
                                    ui.label(
                                        format!(
                                            "- #{}: {}, {}, {}, {}",
                                            window.window,
                                            window.tl_x,
                                            window.tl_y,
                                            window.br_x,
                                            window.br_y
                                        )
                                    );
                                }
                                if
                                    let Some(_controller) = ctx
                                        .db()
                                        .player_ship_controller()
                                        .id()
                                        .find(&player.id)
                                {
                                    ui.label("Has Controller");
                                }
                            });
                        }
                        for ship_objs in ctx.db().ship().iter() {
                            ui.horizontal(|ui| {
                                ui.label(
                                    format!(
                                        "{}: Sector: {}, Ship: {}, SO: {}",
                                        ship_objs.player_id.to_abbreviated_hex(),
                                        ship_objs.sector_id,
                                        ship_objs.id,
                                        ship_objs.sobj_id
                                    )
                                );
                            });
                        }
                    });
            });

            // for player_controlled in ctx.db().ship_object().iter() {
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
