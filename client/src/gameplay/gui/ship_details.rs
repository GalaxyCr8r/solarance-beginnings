use egui::{Align2, Color32, Context, Ui};
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
    pub open: bool,
    current_tab: CurrentTab// = CurrentTab::Ship
}

impl WindowState {
    pub fn new() -> Self {
        WindowState {
            open: false,
            current_tab: CurrentTab::Ship
        }
    }
}

pub fn window(egui_ctx: &Context, ctx: &DbConnection, state: &mut WindowState) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window
        ::new("Ship Details")
        .open(&mut state.open)
        .title_bar(true)
        .resizable(true)
        .collapsible(true)
        .movable(true)
        //.anchor(Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, 0.0))
        .show(egui_ctx, |ui| {
            if let Some(player_ship) = get_player_ship_instance(ctx) {
                if let Some(ship_type) = ctx.db.ship_type_definition().id().find(&player_ship.shiptype_id) {
                    ui.horizontal_top(|ui| {
                        ui.selectable_value(&mut state.current_tab, CurrentTab::Ship, "Ship");
                        ui.selectable_value(&mut state.current_tab, CurrentTab::Cargo, "Cargo");
                        ui.selectable_value(&mut state.current_tab, CurrentTab::Equipment, "Equipment");
                    });

                    match state.current_tab {
                        CurrentTab::Ship => {ship_contents(ui, ctx, ship_type, player_ship);},
                        CurrentTab::Cargo => {cargo_contents(ui, ctx, ship_type, player_ship);},
                        CurrentTab::Equipment => {equipment_contents(ui, ctx, ship_type, player_ship);}
                    }
                }
            }
        })
}

fn ship_contents(ui: &mut Ui, _ctx: &DbConnection, ship_type: ShipTypeDefinition, player_ship: ShipInstance) {
    ui.heading("Detailed Ship Status");
    ui.label(format!("Ship Type: {}", ship_type.name));
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(format!("Health: {}", player_ship.health));
            ui.label(format!("Shield: {}", player_ship.shields));
            ui.label(format!("Energy: {}", player_ship.energy));
        });
        ui.vertical(|ui| {
            ui.label(format!("Max Health: {}", ship_type.max_health));
            ui.label(format!("Max Shield: {}", ship_type.max_shield));
            ui.label(format!("Max Energy: {}", ship_type.max_energy));
        });
    });
}

fn cargo_contents(ui: &mut Ui, ctx: &DbConnection, ship_type: ShipTypeDefinition, player_ship: ShipInstance) {
    ui.heading("Cargo Bay Contents");
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

fn equipment_contents(ui: &mut Ui, _ctx: &DbConnection, _ship_type: ShipTypeDefinition, _player_ship: ShipInstance) {
    ui.heading("Equipment/Modules");
}