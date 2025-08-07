use super::*;

//////////////////////////////////////////////////////////////
// Impls ///
//////////////////////////////////////////////////////////////

impl Player {
    pub fn get_ship_id(&self, ctx: &ReducerContext) -> Option<u64> {
        if let Ok(window) = dsl(ctx).get_sobj_player_window_by_id(&self.get_id()) {
            Some(window.get_sobj_id().value())
        } else {
            None
        }
    }
}
