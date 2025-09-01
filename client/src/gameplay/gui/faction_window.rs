use egui::{Color32, Context, RichText};
use spacetimedb_sdk::*;

use crate::module_bindings::*;

#[derive(PartialEq)]
enum CurrentTab {
    Player,
    Faction,
    Members,
    Relations,
}

pub struct State {
    current_tab: CurrentTab,
}

impl State {
    pub fn new() -> Self {
        State {
            current_tab: CurrentTab::Player,
        }
    }
}

pub fn draw(
    egui_ctx: &Context,
    ctx: &DbConnection,
    state: &mut State,
    open: &mut bool,
) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window::new("Faction Information")
        .open(open)
        .title_bar(true)
        .resizable(true)
        .collapsible(true)
        .movable(true)
        .vscroll(true)
        .default_width(400.0)
        .default_height(500.0)
        .show(egui_ctx, |ui| {
            // Tab selection
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.current_tab, CurrentTab::Player, "Player");
                ui.selectable_value(&mut state.current_tab, CurrentTab::Faction, "Faction");
                ui.selectable_value(&mut state.current_tab, CurrentTab::Members, "Members");
                ui.selectable_value(&mut state.current_tab, CurrentTab::Relations, "Relations");
            });

            ui.separator();

            match state.current_tab {
                CurrentTab::Player => draw_player_tab(ui, ctx),
                CurrentTab::Faction => draw_faction_tab(ui, ctx),
                CurrentTab::Members => draw_members_tab(ui, ctx),
                CurrentTab::Relations => draw_relations_tab(ui, ctx),
            }
        })
}

fn draw_player_tab(ui: &mut egui::Ui, ctx: &DbConnection) {
    ui.heading("Player Information");
    ui.separator();

    if let Some(player) = ctx.db().player().id().find(&ctx.identity()) {
        ui.label(format!("Username: {}", player.username));
        ui.label(format!("Credits: {}", player.credits));
        ui.label(format!(
            "Status: {}",
            if player.logged_in {
                "Online"
            } else {
                "Offline"
            }
        ));

        ui.separator();

        if let Some(faction_id) = &player.faction_id {
            if let Some(faction) = ctx.db().faction().id().find(&faction_id.value) {
                ui.label(format!("Current Faction: {}", faction.name));
                ui.label(format!("Faction Tier: {:?}", faction.tier));
            } else {
                ui.label("Current Faction: Unknown");
            }
        } else {
            ui.label("Current Faction: None (Factionless)");
        }
    } else {
        ui.label("Player information not available");
    }
}

fn draw_faction_tab(ui: &mut egui::Ui, ctx: &DbConnection) {
    ui.heading("Faction Hierarchy");
    ui.separator();

    // Show player's current faction first if they have one
    if let Some(player) = ctx.db().player().id().find(&ctx.identity()) {
        if let Some(faction_id) = &player.faction_id {
            if let Some(faction) = ctx.db().faction().id().find(&faction_id.value) {
                ui.label(
                    RichText::new(format!(
                        "Your Faction: {} ({})",
                        faction.name, faction.short_name
                    ))
                    .size(16.0)
                    .strong()
                    .color(Color32::LIGHT_BLUE),
                );
                ui.separator();
            }
        } else {
            ui.label(
                RichText::new("You are Factionless")
                    .size(16.0)
                    .strong()
                    .color(Color32::GRAY),
            );
            ui.separator();
        }
    }

    ui.label("All Factions:");
    ui.separator();

    // Get all factions and organize them by parent-child relationships
    let all_factions: Vec<_> = ctx.db().faction().iter().collect();

    // Find root factions (those without parents)
    let root_factions: Vec<_> = all_factions
        .iter()
        .filter(|f| f.parent_id.is_none())
        .collect();

    // Draw the faction tree
    for root_faction in root_factions {
        draw_faction_tree_node(ui, ctx, root_faction, &all_factions, 0);
    }
}

fn draw_faction_tree_node(
    ui: &mut egui::Ui,
    ctx: &DbConnection,
    faction: &Faction,
    all_factions: &[Faction],
    depth: usize,
) {
    let indent = "  ".repeat(depth);

    // Find child factions
    let children: Vec<_> = all_factions
        .iter()
        .filter(|f| f.parent_id.as_ref().map(|p| p.value) == Some(faction.id))
        .collect();

    // Create collapsing header for factions with children, or simple label for leaf factions
    if !children.is_empty() {
        let header_text = format!(
            "{}{} ({}) - {:?}",
            indent, faction.name, faction.short_name, faction.tier
        );

        ui.collapsing(header_text, |ui| {
            // Show faction details
            draw_faction_details(ui, ctx, faction);

            ui.separator();
            ui.label("Sub-factions:");

            // Draw children
            for child in children {
                draw_faction_tree_node(ui, ctx, child, all_factions, depth + 1);
            }
        });
    } else {
        // Leaf faction - show as expandable for details
        let header_text = format!(
            "{}{} ({}) - {:?}",
            indent, faction.name, faction.short_name, faction.tier
        );

        ui.collapsing(header_text, |ui| {
            draw_faction_details(ui, ctx, faction);
        });
    }
}

fn draw_faction_details(ui: &mut egui::Ui, ctx: &DbConnection, faction: &Faction) {
    ui.label(format!("Short Name: {}", faction.short_name));
    ui.label(format!("Tier: {:?}", faction.tier));
    ui.label(format!(
        "Joinable: {}",
        if faction.joinable { "Yes" } else { "No" }
    ));

    if let Some(parent_id) = &faction.parent_id {
        if let Some(parent) = ctx.db().faction().id().find(&parent_id.value) {
            ui.label(format!("Parent: {} ({})", parent.name, parent.short_name));
        }
    }

    if let Some(capital_id) = faction.capital_station_id {
        ui.label(format!("Capital Station ID: {}", capital_id));
    }

    ui.separator();
    ui.label("Description:");
    ui.label(&faction.description);

    // Show member count
    let member_count = ctx
        .db()
        .player()
        .iter()
        .filter(|p| p.faction_id.as_ref().map(|f| f.value) == Some(faction.id))
        .count();

    ui.label(format!("Members: {}", member_count));
}

fn draw_members_tab(ui: &mut egui::Ui, ctx: &DbConnection) {
    ui.heading("Faction Members");
    ui.separator();

    if let Some(player) = ctx.db().player().id().find(&ctx.identity()) {
        if let Some(faction_id) = &player.faction_id {
            let members: Vec<_> = ctx
                .db()
                .player()
                .iter()
                .filter(|p| p.faction_id.as_ref().map(|f| f.value) == Some(faction_id.value))
                .collect();

            ui.label(format!("Total Members: {}", members.len()));
            ui.separator();

            for member in members {
                ui.horizontal(|ui| {
                    let status_color = if member.logged_in {
                        Color32::GREEN
                    } else {
                        Color32::GRAY
                    };

                    ui.colored_label(status_color, "●");
                    ui.label(&member.username);
                    ui.label(format!("Credits: {}", member.credits));
                });
            }
        } else {
            ui.label("You must be in a faction to view members");
        }
    }
}

fn draw_relations_tab(ui: &mut egui::Ui, ctx: &DbConnection) {
    ui.heading("Faction Relations");
    ui.separator();

    if let Some(player) = ctx.db().player().id().find(&ctx.identity()) {
        if let Some(faction_id) = &player.faction_id {
            ui.label("Relations with other factions:");
            ui.separator();

            for faction in ctx.db().faction().iter() {
                if faction.id != faction_id.value {
                    ui.horizontal(|ui| {
                        ui.label(&faction.name);

                        // Find standing between current faction and this faction
                        if let Some(standing) = ctx.db().faction_standing().iter().find(|s| {
                            s.faction_one_id == faction_id.value && s.faction_two_id == faction.id
                        }) {
                            let (color, status) = get_reputation_display(standing.reputation_score);
                            ui.colored_label(color, status);
                            ui.label(format!("({})", standing.reputation_score));
                        } else {
                            ui.colored_label(Color32::GRAY, "Unknown");
                        }
                    });
                }
            }
        } else {
            ui.label("You must be in a faction to view relations");
            ui.separator();
            ui.label("General faction standings:");

            // Show some general faction relations for factionless players
            for standing in ctx.db().faction_standing().iter().take(10) {
                if let (Some(faction_one), Some(faction_two)) = (
                    ctx.db().faction().id().find(&standing.faction_one_id),
                    ctx.db().faction().id().find(&standing.faction_two_id),
                ) {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} ↔ {}", faction_one.name, faction_two.name));
                        let (color, status) = get_reputation_display(standing.reputation_score);
                        ui.colored_label(color, status);
                    });
                }
            }
        }
    }
}

fn get_reputation_display(reputation: i32) -> (Color32, &'static str) {
    match reputation {
        r if r >= 75 => (Color32::LIGHT_BLUE, "Allied"),
        r if r >= 25 => (Color32::GREEN, "Friendly"),
        r if r >= -25 => (Color32::YELLOW, "Neutral"),
        r if r >= -75 => (Color32::ORANGE, "Disliked"),
        _ => (Color32::RED, "Hostile"),
    }
}
