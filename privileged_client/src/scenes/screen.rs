
use macroquad::prelude::*;

pub trait Screen {
    fn update_screen(&self, screen_manager: &ScreenManager) -> Option<Box<dyn Screen>>;
}

pub struct ScreenManager {
    pub screen: Box<dyn Screen>,
}

impl ScreenManager {
    pub fn update_screen(&mut self) -> Result<bool, String> {

        let new_screen = self.screen.update_screen(self);
        if new_screen.is_some() {
            info!("Screen changed");
            self.screen = new_screen.unwrap();
        }

        Ok(true)
    }
}
