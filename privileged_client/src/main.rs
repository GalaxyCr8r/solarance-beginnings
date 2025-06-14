use macroquad::prelude::*;

use crate::scenes::login_screen::{LoginScreen};
use crate::scenes::screen::{ScreenManager};

pub mod scenes;

#[macroquad::main("egui with macroquad")]
async fn main() {

    let mut screen_manager: ScreenManager = ScreenManager { 
        screen: Box::new(LoginScreen::new()),
    };

    loop {
        clear_background(BLACK);

        // Process keys, mouse etc.
        
        let update_success = screen_manager.update_screen().ok().unwrap();
        
        // Draw things before egui

        egui_macroquad::draw();
        
        // Draw things after egui
        next_frame().await;

        if !update_success {
            break;
        }
    }
}