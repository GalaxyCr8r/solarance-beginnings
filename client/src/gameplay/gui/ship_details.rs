
use egui::{Context, Ui};
use macroquad::prelude::*;
use spacetimedb_sdk::Table;

use crate::{module_bindings::*, stdb::utils::get_player_ship_instance};

#[derive(PartialEq)]
enum CurrentTab {
    Ship,
    Cargo,
    Equipment
}

//#[derive(Default)]
pub struct WindowState {
    current_tab: CurrentTab, // = CurrentTab::Ship
    current_equipment_tab: EquipmentSlotType,
}

impl WindowState {
    pub fn new() -> Self {
        WindowState {
            current_tab: CurrentTab::Ship,
            current_equipment_tab: EquipmentSlotType::Weapon
        }
    }
}

pub fn window(egui_ctx: &Context, ctx: &DbConnection, state: &mut WindowState,  open: &mut bool) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window
        ::new("Ship Details")
        .open(open)
        .title_bar(true)
        .resizable(true)
        .collapsible(true)
        .movable(true)
        .vscroll(true)
        .show(egui_ctx, |ui| {
            if let Some(player_ship) = get_player_ship_instance(ctx) {
                if let Some(ship_type) = ctx.db.ship_type_definition().id().find(&player_ship.shiptype_id) {
                    ui.horizontal_top(|ui| {
                        ui.selectable_value(&mut state.current_tab, CurrentTab::Ship, "Ship");
                        ui.selectable_value(&mut state.current_tab, CurrentTab::Cargo, "Cargo");
                        ui.selectable_value(&mut state.current_tab, CurrentTab::Equipment, "Equipment");
                    });

                    match state.current_tab {
                        CurrentTab::Ship => {ship_contents(ui, ctx, state, ship_type, player_ship);},
                        CurrentTab::Cargo => {cargo_contents(ui, ctx, state, ship_type, player_ship);},
                        CurrentTab::Equipment => {equipment_contents(ui, ctx, state, ship_type, player_ship);}
                    }
                }
            }
        })
}

fn ship_contents(ui: &mut Ui, ctx: &DbConnection, _state: &mut WindowState, ship_type: ShipTypeDefinition, player_ship: ShipInstance) {
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
            ui.label(format!("Health: {}", player_ship.health));
            ui.label(format!("Shield: {}", player_ship.shields));
            ui.label(format!("Energy: {}", player_ship.energy));
            ui.label(format!("Cargo: {} / {}", player_ship.cargo_capacity, ship_type.cargo_capacity));
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.label(format!("Max Health: {}", ship_type.max_health));
            ui.label(format!("Max Shield: {}", ship_type.max_shield));
            ui.label(format!("Max Energy: {}", ship_type.max_energy));
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.label(format!("Speed: {}", ship_type.base_speed));
            ui.label(format!("Acceleration: {}", ship_type.base_acceleration));
            ui.label(format!("Turn Rate: {}", ship_type.base_turn_rate));
        });
    });
    ui.heading("Equipment Slots");
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(format!("Weapon Slots: {}/{}", 
                player_ship.get_all_equipped_of_type(ctx, EquipmentSlotType::Weapon).iter().count(), 
                ship_type.num_weapon_slots));
            ui.label(format!("Shield Slots: {}/{}", 
                player_ship.get_all_equipped_of_type(ctx, EquipmentSlotType::Shield).iter().count(), 
                ship_type.num_shield_slots));
            ui.label(format!("Engine Slots: {}/{}", 
                player_ship.get_all_equipped_of_type(ctx, EquipmentSlotType::Engine).iter().count(), 
                ship_type.num_engine_slots));
            ui.label(format!("Minig Laser Slots: {}/{}", 
                player_ship.get_all_equipped_of_type(ctx, EquipmentSlotType::MiningLaser).iter().count(), 
                ship_type.num_mining_laser_slots));
            ui.label(format!("Special Slots: {}/{}", 
                player_ship.get_all_equipped_of_type(ctx, EquipmentSlotType::Special).iter().count(), 
                ship_type.num_special_slots));
        });
    });
}

fn cargo_contents(ui: &mut Ui, ctx: &DbConnection, _state: &mut WindowState, ship_type: ShipTypeDefinition, player_ship: ShipInstance) {
    ui.heading("Cargo Bay Contents");
    ui.separator();
    let mut actual_cargo_remaining = ship_type.cargo_capacity;
    for cargo in ctx.db.ship_cargo_item().iter() {
        if cargo.ship_id == player_ship.id { // TECHNICALLY RLS should do this for us.
            if let Some(item) = ctx.db.item_definition().id().find(&cargo.item_id) {
                actual_cargo_remaining -= item.volume_per_unit * cargo.quantity;
                ui.collapsing(format!("Cargo: {} x {} @ {}u", item.name, cargo.quantity, item.volume_per_unit * cargo.quantity), |ui| {
                    ui.heading(item.name);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(item.description.unwrap_or("n/a".to_string()));
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("Volume: {} per unit, ", item.volume_per_unit));
                        ui.label(format!("{} total", item.volume_per_unit * cargo.quantity));
                    });
                    ui.separator();
                    ui.label(format!("Base Value: {} credits", item.base_value));
                });
            }
        }
    }
    ui.separator();
    ui.horizontal(|ui| {
        //let current_cargo = ship_type.cargo_capacity - player_ship.cargo_capacity; // this is still whack due to how ships are currently created

        ui.label("Cargo Capacity:");
        ui.add_enabled(actual_cargo_remaining != 0, egui::Label::new(format!("{} /", actual_cargo_remaining)));
        ui.label(format!("{} units", ship_type.cargo_capacity));
    });
}

fn equipment_contents(ui: &mut Ui, ctx: &DbConnection, state: &mut WindowState, ship_type: ShipTypeDefinition, _player_ship: ShipInstance) {
    ui.heading("Equipment");
    ui.separator();
    ui.horizontal_top(|ui| {
        ui.selectable_value(&mut state.current_equipment_tab, EquipmentSlotType::Weapon, "Weapon");
        ui.selectable_value(&mut state.current_equipment_tab, EquipmentSlotType::Shield, "Shields");
        ui.selectable_value(&mut state.current_equipment_tab, EquipmentSlotType::Engine, "Engine");
        ui.selectable_value(&mut state.current_equipment_tab, EquipmentSlotType::MiningLaser, "Mining");
        ui.selectable_value(&mut state.current_equipment_tab, EquipmentSlotType::Special, "Special");
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

    for equipment in ctx.db.ship_equipment_slot().iter() {
        if state.current_equipment_tab != equipment.slot_type {
            continue;
        }
        ui.horizontal(|ui| {
            ui.label(format!("{} --- {}", slots+1, equipment.slot_type.to_string()));
            ui.add_enabled(false, egui::Label::new("---"));
            if let Some(item) = ctx.db.item_definition().id().find(&equipment.item_id) {
                ui.label(item.name);
            }
        });
        slots += 1;
    }
    for empty_slot in slots..max_slots {
        ui.add_enabled(false, egui::Label::new(format!("{} --- Empty Slot", empty_slot+1)));
    }
    
    ui.spacing();
    // Maybe add a button to equip an item here?
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(format!("Slots: {} / {}", slots, max_slots));
    });
}