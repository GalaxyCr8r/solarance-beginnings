use macroquad::{camera::Camera2D, prelude::*};

use crate::module_bindings::*;
use crate::gameplay::gui::chat::ChatWindowState;

pub(crate) struct GameState<'a> {
    // Game-Wide States
    pub done: bool,
    pub ctx: &'a DbConnection,

    // Display States
    pub camera: Camera2D,

    // GUI States
    pub chat_window: ChatWindowState
}


pub fn initialize<'a>(ctx: &'a DbConnection) -> GameState<'a> {
    GameState {
        done: false,
        ctx: ctx,

        camera: Camera2D::from_display_rect(Rect { x: 0.0, y: 0.0, w: screen_width(), h: screen_height() }),

        chat_window: ChatWindowState {
            ..Default::default()
        }
    }
}