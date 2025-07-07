use std::{ collections::HashMap, f32::consts::PI };

use egui::{ Align, Color32, Context, FontId, Frame, Layout, Rangef, RichText, Shadow, Ui, Vec2 };
use macroquad::prelude::*;
use spacetimedb_sdk::{ DbContext, Table };

pub mod utils;

use crate::{
    gameplay::{
        gui::{
            out_of_play_screen::utils::*,
            ship_details_window::{ show_docked_ship_details, show_ship_details },
        },
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

                    let trading_port = ctx.db().trading_port_module().id().find(&module.id);
                    let refinery = ctx.db().refinery_module().id().find(&module.id);

                    if trading_port.is_some() {
                        ui.label("Trading Port Module connection established.");
                    }

                    if refinery.is_some() {
                        ui.label("Refinery Module connection established.");
                    }

                    for inventory in ctx
                        .db()
                        .station_module_inventory_item()
                        .iter()
                        .filter(|smi| smi.module_id == module.id) {
                        if
                            let Some(item_def) = ctx
                                .db()
                                .item_definition()
                                .id()
                                .find(&inventory.resource_item_id)
                        {
                            ui.label(
                                format!(
                                    "Item: {} (ID: {}) - Quantity: {}",
                                    item_def.name,
                                    item_def.id,
                                    inventory.quantity
                                )
                            );

                            ui.horizontal(|ui| {
                                // Handling Buying only of this is a trading port or it's a refinery's raw resource item
                                let module_can_buy_from_player =
                                    trading_port.is_some() ||
                                    (refinery.is_some() &&
                                        refinery.as_ref().unwrap().input_ore_resource_id ==
                                            inventory.resource_item_id);

                                // Handle Selling only if this is a trading port or it's a refinery's refined (or waste) resource item.
                                let module_can_sell_to_player =
                                    trading_port.is_some() ||
                                    (refinery.is_some() &&
                                        ({
                                            let refinary = refinery.as_ref().unwrap();
                                            refinary.output_ingot_resource_id ==
                                                inventory.resource_item_id ||
                                                refinary.waste_resource_id.is_some_and(
                                                    |waste_id|
                                                        waste_id == inventory.resource_item_id
                                                )
                                        }));

                                if module_can_buy_from_player {
                                    ui.label(RichText::new("Sell:").strong().color(Color32::GREEN));
                                    // TODO check if the player has enough of this item to sell
                                    if ui.button("-1-").clicked() {
                                        match
                                            ctx
                                                .reducers()
                                                .sell_item_to_trading_port(
                                                    module.id.into(),
                                                    docked_ship.id.into(),
                                                    inventory.resource_item_id.into(),
                                                    1
                                                )
                                        {
                                            Ok(_) => {
                                                info!(
                                                    "Sold item {} to trading port",
                                                    inventory.resource_item_id
                                                );
                                            }
                                            Err(e) => {
                                                warn!(
                                                    "Failed to sell item {} to trading port: {}",
                                                    inventory.resource_item_id,
                                                    e
                                                );
                                            }
                                        }
                                    }
                                }

                                if module_can_sell_to_player {
                                    ui.label(RichText::new("Buy:").strong().color(Color32::RED));
                                    // TODO Check if the station has enough of this item to buy.
                                    if ui.button("-1-").clicked() {
                                        if
                                            let Ok(_) = ctx.reducers.buy_item_from_trading_port(
                                                module.id.into(),
                                                docked_ship.id.into(),
                                                inventory.resource_item_id.into(),
                                                1
                                            )
                                        {
                                            info!(
                                                "Bought item {} from trading port",
                                                inventory.resource_item_id
                                            );
                                        } else {
                                            warn!(
                                                "Failed to buy item {} from trading port",
                                                inventory.resource_item_id
                                            );
                                        }
                                    }
                                }
                            });
                        } else {
                            ui.label(format!("Item ID {} not found", inventory.resource_item_id));
                        }
                    }
                }
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
