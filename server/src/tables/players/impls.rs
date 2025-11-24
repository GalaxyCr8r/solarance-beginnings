use super::*;

//////////////////////////////////////////////////////////////
// Impls ///
//////////////////////////////////////////////////////////////

impl Player {
    pub fn get_ship_id(&self, dsl: &DSL) -> Option<u64> {
        if let Ok(window) = dsl.get_sobj_player_window_by_id(&self.get_id()) {
            Some(window.get_sobj_id().value())
        } else {
            None
        }
    }
}
