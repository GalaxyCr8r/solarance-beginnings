use crate::scenes::screen::{Screen, ScreenManager};
use crate::scenes::login_screen::{LoginScreen};
use macroquad::prelude::*;

pub struct GameScreen {
}

impl GameScreen {
    pub fn new() -> GameScreen {
    return GameScreen {}
    }
}

impl Screen for GameScreen {
    fn update_screen(&self, _screen_manager: &ScreenManager) -> Option<Box<dyn Screen>> {
        clear_background(BLACK);

        let mut next_screen: Option<Box<dyn Screen>> = None;

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("egui ❤ macroquad")
                .show(egui_ctx, |ui| {
                    if ui.button("Back").clicked() {
                        info!("Clicked button!");
                        next_screen = Some(Box::new(LoginScreen::new()));
                    }
                });
        });


        return next_screen;
    }
}