use macroquad::prelude::*;

#[macroquad::main("egui with macroquad")]
async fn main() {
    loop {
        clear_background(BLACK);

        // Process keys, mouse etc.

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("egui ❤ macroquad")
                .show(egui_ctx, |ui| {
                    ui.label("Test");
                });
        });

        // Draw things before egui

        egui_macroquad::draw();
        
        // Draw things after egui
        next_frame().await;
    }
}

fn 