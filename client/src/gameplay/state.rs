use macroquad::{camera::Camera2D, prelude::*};

use crate::gameplay::gui::*;
use crate::module_bindings::*;

pub struct GameState<'a> {
    // Game-Wide States
    pub done: bool,
    pub ctx: &'a DbConnection,

    // Display States
    pub camera: Camera2D,
    pub bg_camera: Camera2D,

    // GUI States
    pub chat_window: chat_widget::State,
    pub creation_window: creation_window::State,
    pub details_window: ship_details_window::State,
    pub map_window: map_window::State,

    // GUI Window Booleans
    pub assets_window_open: bool,
    pub details_window_open: bool,
    pub faction_window_open: bool,
    pub map_window_open: bool,

    // Gameplay States
    pub current_target_sobj: Option<StellarObject>,
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

        chat_window: chat_widget::State::default(),
        creation_window: creation_window::State::new(),
        details_window: ship_details_window::State::new(),
        map_window: map_window::State::new(),

        assets_window_open: false,
        details_window_open: false,
        faction_window_open: false,
        map_window_open: false,

        current_target_sobj: None,
    }
}
