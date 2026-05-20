use egui::{Color32, Context, ProgressBar, RichText};
use spacetimedb_sdk::*;

use crate::{server::bindings::*, stdb::utils::*};

/// Must match `server::logic::stations::contribution::CONTRIBUTE_RANGE_PX`.
/// The server rejects deposits past this distance; we mirror it here so the
/// UI can grey the deposit buttons before the player tries.
const CONTRIBUTE_RANGE_PX: f32 = 300.0;

pub struct State {}

impl State {
    pub fn new() -> Self {
        State {}
    }
}

pub fn draw(
    egui_ctx: &Context,
    ctx: &DbConnection,
    _state: &mut State,
    open: &mut bool,
) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window::new("Construction")
        .open(open)
        .title_bar(true)
        .resizable(true)
        .collapsible(true)
        .movable(true)
        .vscroll(true)
        .default_width(360.0)
        .default_height(420.0)
        .show(egui_ctx, |ui| match nearest_construction_site(ctx) {
            None => {
                ui.label("No construction site in this sector.");
            }
            Some((station, under_construction)) => {
                draw_site(ui, ctx, &station, &under_construction);
            }
        })
}

/// Pick the construction site closest to the player's predicted position
/// (within the player's current sector). Returns the `Station` row and its
/// `StationUnderConstruction` row.
fn nearest_construction_site(ctx: &DbConnection) -> Option<(Station, StationUnderConstruction)> {
    let player_ship = get_player_ship(ctx)?;
    let player_pos = get_player_pose(ctx).map(|p| p.pos);

    let mut best: Option<(Station, StationUnderConstruction, f32)> = None;
    for site in ctx.db().station_under_construction().iter() {
        let station = match ctx.db().station().id().find(&site.id) {
            Some(s) => s,
            None => continue,
        };
        if station.sector_id != player_ship.sector_id {
            continue;
        }
        let dist_sq = player_pos
            .map(|pp| {
                let dx = station.position.x - pp.x;
                let dy = station.position.y - pp.y;
                dx * dx + dy * dy
            })
            .unwrap_or(0.0);
        if best.as_ref().map_or(true, |(_, _, d)| dist_sq < *d) {
            best = Some((station, site, dist_sq));
        }
    }
    best.map(|(s, u, _)| (s, u))
}

fn draw_site(
    ui: &mut egui::Ui,
    ctx: &DbConnection,
    station: &Station,
    under_construction: &StationUnderConstruction,
) {
    ui.heading(station_display_name(ctx, station));
    ui.separator();

    if under_construction.is_operational {
        ui.add_space(8.0);
        ui.label(
            RichText::new("Construction Complete!")
                .heading()
                .color(Color32::from_rgb(120, 220, 120)),
        );
        ui.label("This station is now operational.");
        return;
    }

    let pct = under_construction.construction_progress_percentage.clamp(0.0, 100.0);
    ui.add(
        ProgressBar::new(pct / 100.0)
            .text(format!("{:.1}%", pct))
            .desired_width(ui.available_width()),
    );

    ui.add_space(8.0);
    ui.heading("Required Resources");
    ui.separator();

    let mut requirements: Vec<ConstructionRequirement> = ctx
        .db()
        .construction_requirement()
        .iter()
        .filter(|r| r.station_id == station.id)
        .collect();
    requirements.sort_by_key(|r| r.resource_item_id);

    if requirements.is_empty() {
        ui.label("(no resource requirements defined)");
    } else {
        for req in &requirements {
            let contributed: u32 = ctx
                .db()
                .construction_contribution_log()
                .iter()
                .filter(|c| c.station_id == station.id && c.item_id == req.resource_item_id)
                .map(|c| c.quantity)
                .sum();
            let name = ctx
                .db()
                .item_definition()
                .id()
                .find(&req.resource_item_id)
                .map(|i| i.name)
                .unwrap_or_else(|| format!("Item #{}", req.resource_item_id));
            let fill_ratio =
                (contributed as f32 / req.quantity_required.max(1) as f32).clamp(0.0, 1.0);
            ui.label(format!(
                "{}: {} / {}",
                name, contributed, req.quantity_required
            ));
            ui.add(ProgressBar::new(fill_ratio).desired_width(ui.available_width()));
        }
    }

    ui.add_space(8.0);
    ui.heading("Deposit From Cargo");
    ui.separator();

    let player_ship = match get_player_ship(ctx) {
        Some(s) => s,
        None => return,
    };

    let distance = get_player_pose(ctx).map(|p| {
        let dx = station.position.x - p.pos.x;
        let dy = station.position.y - p.pos.y;
        (dx * dx + dy * dy).sqrt()
    });
    let in_range = distance.map_or(false, |d| d <= CONTRIBUTE_RANGE_PX);

    if !in_range {
        let dist_text = distance
            .map(|d| format!("{:.0}px", d))
            .unwrap_or_else(|| "unknown".to_string());
        ui.label(
            RichText::new(format!(
                "Too far to contribute — move within {:.0}px (currently {}).",
                CONTRIBUTE_RANGE_PX, dist_text
            ))
            .color(Color32::from_rgb(230, 190, 80)),
        );
    }

    let useful_items: std::collections::HashSet<u32> =
        requirements.iter().map(|r| r.resource_item_id).collect();

    let mut shown_any = false;
    for cargo in ctx.db().ship_cargo_item().iter() {
        if cargo.ship_id != player_ship.id {
            continue;
        }
        if !useful_items.contains(&cargo.item_id) {
            continue;
        }
        let item_def = match ctx.db().item_definition().id().find(&cargo.item_id) {
            Some(i) => i,
            None => continue,
        };
        shown_any = true;
        ui.horizontal(|ui| {
            ui.label(format!("{}x {}", cargo.quantity, item_def.name));
            deposit_buttons(ui, ctx, station.id, cargo.item_id, cargo.quantity, in_range);
        });
    }

    if !shown_any {
        ui.label("No usable cargo on board.");
    }
}

fn deposit_buttons(
    ui: &mut egui::Ui,
    ctx: &DbConnection,
    station_id: u64,
    item_id: u32,
    cargo_qty: u16,
    enabled: bool,
) {
    let station_id = StationId { value: station_id };
    let item_id = ItemDefinitionId { value: item_id };

    let deposit = |qty: u32| {
        let _ = ctx
            .reducers
            .contribute_to_station(station_id.clone(), item_id.clone(), qty);
    };

    if ui.add_enabled(enabled, egui::Button::new("+1")).clicked() {
        deposit(1);
    }
    if cargo_qty >= 10
        && ui.add_enabled(enabled, egui::Button::new("+10")).clicked()
    {
        deposit(10);
    }
    if cargo_qty >= 100
        && ui.add_enabled(enabled, egui::Button::new("+100")).clicked()
    {
        deposit(100);
    }
    if ui.add_enabled(enabled, egui::Button::new("All")).clicked() {
        deposit(cargo_qty as u32);
    }
}
