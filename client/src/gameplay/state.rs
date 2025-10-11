use macroquad::{camera::Camera2D, prelude::*};
use std::collections::HashMap;

use crate::gameplay::gui::*;
use crate::module_bindings::{self, DbConnection, StellarObject, VisualEffectType};

#[derive(Debug, Clone)]
pub struct FiringEffect {
    pub start_time: f64,
    pub duration: f64,
    pub source_pos: module_bindings::Vec2,
    pub target_pos: module_bindings::Vec2,
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
    pub creation_window: creation_window::State,
    pub details_window: ship_details_window::State,
    pub faction_window: faction_window::State,
    pub map_window: map_window::State,

    pub out_of_play_screen: out_of_play_screen::State,

    // GUI Window Booleans
    pub assets_window_open: bool,
    pub details_window_open: bool,
    pub faction_window_open: bool,
    pub map_window_open: bool,

    // Gameplay States
    pub current_target_sobj: Option<StellarObject>,
    pub combat_mode: bool,

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
        creation_window: creation_window::State::new(),
        details_window: ship_details_window::State::new(),
        faction_window: faction_window::State::new(),
        map_window: map_window::State::new(),

        out_of_play_screen: out_of_play_screen::State::new(),

        assets_window_open: false,
        details_window_open: false,
        faction_window_open: false,
        map_window_open: false,

        current_target_sobj: None,
        combat_mode: false,

        firing_effects: HashMap::new(),
    }
}
