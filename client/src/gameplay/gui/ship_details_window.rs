use std::f32::consts::PI;

use egui::{Context, FontId, RichText, Ui};
use macroquad::prelude::*;
use spacetimedb_sdk::*;

use crate::{module_bindings::*, stdb::utils::*};

#[derive(PartialEq)]
enum CurrentTab {
    Ship,
    Cargo,
    Equipment,
}

//#[derive(Default)]
pub struct State {
    current_tab: CurrentTab, // = CurrentTab::Ship
    current_equipment_tab: EquipmentSlotType,
}

impl State {
    pub fn new() -> Self {
        State {
            current_tab: CurrentTab::Ship,
            current_equipment_tab: EquipmentSlotType::Weapon,
        }
    }
}

pub fn draw(
    egui_ctx: &Context,
    ctx: &DbConnection,
    state: &mut State,
    open: &mut bool,
) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window::new("Ship Details")
        .open(open)
        .title_bar(true)
        .resizable(true)
        .collapsible(true)
        .movable(true)
        .vscroll(true)
        .show(egui_ctx, |ui| {
            if let Some(player_ship) = get_player_ship(ctx) {
                if let Some(player_ship_status) = player_ship.status(ctx) {
                    if let Some(ship_type) = ctx
                        .db()
                        .ship_type_definition()
                        .id()
                        .find(&player_ship.shiptype_id)
                    {
                        ui.horizontal_top(|ui| {
                            ui.selectable_value(
                                &mut state.current_tab,
                                CurrentTab::Ship,
                                RichText::new("Ship").font(FontId::proportional(20.0)),
                            );
                            ui.selectable_value(
                                &mut state.current_tab,
                                CurrentTab::Cargo,
                                RichText::new("Cargo").font(FontId::proportional(20.0)),
                            );
                            ui.selectable_value(
                                &mut state.current_tab,
                                CurrentTab::Equipment,
                                RichText::new("Equipment").font(FontId::proportional(20.0)),
                            );
                        });

                        ui.separator();

                        match state.current_tab {
                            CurrentTab::Ship => {
                                ship_contents(
                                    ui,
                                    ctx,
                                    state,
                                    ship_type,
                                    player_ship,
                                    player_ship_status,
                                );
                            }
                            CurrentTab::Cargo => {
                                cargo_contents(
                                    ui,
                                    ctx,
                                    state,
                                    ship_type,
                                    player_ship,
                                    player_ship_status,
                                );
                            }
                            CurrentTab::Equipment => {
                                equipment_contents(ui, ctx, state, ship_type, player_ship);
                            }
                        }
                    }
                }
            }
        })
}

fn ship_contents(
    ui: &mut Ui,
    ctx: &DbConnection,
    _state: &mut State,
    ship_type: ShipTypeDefinition,
    player_ship: Ship,
    player_ship_status: ShipStatus,
) {
    ui.heading("Detailed Ship Status");
    ui.separator();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(format!("Ship Type: {}", ship_type.name));
            ui.label(format!("Class: {}", ship_type.class.to_string()));
        });
        ui.collapsing("Description", |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(ship_type.description.unwrap_or("n/a".to_string()));
            })
        });
    });
    ui.separator();
    ui.heading("Stats");
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(format!("Health: {}", player_ship_status.health));
            ui.label(format!("Shield: {}", player_ship_status.shields));
            ui.label(format!("Energy: {}", player_ship_status.energy));
            ui.label(format!(
                "Cargo: {} / {}",
                player_ship_status.used_cargo_capacity, player_ship_status.max_cargo_capacity
            ));
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.label(format!("Max Health: {}", ship_type.max_health));
            ui.label(format!("Max Shield: {}", ship_type.max_shields));
            ui.label(format!("Max Energy: {}", ship_type.max_energy));
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.label(format!("Speed: {}", ship_type.base_speed));
            ui.label(format!("Acceleration: {}", ship_type.base_acceleration));
            ui.label(format!(
                "Turn Rate: {}d",
                ship_type.base_turn_rate * (180.0 / PI)
            ));
        });
    });
    ui.heading("Equipment Slots");
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(format!(
                "Weapon Slots: {}/{}",
                player_ship
                    .get_all_equipped_of_type(ctx, EquipmentSlotType::Weapon)
                    .iter()
                    .count(),
                ship_type.num_weapon_slots
            ));
            ui.label(format!(
                "Shield Slots: {}/{}",
                player_ship
                    .get_all_equipped_of_type(ctx, EquipmentSlotType::Shield)
                    .iter()
                    .count(),
                ship_type.num_shield_slots
            ));
            ui.label(format!(
                "Engine Slots: {}/{}",
                player_ship
                    .get_all_equipped_of_type(ctx, EquipmentSlotType::Engine)
                    .iter()
                    .count(),
                ship_type.num_engine_slots
            ));
            ui.label(format!(
                "Mining Laser Slots: {}/{}",
                player_ship
                    .get_all_equipped_of_type(ctx, EquipmentSlotType::MiningLaser)
                    .iter()
                    .count(),
                ship_type.num_mining_laser_slots
            ));
            ui.label(format!(
                "Special Slots: {}/{}",
                player_ship
                    .get_all_equipped_of_type(ctx, EquipmentSlotType::Special)
                    .iter()
                    .count(),
                ship_type.num_special_slots
            ));
        });
    });
}

fn cargo_contents(
    ui: &mut Ui,
    ctx: &DbConnection,
    _state: &mut State,
    _ship_type: ShipTypeDefinition,
    player_ship: Ship,
    player_ship_status: ShipStatus,
) {
    ui.heading("Cargo Bay Contents");
    ui.separator();
    let mut total_cargo_usage = 0;
    for cargo in ctx.db().ship_cargo_item().iter() {
        if cargo.ship_id == player_ship.id {
            // TECHNICALLY RLS should do this for us.
            if let Some(item) = ctx.db().item_definition().id().find(&cargo.item_id) {
                total_cargo_usage += (item.volume_per_unit * cargo.quantity) as i32;

                ui.collapsing(
                    format!(
                        "Cargo: {} x {}u @ {}v",
                        item.name,
                        cargo.quantity,
                        item.volume_per_unit * cargo.quantity
                    ),
                    |ui| {
                        ui.heading(item.name);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(item.description.unwrap_or("n/a".to_string()));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("Volume: {} per unit,", item.volume_per_unit));
                            ui.label(format!("{} units per stack,", item.units_per_stack));
                            ui.label(format!(
                                "{} total volume",
                                item.volume_per_unit * cargo.quantity
                            ));
                        });
                        ui.separator();
                        ui.label(format!("Base Value: {} credits", item.base_value));

                        ui.horizontal(|ui| {
                            ui.label("Jettison:");
                            if ui.button("-1-").clicked() {
                                let _ = ctx.reducers.jettison_cargo_from_ship(
                                    cargo.ship_id,
                                    cargo.id,
                                    1,
                                );
                            }
                            if cargo.quantity > 1 && ui.button("-2-").clicked() {
                                let _ = ctx.reducers.jettison_cargo_from_ship(
                                    cargo.ship_id,
                                    cargo.id,
                                    2,
                                );
                            }
                            let half = (cargo.quantity as f32 / 2.0).floor() as u16;
                            if half > 2 && ui.button(format!("-{}-", half)).clicked() {
                                let _ = ctx.reducers.jettison_cargo_from_ship(
                                    cargo.ship_id,
                                    cargo.id,
                                    half,
                                );
                            }
                            if ui.button("-All-").clicked() {
                                let _ = ctx.reducers.jettison_cargo_from_ship(
                                    cargo.ship_id,
                                    cargo.id,
                                    cargo.quantity,
                                );
                            }
                        });
                    },
                );
            }
        }
    }
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Cargo Capacity:");
        ui.add_enabled(
            total_cargo_usage != 0,
            egui::Label::new(format!("{} /", total_cargo_usage)),
        );
        ui.label(format!("{} volume", player_ship_status.max_cargo_capacity));
    });
}

fn equipment_contents(
    ui: &mut Ui,
    ctx: &DbConnection,
    state: &mut State,
    ship_type: ShipTypeDefinition,
    _player_ship: Ship,
) {
    ui.heading("Equipment");
    ui.separator();
    ui.horizontal_top(|ui| {
        ui.selectable_value(
            &mut state.current_equipment_tab,
            EquipmentSlotType::Weapon,
            "Weapon",
        );
        ui.selectable_value(
            &mut state.current_equipment_tab,
            EquipmentSlotType::Shield,
            "Shields",
        );
        ui.selectable_value(
            &mut state.current_equipment_tab,
            EquipmentSlotType::Engine,
            "Engine",
        );
        ui.selectable_value(
            &mut state.current_equipment_tab,
            EquipmentSlotType::MiningLaser,
            "Mining",
        );
        ui.selectable_value(
            &mut state.current_equipment_tab,
            EquipmentSlotType::Special,
            "Special",
        );
    });
    ui.separator();

    let max_slots = match state.current_equipment_tab {
        EquipmentSlotType::Weapon => ship_type.num_weapon_slots,
        EquipmentSlotType::Shield => ship_type.num_shield_slots,
        EquipmentSlotType::Engine => ship_type.num_engine_slots,
        EquipmentSlotType::MiningLaser => ship_type.num_mining_laser_slots,
        EquipmentSlotType::Special => ship_type.num_special_slots,
        EquipmentSlotType::CargoExpansion => todo!(),
    };
    let mut slots = 0;

    for equipment in ctx.db().ship_equipment_slot().iter() {
        if state.current_equipment_tab != equipment.slot_type {
            continue;
        }
        ui.horizontal(|ui| {
            ui.label(format!(
                "{} --- {}",
                slots + 1,
                equipment.slot_type.to_string()
            ));
            ui.add_enabled(false, egui::Label::new("---"));
            if let Some(item) = ctx.db().item_definition().id().find(&equipment.item_id) {
                ui.label(item.name);
            }
        });
        slots += 1;
    }
    for empty_slot in slots..max_slots {
        ui.add_enabled(
            false,
            egui::Label::new(format!("{} --- Empty Slot", empty_slot + 1)),
        );
    }

    ui.spacing();
    // Maybe add a button to equip an item here?
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(format!("Slots: {} / {}", slots, max_slots));
    });
}
