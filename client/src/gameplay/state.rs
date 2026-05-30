use macroquad::{camera::Camera2D, prelude::*};
use std::collections::HashMap;

use crate::gameplay::gui::*;
use crate::server::bindings::{self, DbConnection, VisualEffectType};

#[derive(Debug, Clone)]
pub struct FiringEffect {
    pub start_time: f64,
    pub duration: f64,
    pub source_pos: bindings::Vec2,
    pub target_pos: bindings::Vec2,
    pub effect_type: VisualEffectType,
}

pub struct GameState<'a> {
    // Game-Wide States
    pub done: bool,
    pub ctx: &'a DbConnection,

    // Display States
    pub camera: Camera2D,
    pub bg_camera: Camera2D,

    // GUI States
    pub assets_window: assets_window::State,
    pub chat_window: chat_widget::State,
    pub construction_window: construction_window::State,
    pub creation_window: creation_window::State,
    pub details_window: ship_details_window::State,
    pub faction_window: faction_window::State,
    pub map_window: map_window::State,
    pub welcome_back: welcome_back_widget::State,

    pub out_of_play_screen: out_of_play_screen::State,

    // GUI Window Booleans
    pub assets_window_open: bool,
    pub construction_window_open: bool,
    pub details_window_open: bool,
    pub faction_window_open: bool,
    pub map_window_open: bool,

    // Gameplay States
    // Cache the target's *id*, never the row — the `StellarObject` row is
    // evicted on sector jump / undock and a cached copy goes stale (#123).
    // Read it back through `stdb::utils::get_current_target`, which re-queries
    // fresh and clears this field when the row is gone.
    pub current_target_sobj_id: Option<u64>,
    pub combat_mode: bool,
    pub mining_active: bool,
    pub movement_flags: (bool, bool, bool, bool), // (forward, backward, left, right)

    // Visual Effects
    pub firing_effects: HashMap<u64, FiringEffect>,
}

pub fn initialize<'a>(ctx: &'a DbConnection) -> GameState<'a> {
    GameState {
        done: false,
        ctx: ctx,

        camera: Camera2D::from_display_rect(Rect {
            x: 0.0,
            y: 0.0,
            w: screen_width(),
            h: screen_height(),
        }),
        bg_camera: Camera2D::from_display_rect(Rect {
            x: 0.0,
            y: 0.0,
            w: screen_width(),
            h: screen_height(),
        }),

        assets_window: assets_window::State::new(),
        chat_window: chat_widget::State::default(),
        construction_window: construction_window::State::new(),
        creation_window: creation_window::State::new(),
        details_window: ship_details_window::State::new(),
        faction_window: faction_window::State::new(),
        map_window: map_window::State::new(),
        welcome_back: welcome_back_widget::State::new(),

        out_of_play_screen: out_of_play_screen::State::new(),

        assets_window_open: false,
        construction_window_open: false,
        details_window_open: false,
        faction_window_open: false,
        map_window_open: false,

        current_target_sobj_id: None,
        combat_mode: false,
        mining_active: false,
        movement_flags: (false, false, false, false),

        firing_effects: HashMap::new(),
    }
}
