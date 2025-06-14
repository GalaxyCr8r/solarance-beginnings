
use crate::scenes::{game_screen::GameScreen, screen::{Screen, ScreenManager}};
use std::thread::JoinHandle;
use macroquad::prelude::*;

pub struct LoginScreen {
    pub client_token_thread: Option<JoinHandle<Result<String, String>>>,
}

impl LoginScreen {
    pub fn new() -> LoginScreen {
    return LoginScreen {       
            client_token_thread: None,
        }
    }
}

impl Screen for LoginScreen {
    fn update_screen(&self, _screen_manager: &ScreenManager) -> Option<Box<dyn Screen>> {
        clear_background(DARKGRAY);
        
        let mut next_screen: Option<Box<dyn Screen>> = None;

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("egui ❤ macroquad")
                .show(egui_ctx, |ui| {
                    if ui.button("Play").clicked() {
                        info!("Clicked button!");
                        next_screen = Some(Box::new(GameScreen::new()));
                    }
                });
        });

        return next_screen;
    }
}