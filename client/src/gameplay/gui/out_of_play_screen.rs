use std::{ collections::HashMap, f32::consts::PI };

use egui::{ Align, Color32, Context, FontId, Frame, Layout, Rangef, RichText, Shadow, Ui, Vec2 };
use macroquad::prelude::*;
use spacetimedb_sdk::{ DbContext, Table };

use crate::{
    gameplay::{
        gui::ship_details_window::{ show_docked_ship_details, show_ship_details },
        state::{ self, GameState },
    },
    module_bindings::*,
    stdb::utils::*,
};

// #[derive(PartialEq)]
// enum CurrentTab {
//     Ship,
//     Cargo,
//     Equipment,
// }

//#[derive(Default)]
pub struct State {
    // current_tab: CurrentTab, // = CurrentTab::Ship
    // current_equipment_tab: EquipmentSlotType,
    currently_selected_module: Option<(u8, StationModule, StationModuleBlueprint)>,
    selected_ship: Option<DockedShip>,
}

impl State {
    pub fn new() -> Self {
        State {
            currently_selected_module: None,
            selected_ship: None,
        }
    }
}

pub fn draw(
    egui_ctx: &Context,
    ctx: &DbConnection,
    game_state: &mut GameState
) -> egui::InnerResponse<()> {
    egui::CentralPanel
        ::default()
        .frame(
            Frame::group(&egui_ctx.style())
                .fill(Color32::from_rgb(0, 5, 10))
                .multiply_with_opacity(0.75)
                .shadow(Shadow::NONE)
        )
        .show(egui_ctx, |ui| {
            egui::SidePanel
                ::left("left_panel")
                .resizable(true)
                .default_width(320.0)
                .width_range(Rangef::new(150.0, 400.0))
                .show_inside(ui, |ui| {
                    left_panel(ui, ctx, game_state);
                });

            egui::TopBottomPanel
                ::bottom("bottom_chat")
                .resizable(false)
                .min_height(150.0)
                .max_height(screen_height() / 5.0)
                .show_inside(ui, |ui| {
                    super::chat_widget::draw_panel(ui, ctx, &mut game_state.chat_window)
                });

            if game_state.out_of_play_screen.selected_ship.is_some() {
                if let Some(docked_ship) = game_state.out_of_play_screen.selected_ship.clone() {
                    if let Some(station) = ctx.db().station().id().find(&docked_ship.station_id) {
                        show_station_window(egui_ctx, ctx, game_state, docked_ship, station);
                    }
                }
            }

            egui::TopBottomPanel
                ::bottom("bottom_panel")
                .resizable(false)
                .min_height(0.0)
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Bottom Panel");
                    });
                });
        })
}

fn show_station_window(
    egui_ctx: &Context,
    ctx: &DbConnection,
    game_state: &mut GameState<'_>,
    docked_ship: DockedShip,
    station: Station
) {
    egui::Window
        ::new(format!("{} Station - Docked Ship ID {}", station.name, docked_ship.id))
        .title_bar(true)
        .resizable(true)
        .collapsible(true)
        .movable(true)
        .vscroll(true)
        .frame(
            Frame::group(&egui_ctx.style())
                .fill(Color32::from_rgb(0, 5, 10))
                .multiply_with_opacity(0.75)
        )
        .show(egui_ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Station Panel");
            });

            // Show tabs for each
            for (index, module) in ctx
                .db()
                .station_module()
                .iter()
                .filter(|sm| sm.station_id == station.id)
                .enumerate() {
                if
                    let Some(blueprint) = ctx
                        .db()
                        .station_module_blueprint()
                        .id()
                        .find(&module.blueprint)
                {
                    // Check if this is module is selected
                    let mut selected = false;
                    if
                        let Some((selected_index, _, _)) =
                            game_state.out_of_play_screen.currently_selected_module
                    {
                        selected = (index as u8) == selected_index;
                    }

                    //
                    if
                        ui
                            .selectable_label(
                                selected,
                                format!(
                                    "Module #{} {:?}-type: {}",
                                    index,
                                    blueprint.category,
                                    blueprint.name
                                )
                            )
                            .clicked()
                    {
                        game_state.out_of_play_screen.currently_selected_module = Some((
                            index as u8,
                            module.clone(),
                            blueprint.clone(),
                        ));
                    }
                } else {
                    ui.label(format!("Module #{}: Unknown type", index));
                }
            }
            //
            egui::ScrollArea::vertical().show(ui, |ui| {
                if
                    let Some((index, module, blueprint)) =
                        game_state.out_of_play_screen.currently_selected_module.clone()
                {
                    ui.heading(format!("Station Module #{}: {}", index, blueprint.name));
                }

                // ui.label(
                //     "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
                // );
            });
        });
}

fn left_panel(ui: &mut Ui, ctx: &DbConnection, game_state: &mut GameState) {
    let system_to_docked_ships_map = prepare_docked_ships_for_system_tree(ctx);
    let mut sorted_system_to_docked_ships: Vec<_> = system_to_docked_ships_map.values().collect();
    sorted_system_to_docked_ships.sort_by_key(|(system, _)| system.name.clone());

    egui::TopBottomPanel::top("left_panel_top").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Quit").clicked() {
                game_state.done = true;
            }
            if ui.button("Settings").clicked() {
                //
            }
        });
        ui.separator();
    });

    if let Some(ship) = game_state.out_of_play_screen.selected_ship.clone() {
        egui::TopBottomPanel::bottom("left_panel_bottom").show_inside(ui, |ui| {
            ui.heading("Ship Details");
            show_docked_ship_details(ctx, &mut game_state.details_window, ui, ship);
        });
    }

    ui.heading("Assets Tree");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (star_system, sectors_with_ships) in sorted_system_to_docked_ships {
            egui::collapsing_header::CollapsingState
                ::load_with_default_open(
                    ui.ctx(),
                    ui.make_persistent_id(format!("system_{}", star_system.id)),
                    true // Default open state
                )
                .show_header(ui, |ui| {
                    ui.label(format!("System: {} (ID: {})", star_system.name, star_system.id));
                })
                .body(|ui| {
                    display_sectors_with_ships(
                        ctx,
                        sectors_with_ships,
                        ui,
                        &mut game_state.out_of_play_screen
                    );
                });
        }
    });
}

fn display_sectors_with_ships(
    ctx: &DbConnection,
    sectors_with_ships: &Vec<(Sector, Vec<DockedShip>)>,
    ui: &mut Ui,
    state: &mut State
) {
    if sectors_with_ships.is_empty() {
        ui.label("(No sectors with your docked ships in this system)");
    } else {
        for (sector, docked_ships_in_sector) in sectors_with_ships {
            egui::collapsing_header::CollapsingState
                ::load_with_default_open(
                    ui.ctx(),
                    ui.make_persistent_id(format!("sector_{}", sector.id)),
                    true // Default open state
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
                            display_ship_on_tree(ctx, state, ui, ship);
                        }
                    }
                });
        }
    }
}

fn display_ship_on_tree(ctx: &DbConnection, state: &mut State, ui: &mut Ui, ship: &DockedShip) {
    let ship_type = ctx.db().ship_type_definition().id().find(&ship.shiptype_id);

    let mut select_enabled = true;
    if state.selected_ship.clone().is_some_and(|selected| selected.id == ship.id) {
        select_enabled = false;
    }

    ui.horizontal(|ui| {
        // You can make ships collapsible too, or just list them
        ui.label(
            format!(
                "    - Ship: {} (ID: {})",
                if ship_type.is_some() {
                    ship_type.unwrap().name
                } else {
                    "Unknown Ship Type".to_string()
                },
                ship.id
            )
        );

        // Buttons on the right
        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            // Add buttons in reverse order of appearance (rightmost first)
            if ui.button("Undock").clicked() {
                println!("Undock clicked for ship ID: {}", ship.id);
                state.selected_ship = None;
                let _ = ctx.reducers().undock_ship(ShipGlobalId { value: ship.id });
                // TODO Add a system message to alert the player if it failed.
            }
            if select_enabled && ui.button("Select").clicked() {
                println!("Select clicked for ship ID: {}", ship.id);
                // Handle selection, e.g., update some state
                // *selected_ship_id = Some(ship.id);
                state.selected_ship = Some(ship.clone());
            } else if !select_enabled {
                ui.add_enabled(select_enabled, egui::Button::new("Select"));
            }
        });
    });
}

fn collect_docked_ships_per_sector(ctx: &DbConnection) -> HashMap<u64, Vec<DockedShip>> {
    let mut docked_ships_map: HashMap<u64, Vec<DockedShip>> = HashMap::new();

    for docked_ship in ctx
        .db()
        .docked_ship() // Assuming generated table handle
        .iter()
        .filter(|ship| ship.player_id == ctx.identity()) {
        // sector_id is u64, which is a Copy, so no clone needed for the key.
        // Clone the ship itself to store in the Vec.
        docked_ships_map.entry(docked_ship.sector_id).or_default().push(docked_ship.clone());
    }
    docked_ships_map
}

fn prepare_docked_ships_for_system_tree(
    ctx: &DbConnection
) -> HashMap<u32, (StarSystem, Vec<(Sector, Vec<DockedShip>)>)> {
    let docked_ships_per_sector = collect_docked_ships_per_sector(ctx);
    let mut systems_data: HashMap<
        u32,
        (StarSystem, Vec<(Sector, Vec<DockedShip>)>)
    > = HashMap::new();

    for (sector_id, ships_in_this_sector) in docked_ships_per_sector.iter() {
        // Find the sector object for the current sector_id
        if let Some(sector) = ctx.db().sector().id().find(sector_id) {
            // Assuming PK on Sector is 'id'
            // Find the star system for this sector
            if let Some(star_system) = ctx.db().star_system().id().find(&sector.system_id) {
                // Assuming PK on StarSystem is 'id'
                // Get or insert the entry for this star system
                let system_entry = systems_data
                    .entry(star_system.id) // Use system_id as the key
                    .or_insert_with(|| (star_system.clone(), Vec::new()));

                // Add the current sector and its ships to this system's list
                // We clone ships_in_this_sector because we are borrowing it from docked_ships_per_sector
                system_entry.1.push((sector.clone(), ships_in_this_sector.clone()));
            } else {
                info!(
                    "Warning: StarSystem with ID {} not found for sector {}",
                    sector.system_id,
                    sector.name
                );
            }
        } else {
            info!("Warning: Sector with ID {} not found, but ships are docked there.", sector_id);
        }
    }

    // Sort sectors within each system, e.g., by name or ID
    for (_system_id, (_system, sectors_with_ships)) in systems_data.iter_mut() {
        sectors_with_ships.sort_by_key(|(sector, _ships)| sector.id.clone());
        // Or by name: sectors_with_ships.sort_by(|(s1, _), (s2, _)| s1.name.cmp(&s2.name));

        // Optional: Sort ships within each sector
        for (_sector, ships) in sectors_with_ships.iter_mut() {
            ships.sort_by_key(|ship| ship.id.clone());
            // Or by name: ships.sort_by(|s1, s2| s1.name.cmp(&s2.name));
        }
    }

    // // If we want the outer map to be sorted for consistent tree display:
    // let mut sorted_systems_vec: Vec<_> = systems_data.into_iter().collect();
    // sorted_systems_vec.sort_by_key(|(system_id, (system_obj, _))| system_obj.name.clone());

    // sorted_systems_vec // We'll have to change the return value to be a vec, we'll do that elsewhere.

    systems_data
}
