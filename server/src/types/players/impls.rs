use crate::types::stellarobjects::GetStellarObjectPlayerWindowRowOptionByPlayerId;

use super::*;

//////////////////////////////////////////////////////////////
// Impls ///
//////////////////////////////////////////////////////////////

impl Player {
    pub fn get_ship_id(&self, ctx: &ReducerContext) -> Option<u64> {
        if let Some(window) = dsl(ctx).get_sobj_player_window_by_player_id(&self.player_id) {
            Some(window.sobj_id)
        } else {
            None
        }
    }
}