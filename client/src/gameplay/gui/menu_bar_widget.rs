
use egui::{Align2, Color32, Context, FontId, Frame, RichText, Shadow, Ui};

use crate::{
    gameplay::{hotkeys::Action, state::GameState},
    server::bindings::*,
};

//#[derive(Default)]
pub struct State {
    // current_tab: CurrentTab, // = CurrentTab::Ship
    // current_equipment_tab: EquipmentSlotType,
}

impl State {
    pub fn new() -> Self {
        State {
            // current_tab: CurrentTab::Ship,
            // current_equipment_tab: EquipmentSlotType::Weapon
        }
    }
}

pub fn draw(egui_ctx: &Context, _ctx: &DbConnection, game_state: &mut GameState) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window
        ::new("Menu Bar")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .movable(false)
        .vscroll(false)
        .frame(Frame::group(&egui_ctx.style()).fill(Color32::from_rgb(15, 15, 15)).shadow(Shadow::NONE))
        .anchor(Align2::CENTER_TOP, egui::Vec2::new(0.0, 0.0))
        .show(egui_ctx, |ui| {
            // Keys come from the binding list, so a remap renames the tab
            // instead of leaving a stale hint behind (#218).
            let ship = game_state.hotkeys.hint(Action::ShipWindow);
            let faction = game_state.hotkeys.key_name(Action::FactionWindow);
            let assets = game_state.hotkeys.key_name(Action::AssetsWindow);
            let map = game_state.hotkeys.key_name(Action::MapWindow);
            let build = game_state.hotkeys.key_name(Action::BuildWindow);

            ui.horizontal(|ui| {
              toggable_label(ui, &format!("{ship} SHIP"), &mut game_state.details_window_open);
              ui.separator();
              toggable_label(ui, &format!("[{faction}]ACTION"), &mut game_state.faction_window_open);
              ui.separator();
              toggable_label(ui, &format!("ASSE[{assets}]S"), &mut game_state.assets_window_open);
              ui.separator();
              toggable_label(ui, &format!("[{map}]AP"), &mut game_state.map_window_open);
              ui.separator();
              toggable_label(ui, &format!("[{build}]UILD"), &mut game_state.construction_window_open);
            });
        })
}

fn toggable_label(ui: &mut Ui, label: &str, open: &mut bool) {
  if ui.selectable_label(*open, RichText::new(label).font(FontId::proportional(20.0))).clicked() {
    *open = !*open;
  }
}