use std::collections::HashMap;

use egui::{ Align, Color32, Context, Frame, Layout, Rangef, RichText, Shadow, Ui };
use macroquad::prelude::*;
use spacetimedb_sdk::{ DbContext, Table };

pub mod utils;

use crate::{
    gameplay::{
        gui::{ out_of_play_screen::utils::*, ship_details_window::show_docked_ship_details },
        state::{ GameState },
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

    /// A hashmap of station module IDs + item def IDs to the currently selected buy/sell amounts.
    buy_sell_scalars: HashMap<(u64, u32), (u32, u32)>,
}

impl State {
    pub fn new() -> Self {
        State {
            currently_selected_module: None,
            selected_ship: None,
            buy_sell_scalars: HashMap::new(),
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
            ui.horizontal(|ui| {
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
                        show_station_module(game_state, ui, index, module, blueprint);
                    } else {
                        ui.label(format!("Module #{} (Unknown)", index));
                    }
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                if
                    let Some((index, module, blueprint)) =
                        game_state.out_of_play_screen.currently_selected_module.clone()
                {
                    ui.group(|ui| {
                        show_currently_selected_module(
                            ctx,
                            &mut game_state.out_of_play_screen,
                            docked_ship,
                            ui,
                            index,
                            module,
                            blueprint
                        );
                    });
                }
            });
        });
}

fn show_station_module(
    game_state: &mut GameState<'_>,
    ui: &mut Ui,
    index: usize,
    module: StationModule,
    blueprint: StationModuleBlueprint
) {
    // Check if this is module is selected
    let mut selected = false;
    if let Some((selected_index, _, _)) = game_state.out_of_play_screen.currently_selected_module {
        selected = (index as u8) == selected_index;
    }

    if ui.selectable_label(selected, format!("Module #{} ({})", index, blueprint.name)).clicked() {
        game_state.out_of_play_screen.currently_selected_module = Some((
            index as u8,
            module.clone(),
            blueprint.clone(),
        ));
    }
}

fn show_currently_selected_module(
    ctx: &DbConnection,
    state: &mut State,
    docked_ship: DockedShip,
    ui: &mut Ui,
    index: u8,
    module: StationModule,
    blueprint: StationModuleBlueprint
) {
    ui.heading(format!("Station Module #{}: {}", index, blueprint.name));

    let trading_port = ctx.db().trading_port_module().id().find(&module.id);
    let refinery = ctx.db().refinery_module().id().find(&module.id);

    if trading_port.is_some() {
        ui.label("Trading Port Module connection established.");
    }

    if refinery.is_some() {
        ui.label("Refinery Module connection established.");
    }

    // Sort the station module inventory items by their item names
    let mut inventory_names_to_item_definition_ids_list: Vec<(String, u64)> = Vec::new();
    let mut inventory_item_ids_to_item_definition_map: HashMap<
        u64,
        (StationModuleInventoryItem, ItemDefinition)
    > = HashMap::new();
    for inventory in ctx
        .db()
        .station_module_inventory_item()
        .iter()
        .filter(|smi| smi.module_id == module.id) {
        if let Some(item_def) = ctx.db().item_definition().id().find(&inventory.resource_item_id) {
            if let ItemCategory::Resource(category) = item_def.category {
                inventory_item_ids_to_item_definition_map.insert(inventory.id, (
                    inventory.clone(),
                    item_def.clone(),
                ));
                inventory_names_to_item_definition_ids_list.push((
                    format!("{:?}: {}", category, item_def.name.clone()),
                    inventory.id,
                ));
            }
        } else {
            warn!("Item def for resource item ID {} not found!", inventory.resource_item_id);
        }
    }
    inventory_names_to_item_definition_ids_list.sort_by(|(str_a, _), (str_b, _)|
        str_a.to_lowercase().cmp(&str_b.to_lowercase())
    );

    for (inventory_label, inventory_id) in inventory_names_to_item_definition_ids_list {
        if
            let Some((inventory, item_def)) = inventory_item_ids_to_item_definition_map.get(
                &inventory_id
            )
        {
            let id = ui.make_persistent_id(format!("{}.{}", module.id, inventory.id));
            egui::collapsing_header::CollapsingState
                ::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| {
                    ui.label(RichText::new(inventory_label).strong());
                    ui.label(format!("--- Cost: {}c --- ", inventory.cached_price));
                    ui.add(
                        egui::ProgressBar
                            ::new((inventory.quantity as f32) / (inventory.max_quantity as f32))
                            .text(format!("{} / {}", inventory.quantity, inventory.max_quantity))
                    );
                })
                .body(|ui|
                    buy_and_sell_inventory_item(
                        ctx,
                        state,
                        &docked_ship,
                        ui,
                        &module,
                        &trading_port,
                        &refinery,
                        &inventory,
                        &item_def
                    )
                );
        }
    }

    //
}

fn buy_and_sell_inventory_item(
    ctx: &DbConnection,
    state: &mut State,
    docked_ship: &DockedShip,
    ui: &mut Ui,
    module: &StationModule,
    trading_port: &Option<TradingPort>,
    refinery: &Option<Refinery>,
    inventory: &StationModuleInventoryItem,
    item_def: &ItemDefinition
) {
    ui.label(item_def.clone().description.unwrap_or("No description available.".to_string()));
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(format!("Base Value: {}c", item_def.base_value));
        ui.spacing();
        ui.label(format!("Station's Value: {}c", inventory.cached_price));
    });
    ui.label(format!("Volume per Unit: {}v", item_def.volume_per_unit));

    let (mut buy_scalar, mut sell_scalar) = {
        let tmp = state.buy_sell_scalars.get(&(module.id, item_def.id));

        match tmp {
            Some(scalars) => scalars.clone(),
            None => {
                let default = (0, 0);
                state.buy_sell_scalars.insert((module.id, item_def.id), default);
                default
            }
        }
    };

    ui.group(|ui| {
        ui.horizontal(|ui| {
            let players_current_amount = {
                ctx.db()
                    .ship_cargo_item()
                    .iter()
                    .filter(|sci| {
                        //info!("Found cargo item {} for ship {}", sci.item_id, sci.ship_id);

                        sci.ship_id == docked_ship.id && sci.item_id == inventory.resource_item_id
                    })
                    .map(|sci| {
                        // info!(
                        //     "Found cargo item {} with quantity {} in player inventory!!",
                        //     sci.item_id,
                        //     sci.quantity
                        // );
                        sci.quantity as u32
                    })
                    .sum::<u32>()
            };

            // Handling Buying only of this is a trading port or it's a refinery's raw resource item
            let module_can_buy_from_player =
                trading_port.is_some() ||
                (refinery.is_some() &&
                    refinery.as_ref().unwrap().input_ore_resource_id == inventory.resource_item_id);

            ui.label(RichText::new("Sell:").strong().color(Color32::GREEN));
            if module_can_buy_from_player {
                ui.add_enabled(
                    players_current_amount > 0,
                    egui::Slider::new(&mut sell_scalar, 0..=players_current_amount)
                );
                ui.label(format!("{}c", sell_scalar * inventory.cached_price));
                sell_item_to_station(ctx, sell_scalar, docked_ship, module, inventory, ui);

                if players_current_amount == 0 {
                    ui.label("You do not have any of this item.");
                }
            } else {
                ui.add_enabled_ui(false, |ui| {
                    ui.label("You cannot sell this item here.");
                });
            }
        });

        ui.horizontal(|ui| {
            // Handle Selling only if this is a trading port or it's a refinery's refined (or waste) resource item.
            let module_can_sell_to_player =
                trading_port.is_some() ||
                (refinery.is_some() &&
                    ({
                        let refinary = refinery.as_ref().unwrap();
                        refinary.output_ingot_resource_id == inventory.resource_item_id ||
                            refinary.waste_resource_id.is_some_and(
                                |waste_id| waste_id == inventory.resource_item_id
                            )
                    }));
            let space_available = {
                if let Some(status) = ctx.db().ship_status().id().find(&docked_ship.id) {
                    (status.max_cargo_capacity - status.used_cargo_capacity) /
                        item_def.volume_per_unit
                } else {
                    0
                }
            };

            ui.label(RichText::new("Buy:").strong().color(Color32::RED));
            if module_can_sell_to_player {
                ui.add_enabled(
                    inventory.quantity > 0 && space_available > 0,
                    egui::Slider::new(
                        &mut buy_scalar,
                        0..=({
                            if (space_available as u32) > inventory.quantity {
                                inventory.quantity
                            } else {
                                space_available as u32
                            }
                        })
                    )
                );
                ui.label(format!("{}c", buy_scalar * inventory.cached_price));
                buy_item_from_station(ctx, buy_scalar, docked_ship, module, inventory, ui);

                if space_available == 0 {
                    ui.label("Your ship has no space for this item.");
                } else if inventory.quantity > (space_available as u32) {
                    ui.label(format!("Limited to {} due to cargo space.", space_available));
                }
                if inventory.quantity == 0 {
                    ui.label("Module doesn't have any of this item.");
                }
            } else {
                ui.add_enabled_ui(false, |ui| {
                    ui.label("You cannot buy this item here.");
                });
            }
        });
    });

    state.buy_sell_scalars.insert((module.id, item_def.id), (buy_scalar, sell_scalar));
}

fn buy_item_from_station(
    ctx: &DbConnection,
    quantity: u32,
    docked_ship: &DockedShip,
    module: &StationModule,
    inventory: &StationModuleInventoryItem,
    ui: &mut Ui
) {
    if quantity == 0 {
        ui.label("BUY");
        return;
    }

    if ui.button("BUY").clicked() {
        if
            let Ok(_) = ctx.reducers.buy_item_from_station_module(
                module.id.into(),
                docked_ship.id.into(),
                inventory.resource_item_id.into(),
                quantity
            )
        {
            info!("Bought {} item(s) {} from trading port", quantity, inventory.resource_item_id);
        } else {
            warn!(
                "Failed to buy {} item(s) {} from trading port",
                quantity,
                inventory.resource_item_id
            );
        }
    }
}

fn sell_item_to_station(
    ctx: &DbConnection,
    quantity: u32,
    docked_ship: &DockedShip,
    module: &StationModule,
    inventory: &StationModuleInventoryItem,
    ui: &mut Ui
) {
    if quantity == 0 {
        ui.label("SELL");
        return;
    }

    if ui.button("SELL").clicked() {
        match
            ctx
                .reducers()
                .sell_item_to_station_module(
                    module.id.into(),
                    docked_ship.id.into(),
                    inventory.resource_item_id.into(),
                    quantity
                )
        {
            Ok(_) => {
                info!("Sold {} item(s) {} to trading port", quantity, inventory.resource_item_id);
            }
            Err(e) => {
                warn!(
                    "Failed to sell {} item(s) {} to trading port: {}",
                    quantity,
                    inventory.resource_item_id,
                    e
                );
            }
        }
    }
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
    ui.label(
        format!(
            "Credits: {}",
            get_player(&game_state.ctx.db, &game_state.ctx.identity()).map_or_else(
                || 0,
                |player| player.credits
            )
        )
    );

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
