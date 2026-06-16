//! Galaxy Creator entry point. A thin macroquad shell: every frame we clear the
//! screen and hand control to [`AdminApp::draw`], which renders either the
//! connection dialog or the admin panels via egui.

use macroquad::prelude::*;

use solarance_galaxy_creator::app::AdminApp;

fn window_conf() -> Conf {
    Conf {
        window_title: "Solarance — Galaxy Creator (Admin)".to_owned(),
        window_width: 1280,
        window_height: 820,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = AdminApp::new();

    loop {
        app.draw();
        next_frame().await;
    }
}
