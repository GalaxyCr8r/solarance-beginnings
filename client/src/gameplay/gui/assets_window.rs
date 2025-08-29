use egui::Context;
use spacetimedb_sdk::*;

use crate::{
    gameplay::gui::{
        asset_utils::{display_sectors_with_ships, ShipTreeHandler},
        out_of_play_screen::utils::prepare_docked_ships_for_system_tree,
    },
    module_bindings::*,
    stdb::utils::*,
};

pub struct State {
    selected_ship: Option<DockedShip>,
}

impl State {
    pub fn new() -> Self {
        State {
            selected_ship: None,
        }
    }
}

impl ShipTreeHandler for State {
    fn is_ship_selected(&self, ship: &DockedShip) -> bool {
        self.selected_ship
            .as_ref()
            .map_or(false, |selected| selected.id == ship.id)
    }

    fn select_ship(&mut self, ship: &DockedShip) {
        self.selected_ship = Some(ship.clone());
    }

    fn deselect_ship(&mut self) {
        self.selected_ship = None;
    }
}

pub fn draw(
    egui_ctx: &Context,
    ctx: &DbConnection,
    state: &mut State,
    open: &mut bool,
) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window::new("Assets")
        .open(open)
        .title_bar(true)
        .resizable(true)
        .collapsible(true)
        .movable(true)
        .vscroll(true)
        .default_width(350.0)
        .default_height(500.0)
        .show(egui_ctx, |ui| {
            ui.heading("Player Assets");
            ui.separator();

            // Display player credits
            ui.label(format!(
                "Credits: {}",
                get_player(&ctx.db, &ctx.identity()).map_or_else(|| 0, |player| player.credits)
            ));

            ui.separator();
            ui.heading("Docked Ships");

            // Prepare the system tree data
            let system_to_docked_ships_map = prepare_docked_ships_for_system_tree(ctx);
            let mut sorted_system_to_docked_ships: Vec<_> =
                system_to_docked_ships_map.values().collect();
            sorted_system_to_docked_ships.sort_by_key(|(system, _)| system.name.clone());

            egui::ScrollArea::vertical().show(ui, |ui| {
                if sorted_system_to_docked_ships.is_empty() {
                    ui.label("No docked ships found.");
                } else {
                    for (star_system, sectors_with_ships) in sorted_system_to_docked_ships {
                        egui::collapsing_header::CollapsingState::load_with_default_open(
                            ui.ctx(),
                            ui.make_persistent_id(format!("system_{}", star_system.id)),
                            true, // Default open state
                        )
                        .show_header(ui, |ui| {
                            ui.label(format!(
                                "System: {} (ID: {})",
                                star_system.name, star_system.id
                            ));
                        })
                        .body(|ui| {
                            display_sectors_with_ships(ctx, sectors_with_ships, ui, state);
                        });
                    }
                }
            });
        })
}
