use egui::{ Align2, Color32, Context, RichText, Ui };
use macroquad::{miniquad::date::now, prelude::*};
use spacetimedb_sdk::DbContext;

use crate::{
    gameplay::state::{ GameState },
    module_bindings::*,
    stdb::utils::{
        get_player_ship_instance,
        get_player_transform,
        get_transform,
    },
};

#[derive(Default)]
pub struct WindowState {
    //pub hidden: bool
}

pub fn window(
    egui_ctx: &Context,
    ctx: &DbConnection,
    game_state: &mut GameState
) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window
        ::new("Status Window")
        .title_bar(false)
        .resizable(false)
        .collapsible(true)
        .movable(false)
        .anchor(Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, 0.0))
        .show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(player_ship) = get_player_ship_instance(ctx) {
                    if
                        let Some(ship_type) = ctx.db
                            .ship_type_definition()
                            .id()
                            .find(&player_ship.shiptype_id)
                    {
                        ship_function_status(ctx, ui);

                        ui.separator();
                        ship_status(ui, ship_type, player_ship);
                        ui.separator();

                        if game_state.current_target_sobj != None {
                            ui.vertical(|ui| {
                                let _ = add_targeted_object_status(
                                    ui,
                                    ctx,
                                    &game_state.current_target_sobj.clone().unwrap()
                                );
                            });
                        } else {
                            ui.label("No Target");
                        }
                    }
                }
            });
        })
}

fn ship_status(ui: &mut Ui, ship_type: ShipTypeDefinition, player_ship: ShipInstance) {
    ui.vertical(|ui| {
        add_status_bar(
            ui,
            "Health",
            ship_type.max_health as f32,
            player_ship.health,
            Color32::from_rgb(242, 0, 32),
            true
        );
        add_status_bar(
            ui,
            "Shields",
            ship_type.max_shield as f32,
            player_ship.shields,
            Color32::from_rgb(0, 64, 192),
            true
        );
        add_status_bar(
            ui,
            "Energy",
            ship_type.max_energy as f32,
            player_ship.energy,
            Color32::from_rgb(0, 100, 64),
            true
        );
    });
}

fn ship_function_status(ctx: &DbConnection, ui: &mut Ui) {
    ui.vertical(|ui| {
        if let Some(controller) = ctx.db.player_controller().identity().find(&ctx.identity()) {
            if controller.cargo_bay_open {
                ui.label(RichText::new("Cargo Bay: Open").color({
                    if now() % 1.0 < 0.45 {
                        Color32::YELLOW
                    } else {
                        Color32::BLACK
                    }
                }));
            } else {
                ui.label(RichText::new("Cargo Bay: Closed").color(Color32::DARK_GRAY));
            }
            if controller.mining_laser_on {
                ui.label(RichText::new("Mining Beam: On").color({
                    if now() % 1.0 < 0.45 {
                        Color32::RED
                    } else {
                        Color32::BLACK
                    }
                }));
            } else {
                ui.label(RichText::new("Mining Beam: Off").color(Color32::DARK_GRAY));
            }
            if controller.dock {
                ui.label(RichText::new("Autodocking: On").color({
                    if now() % 1.0 < 0.45 {
                        Color32::LIGHT_BLUE
                    } else {
                        Color32::BLACK
                    }
                }));
            } else {
                ui.label(RichText::new("Autodocking: Off").color(Color32::DARK_GRAY));
            }
        }
    });
}

fn add_targeted_object_status(
    ui: &mut Ui,
    ctx: &DbConnection,
    target: &StellarObject
) -> Result<(), String> {
    let mut kind = "Unknown Object".to_string();
    let distance = {
        if let Some(player_ship) = get_player_transform(ctx) {
            if let Ok(target_object) = get_transform(ctx, target.id) {
                if let Some(sobj) = ctx.db.stellar_object().id().find(&target_object.sobj_id) {
                    kind = format!("{:?}", sobj.kind);
                }

                let target_position = target_object.to_vec2();
                let player_position = player_ship.to_vec2();
                player_position.distance(target_position)
            } else {
                999999.9f32 /* If target isn't found somehow. */
            }
        } else {
            999999.9f32 /* If target isn't found somehow. */
        }
    };

    ui.label(format!("Target: {}", kind));
    ui.label(format!("Distance: {:.0}", distance));

    match target.kind {
        StellarObjectKinds::Asteroid => {
            if let Some(asteroid) = ctx.db.asteroid().sobj_id().find(&target.id) {
                add_status_bar(
                    ui,
                    "Resources",
                    asteroid.initial_resources as f32,
                    asteroid.current_resources as f32,
                    Color32::from_rgb(96, 82, 128),
                    false
                );
            }
        }
        StellarObjectKinds::Ship => {
            if let Some(ship_instance) = ctx.db.ship_instance().id().find(&target.id) {
                if
                    let Some(ship_type) = ctx.db
                        .ship_type_definition()
                        .id()
                        .find(&ship_instance.shiptype_id)
                {
                    add_status_bar(
                        ui,
                        "Health",
                        ship_type.max_health as f32,
                        ship_instance.health,
                        Color32::from_rgb(242, 0, 32),
                        true
                    );
                    add_status_bar(
                        ui,
                        "Shields",
                        ship_type.max_shield as f32,
                        ship_instance.shields,
                        Color32::from_rgb(0, 64, 192),
                        true
                    );
                    add_status_bar(
                        ui,
                        "Energy",
                        ship_type.max_energy as f32,
                        ship_instance.energy,
                        Color32::from_rgb(0, 100, 64),
                        true
                    );
                }
            }
        }
        StellarObjectKinds::Station => {
            // if let Some(station) = ctx.db.station().sobj_id().find(&target.id) {
            //     add_status_bar(ui, "Health", station.max_health as f32, station.health, Color32::from_rgb(242, 0, 32));
            // }
        }
        StellarObjectKinds::CargoCrate => {
            if let Some(cargo_crate) = ctx.db.cargo_crate().sobj_id().find(&target.id) {
                if let Some(item_def) = ctx.db.item_definition().id().find(&cargo_crate.item_id) {
                    ui.label(format!("Contains: {}x {}", cargo_crate.quantity, item_def.name));
                }
                //add_status_bar(ui, "Health", crate_.max_health as f32, crate_.health, Color32::from_rgb(242, 0, 32));
            }
        }
        StellarObjectKinds::JumpGate => {
            if let Some(jump_gate) = ctx.db.jump_gate().sobj_id().find(&target.id) {
                ui.horizontal(|ui| {
                    ui.label("Destination:");
                    if let Some(sector) = ctx.db.sector().id().find(&jump_gate.target_sector_id) {
                        ui.label(format!("{}", sector.name));
                    } else {
                        ui.label(format!("Sector #{}", jump_gate.target_sector_id));
                    }
                });
            }
        }
        //_ => {}
    }
    Ok(())
}

fn add_status_bar(ui: &mut Ui, name: &str, max: f32, current: f32, color: Color32, horiz: bool) {
    let contents = |ui: &mut Ui| {
        ui.label(name);

        let progress_bar = egui::ProgressBar
            ::new(current / max)
            .show_percentage()
            .fill(color)
            .desired_width(128.0);
        ui.add(progress_bar).on_hover_text(format!("{}/{}", current, max)).hovered();
    };

    if horiz {
        ui.horizontal(contents);
    } else {
        ui.vertical(contents);
    }
}
