use egui::{Align2, Context};

use crate::{gameplay::state::GameState, module_bindings::*, stdb::utils::get_player_ship_instance};

#[derive(PartialEq)]
enum CurrentTab {
    Ship,
    Cargo
}

pub struct State {
    current_tab: CurrentTab,
}

impl State {
    pub fn new() -> Self {
        State {
            current_tab: CurrentTab::Ship,
        }
    }
}

pub fn draw(egui_ctx: &Context, game_state: &mut GameState) -> Option<egui::InnerResponse<Option<()>>> {
    let ctx = game_state.ctx;
    
    egui::Window
        ::new("Minimap")
        .title_bar(true)
        .resizable(false)
        .collapsible(true)
        .movable(false)
        .anchor(Align2::RIGHT_BOTTOM, egui::Vec2::new(0.0, 0.0))
        .show(egui_ctx, |ui| {
            if let Some(player_ship) = get_player_ship_instance(ctx) {
                if let Some(sector) = ctx.db.sector().id().find(&player_ship.current_sector_id) {
                    ui.label("Current Sector:");
                    ui.label(sector.name);

                    // ui.separator();

                    // match state.current_tab {
                    //     CurrentTab::Ship => { todo!() },
                    //     CurrentTab::Cargo => { todo!() },
                    // }
                }
            }
        })
}