use egui::{Align2, Context, RichText, ScrollArea};
use spacetimedb_sdk::*;

use crate::{gameplay::state::GameState, module_bindings::*, stdb::utils::*};

pub fn draw(
    egui_ctx: &Context,
    game_state: &mut GameState,
) -> Option<egui::InnerResponse<Option<()>>> {
    let ctx = game_state.ctx;

    egui::Window::new("Minimap")
        .title_bar(true)
        .resizable(true)
        .collapsible(true)
        .movable(false)
        .anchor(Align2::RIGHT_BOTTOM, egui::Vec2::new(0.0, 0.0))
        .default_size(egui::Vec2::new(300.0, 400.0))
        .show(egui_ctx, |ui| {
            if let Some(player_ship) = get_player_ship(ctx) {
                if let Some(sector) = ctx.db().sector().id().find(&player_ship.sector_id) {
                    ui.horizontal(|ui| {
                        ui.heading("Current Sector:");
                        ui.heading(RichText::new(&sector.name).strong());
                    });
                    ui.label("Sector Faction: n/a");
                    ui.separator();

                    if let Some(mut controller) =
                        ctx.db().player_ship_controller().id().find(&ctx.identity())
                    {
                        let _ = list_sector_objects(
                            game_state,
                            ctx,
                            ui,
                            sector,
                            player_ship,
                            &mut controller,
                        );
                    }
                }
            } else {
                ui.label("No ship detected");
            }
        })
}

fn list_sector_objects(
    game_state: &mut GameState<'_>,
    ctx: &DbConnection,
    ui: &mut egui::Ui,
    _sector: Sector,
    player_ship: Ship,
    controller: &mut PlayerShipController,
) -> std::result::Result<(), String> {
    // Get player position for distance calculations
    let player_sobj_id = get_player_sobj_id(ctx).ok_or("Failed to find player".to_string())?;
    let player_transform = get_transform(ctx, player_sobj_id)?;
    let player_pos = player_transform.to_vec2();

    // Collect all stellar objects in the current sector with distances
    let mut stellar_objects_with_distance: Vec<(StellarObject, f32)> = Vec::new();

    for sobj in ctx.db().stellar_object().iter() {
        // Skip player's own ship and objects not in current sector
        if sobj.id == player_sobj_id || sobj.sector_id != player_ship.sector_id {
            continue;
        }

        if let Ok(transform) = get_transform(ctx, sobj.id) {
            let distance = player_pos.distance(transform.to_vec2());
            stellar_objects_with_distance.push((sobj, distance));
        }
    }

    // Sort by distance (ascending)
    stellar_objects_with_distance
        .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(200.0)
        .show(ui, |ui| {
            if stellar_objects_with_distance.is_empty() {
                ui.label("No objects detected in sector");
            } else {
                for (sobj, distance) in stellar_objects_with_distance {
                    let selected = controller
                        .targetted_sobj_id
                        .map_or(false, |sobj_id| sobj_id == sobj.id);

                    ui.horizontal(|ui| {
                        // Target button
                        if ui.button(if selected { "X" } else { "O" }).clicked() {
                            // Toggle target if already selected
                            if controller.targetted_sobj_id.is_some()
                                && controller.targetted_sobj_id.unwrap() == sobj.id
                            {
                                controller.targetted_sobj_id = None;
                                game_state.current_target_sobj = None;
                            } else {
                                controller.targetted_sobj_id = Some(sobj.id);
                                game_state.current_target_sobj = Some(sobj.clone());
                            }
                            let _ = ctx.reducers.update_player_controller(controller.clone());
                        }

                        // Object type
                        let type_str = match sobj.kind {
                            StellarObjectKinds::Ship => {
                                if let Some(ship) =
                                    ctx.db().ship().iter().find(|s| s.sobj_id == sobj.id)
                                {
                                    format!(
                                        "[{}] {}",
                                        get_faction_shortname(ctx, &ship.faction_id),
                                        get_username(ctx, &ship.player_id)
                                    )
                                } else {
                                    "Unknown Ship".to_string()
                                }
                            }
                            StellarObjectKinds::Asteroid => "Asteroid".to_string(),
                            StellarObjectKinds::CargoCrate => "Cargo Crate".to_string(),
                            StellarObjectKinds::Station => {
                                if let Some(station) = ctx.db().station().sobj_id().find(&sobj.id) {
                                    format!(
                                        "[{}] {}",
                                        get_faction_shortname(ctx, &station.owner_faction_id),
                                        station.name
                                    )
                                } else {
                                    "Unknown Station".to_string()
                                }
                            }
                            StellarObjectKinds::JumpGate => {
                                if let Some(jump_gate) = ctx.db().jump_gate().id().find(&sobj.id) {
                                    if let Some(sector) =
                                        ctx.db().sector().id().find(&jump_gate.target_sector_id)
                                    {
                                        format!("Jump Gate to {}", sector.name)
                                    } else {
                                        "Unknown Jump Gate".to_string()
                                    }
                                } else {
                                    "Unknown Jump Gate".to_string()
                                }
                            }
                        }
                        .to_string();

                        let text = RichText::new(format!("{}: {:.0}m", type_str, distance));
                        ui.label(if selected { text.strong() } else { text });
                    });
                }
            }
        });

    Ok(())
}
