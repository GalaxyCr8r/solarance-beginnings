use egui::{Align2, Color32, Context, RichText, ScrollArea, TextStyle, Ui};
use macroquad::prelude::*;

use crate::{module_bindings::*, stdb::utils::get_player_ship_instance};

#[derive(Default)]
pub struct WindowState {
    pub global_chat_channel: Vec<GlobalChatMessage>,
    pub text: String,
    pub selected_tab: u8,
    pub has_focus: bool,
    pub hidden: bool
}

pub fn window(egui_ctx: &Context, ctx: &DbConnection, state: &mut WindowState) -> Option<egui::InnerResponse<Option<()>>> {
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
                    fun_name(ui, "Health", ship_type.max_health as f32, player_ship.health, Color32::RED);
                    fun_name(ui, "Shields", ship_type.max_shield as f32, player_ship.shields, Color32::DARK_BLUE);
                    fun_name(ui, "Energy", ship_type.max_energy as f32, player_ship.energy, Color32::DARK_GREEN);
                }
            }

        })
}

fn fun_name(ui: &mut Ui, name: &str, max:f32, current: f32, color: Color32) {
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