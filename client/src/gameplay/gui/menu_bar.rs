
use egui::{Align2, Context, FontId, RichText};

use crate::{gameplay::state::GameState, module_bindings::*};

//#[derive(Default)]
pub struct WindowState {
    // current_tab: CurrentTab, // = CurrentTab::Ship
    // current_equipment_tab: EquipmentSlotType,
}

impl WindowState {
    pub fn new() -> Self {
        WindowState {
            // current_tab: CurrentTab::Ship,
            // current_equipment_tab: EquipmentSlotType::Weapon
        }
    }
}

pub fn window(egui_ctx: &Context, ctx: &DbConnection, game_state: &mut GameState) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window
        ::new("Menu Bar")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .movable(false)
        .vscroll(false)
        .anchor(Align2::CENTER_TOP, egui::Vec2::new(0.0, 0.0))
        .show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
              if ui.selectable_label(game_state.details_window_open, RichText::new("SHIP").font(FontId::proportional(20.0))).clicked() {
                game_state.details_window_open = !game_state.details_window_open;
              }
              ui.separator();
              if ui.selectable_label(game_state.faction_window_open, RichText::new("FACTION").font(FontId::proportional(20.0))).clicked() {
                game_state.faction_window_open = !game_state.faction_window_open;
              }
              ui.separator();
              if ui.selectable_label(game_state.assets_window_open, RichText::new("ASSETS").font(FontId::proportional(20.0))).clicked() {
                game_state.assets_window_open = !game_state.assets_window_open;
              }
            });
        })
}