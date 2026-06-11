pub mod asset_utils;
pub mod assets_window;
pub mod chat_widget;
pub mod construction_window;
pub mod creation_window;
pub mod debug_widget;
pub mod faction_window;
pub mod map_window;
pub mod menu_bar_widget;
pub mod minimap_widget;
pub mod out_of_play_screen;
pub mod ship_details_window;
pub mod status_widget;
pub mod welcome_back_widget;

// Factions
pub const FACTION_FACTIONLESS: u32 = 0;
pub const FACTION_LRAK_COMBINE: u32 = 1;
pub const FACTION_INDEPENDENT_WORLDS_ALLIANCE: u32 = 2;
pub const FACTION_FREE_TRADE_UNION: u32 = 3;
pub const FACTION_REDIAR_FEDERATION: u32 = 4;
pub const FACTION_VANCELLAN: u32 = 5;

/// Display color for a faction (Lrak red, Rediar blue — CONTEXT.md §3).
/// Faction colors are a design commitment but are not stored in the Faction
/// table, so the canonical id → color mapping lives here for every window
/// that tints by faction (creation picker, construction sites, map, …).
/// Unknown / colorless factions render light gray.
pub fn faction_color(faction_id: u32) -> egui::Color32 {
    match faction_id {
        FACTION_LRAK_COMBINE => egui::Color32::from_rgb(230, 90, 90), // Lrak Combine — red
        FACTION_INDEPENDENT_WORLDS_ALLIANCE => egui::Color32::from_rgb(90, 230, 90), // IWA - green
        FACTION_FREE_TRADE_UNION => egui::Color32::ORANGE,
        FACTION_REDIAR_FEDERATION => egui::Color32::from_rgb(100, 150, 255), // Rediar Federation — blue
        FACTION_VANCELLAN => egui::Color32::MAGENTA,
        _ => egui::Color32::LIGHT_GRAY,
    }
}
