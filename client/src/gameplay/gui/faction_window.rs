use egui::{Color32, Context, RichText};
use spacetimedb_sdk::*;

use crate::{module_bindings::*, stdb::utils::*};

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
    ui.heading("Faction Details");
    ui.separator();

    if let Some(player) = ctx.db().player().id().find(&ctx.identity()) {
        if let Some(faction_id) = &player.faction_id {
            if let Some(faction) = ctx.db().faction().id().find(&faction_id.value) {
                ui.label(RichText::new(&faction.name).size(18.0).strong());
                ui.separator();

                ui.label(format!("Tier: {:?}", faction.tier));
                ui.label(format!(
                    "Joinable: {}",
                    if faction.joinable { "Yes" } else { "No" }
                ));

                ui.separator();
                ui.label("Description:");
                ui.label(&faction.description);

                if let Some(capital_id) = faction.capital_station_id {
                    ui.separator();
                    ui.label(format!("Capital Station ID: {}", capital_id));
                }
            } else {
                ui.label("Faction details not available");
            }
        } else {
            ui.label("You are not a member of any faction");
            ui.separator();
            ui.label("Available factions to join:");

            for faction in ctx.db().faction().iter() {
                if faction.joinable && faction.id != 0 {
                    // Exclude factionless
                    ui.horizontal(|ui| {
                        ui.label(&faction.name);
                        ui.label(format!("({:?})", faction.tier));
                    });
                }
            }
        }
    }
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
