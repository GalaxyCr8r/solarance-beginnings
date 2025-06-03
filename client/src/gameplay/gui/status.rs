use egui::{Align2, Color32, Context, Ui};
use macroquad::prelude::*;

use crate::{gameplay::state::{GameState, Targets}, module_bindings::*, stdb::utils::{get_player_ship_instance, get_player_transform, get_player_transform_vec2, get_transform}};

#[derive(Default)]
pub struct WindowState {
    //pub hidden: bool
}

pub fn window(egui_ctx: &Context, ctx: &DbConnection, game_state: &mut GameState) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window
        ::new("Status Window")
        .title_bar(false)
        .resizable(false)
        .collapsible(true)
        .movable(false)
        .anchor(Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, 0.0))
        .show(egui_ctx, |ui| {
            if let Some(player_ship) = get_player_ship_instance(ctx) {
                if let Some(ship_type) = ctx.db.ship_type_definition().id().find(&player_ship.shiptype_id) {
                    add_status_bar(ui, "Health", ship_type.max_health as f32, player_ship.health, Color32::from_rgb(242, 0, 32));
                    add_status_bar(ui, "Shields", ship_type.max_shield as f32, player_ship.shields, Color32::from_rgb(0, 64, 192));
                    add_status_bar(ui, "Energy", ship_type.max_energy as f32, player_ship.energy, Color32::from_rgb(0, 100, 64));

                    if game_state.current_target != Targets::None {
                        add_targeted_object_status(ui, ctx, &game_state.current_target);
                    } else {
                        ui.label("No Target");
                    }
                }
            }
        })
}

fn add_targeted_object_status(ui: &mut Ui, ctx: &DbConnection, current_target: &Targets) -> Result<(), String> {
    let distance = {
        let current_target_sobj_id = match current_target {
            Targets::Asteroid(id) => *id,
            Targets::Ship(id) => *id,
            Targets::Station(id) => *id,
            Targets::CargoCrate(id) => *id,
            Targets::JumpGate(id) => *id,
            _ => return Ok(()),
        };

        if let Some(player_ship) = get_player_transform(ctx) {
            if let Ok(target_object) = get_transform(ctx, current_target_sobj_id) {
                let target_position = target_object.to_vec2();
                let player_position = player_ship.to_vec2();
                player_position.distance(target_position)
            } else { 999999.9f32 /* If target isn't found somehow. */ }
        } else { 999999.9f32 /* If target isn't found somehow. */ }
    };
    ui.horizontal(|ui| {
        ui.label(format!("Target: {:?}", current_target));
        ui.label(format!("Distance: {}", distance))
    });
    match current_target {
        Targets::Asteroid(id) => {
            if let Some(asteroid) = ctx.db.asteroid().sobj_id().find(id) {
                add_status_bar(ui, "Resources", asteroid.initial_resources as f32, asteroid.current_resources as f32, Color32::from_rgb(96, 82, 128));
            }
        },
        Targets::Ship(id) => {
            if let Some(ship_instance) = ctx.db.ship_instance().id().find(id) {
                if let Some(ship_type) = ctx.db.ship_type_definition().id().find(&ship_instance.shiptype_id) {
                    add_status_bar(ui, "Health", ship_type.max_health as f32, ship_instance.health, Color32::from_rgb(242, 0, 32));
                    add_status_bar(ui, "Shields", ship_type.max_shield as f32, ship_instance.shields, Color32::from_rgb(0, 64, 192));
                    add_status_bar(ui, "Energy", ship_type.max_energy as f32, ship_instance.energy, Color32::from_rgb(0, 100, 64));
                }
            }
        },
        Targets::Station(_id) => {
            // if let Some(station) = ctx.db.station().sobj_id().find(id) {
            //     add_status_bar(ui, "Health", station.max_health as f32, station.health, Color32::from_rgb(242, 0, 32));
            // }
        },
        Targets::CargoCrate(_id) => {
            // if let Some(crate_) = ctx.db.cargo_crate().sobj_id().find(id) {
            //     add_status_bar(ui, "Health", crate_.max_health as f32, crate_.health, Color32::from_rgb(242, 0, 32));
            // }
        },
        Targets::JumpGate(id) => {
            if let Some(jump_gate) = ctx.db.jump_gate().sobj_id().find(id) {
                if let Some(sector) = ctx.db.sector().id().find(&jump_gate.target_sector_id) {
                    ui.label(format!("Destination: {}", sector.name));
                } else {
                    ui.label(format!("Destination: Sector #{}", jump_gate.target_sector_id));
                }
            }
        },
        _ => {}
    }
    Ok(())
}

fn add_status_bar(ui: &mut Ui, name: &str, max:f32, current: f32, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(name);
    
        let progress_bar = egui::ProgressBar::new(current / max)
            .show_percentage()
            .fill(color)
            .desired_width(128.0);
        ui
            .add(progress_bar)
            .on_hover_text(format!("{}/{}", current, max))
            .hovered();
    });
}