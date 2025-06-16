use egui::{Context, FontId, RichText};
use macroquad::prelude::*;

use crate::{module_bindings::*, stdb::utils::get_player_ship_instance};

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

pub fn draw(egui_ctx: &Context, ctx: &DbConnection, state: &mut State,  open: &mut bool) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window
        ::new("Window")
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
                        ui.selectable_value(&mut state.current_tab, CurrentTab::Ship,
                            RichText::new("Ship").font(FontId::proportional(20.0)));
                        ui.selectable_value(&mut state.current_tab, CurrentTab::Cargo,
                            RichText::new("Cargo").font(FontId::proportional(20.0)));
                    });

                    ui.separator();

                    match state.current_tab {
                        CurrentTab::Ship => { todo!() },
                        CurrentTab::Cargo => { todo!() },
                    }
                }
            }
        })
}