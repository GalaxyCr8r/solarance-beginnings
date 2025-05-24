use egui::{Align2, Color32, Context, Ui};
use macroquad::prelude::*;

use crate::{module_bindings::*, stdb::utils::get_player_ship_instance};

#[derive(Default)]
pub struct WindowState {
    //pub hidden: bool
}

pub fn window(egui_ctx: &Context, ctx: &DbConnection, _state: &mut WindowState) -> Option<egui::InnerResponse<Option<()>>> {
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
                }
            }
        })
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