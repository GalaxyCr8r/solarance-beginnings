use egui::{Context, FontId, RichText, Ui};
use macroquad::prelude::*;
use spacetimedb_sdk::Table;

use crate::{module_bindings::*, stdb::utils::*};

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
            draw_galaxy_map(ui, ctx);
        })
}

fn draw_galaxy_map(ui: &mut egui::Ui, ctx: &DbConnection) {
  let mut current_sector = None;
  if let Some(player_obj) = get_player_ship_object(ctx) {
    if let Some(sector) = ctx.db.sector().id().find(&player_obj.sector_id) {
      current_sector = Some(sector);
    }
  }
  ui.horizontal(|ui| {
    ui.label("Current Sector:");
    if let Some(sector) = current_sector {
      ui.label(sector.name);
    } else {
      ui.label("Unknown");
    }
    
  });

  ui.separator();

  // TODO Canvas of the galaxy

  // egui::Frame::group(ui.style())
  //   .inner_margin(0.0)
  //   .show(ui, |ui| {
  //     //
  //     let mut reset_view = false;
  //     let mut inner_rect = egui::Rect::NAN;
  //     egui::
  //   });

}