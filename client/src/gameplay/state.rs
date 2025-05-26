use macroquad::{camera::Camera2D, prelude::*};

use crate::module_bindings::*;
use crate::gameplay::gui::*;

pub(crate) struct GameState<'a> {
    // Game-Wide States
    pub done: bool,
    pub ctx: &'a DbConnection,

    // Display States
    pub camera: Camera2D,

    // GUI States
    pub chat_window: chat::WindowState,
    pub details_window: ship_details::WindowState,
    pub details_window_open: bool,
    pub faction_window_open: bool,
    pub assets_window_open: bool,
}


pub fn initialize<'a>(ctx: &'a DbConnection) -> GameState<'a> {
    GameState {
        done: false,
        ctx: ctx,

        camera: Camera2D::from_display_rect(Rect { x: 0.0, y: 0.0, w: screen_width(), h: screen_height() }),

        chat_window: chat::WindowState {
            ..Default::default()
        },
        details_window: ship_details::WindowState::new(),
        details_window_open: false,
        faction_window_open: false,
        assets_window_open: false,
    }
}