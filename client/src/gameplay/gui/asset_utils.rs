use crate::module_bindings::*;
use egui::{Align, Layout, Ui};

/// Trait for handling ship selection and actions in the asset tree
/// This allows different windows to implement their own ship selection logic
pub trait ShipTreeHandler {
    /// Check if a ship is currently selected
    fn is_ship_selected(&self, ship: &DockedShip) -> bool;

    /// Handle ship selection
    fn select_ship(&mut self, ship: &DockedShip);

    /// Handle ship deselection (when undocking or other actions)
    fn deselect_ship(&mut self);
}

/// Display sectors with ships in a collapsible tree format
/// This function is used by both the out-of-play screen and the assets window
pub fn display_sectors_with_ships<T: ShipTreeHandler>(
    ctx: &DbConnection,
    sectors_with_ships: &Vec<(Sector, Vec<DockedShip>)>,
    ui: &mut Ui,
    handler: &mut T,
) {
    if sectors_with_ships.is_empty() {
        ui.label("(No sectors with your docked ships in this system)");
    } else {
        for (sector, docked_ships_in_sector) in sectors_with_ships {
            egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                ui.make_persistent_id(format!("sector_{}", sector.id)),
                true, // Default open state
            )
            .show_header(ui, |ui| {
                ui.label(format!("  Sector: {} (ID: {})", sector.name, sector.id));
            })
            .body(|ui| {
                if docked_ships_in_sector.is_empty() {
                    // This case should ideally not happen if collect_docked_ships_per_sector only includes sectors with ships
                    ui.label("    (No docked ships - unexpected)");
                } else {
                    for ship in docked_ships_in_sector {
                        display_ship_on_tree(ctx, handler, ui, ship);
                    }
                }
            });
        }
    }
}

/// Display a single ship in the tree with selection and undock buttons
pub fn display_ship_on_tree<T: ShipTreeHandler>(
    ctx: &DbConnection,
    handler: &mut T,
    ui: &mut Ui,
    ship: &DockedShip,
) {
    let ship_type = ctx.db.ship_type_definition().id().find(&ship.shiptype_id);
    let is_selected = handler.is_ship_selected(ship);

    ui.horizontal(|ui| {
        // Display ship information
        ui.label(format!(
            "    - Ship: {} (ID: {})",
            if let Some(ship_type) = ship_type {
                ship_type.name.clone()
            } else {
                "Unknown Ship Type".to_string()
            },
            ship.id
        ));

        // Buttons on the right
        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            // Add buttons in reverse order of appearance (rightmost first)
            if ui.button("Undock").clicked() {
                println!("Undock clicked for ship ID: {}", ship.id);
                handler.deselect_ship();
                let _ = ctx.reducers.undock_ship(ShipGlobalId { value: ship.id });
                // TODO Add a system message to alert the player if it failed.
            }

            if !is_selected && ui.button("Select").clicked() {
                println!("Select clicked for ship ID: {}", ship.id);
                handler.select_ship(ship);
            } else if is_selected {
                ui.add_enabled(false, egui::Button::new("Select"));
            }
        });
    });
}
