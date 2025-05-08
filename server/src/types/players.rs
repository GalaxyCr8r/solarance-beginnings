use spacetimedb::{table, ReducerContext, Identity};

use super::stellarobjects::player_controlled_stellar_object;

#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,

    #[index(btree)]
    pub username: String,

}

//// Impls ///

impl Player {
    pub fn get_controlled_stellar_object(&self, ctx: &ReducerContext) -> Option<u64> {
        if let Some(player_controlled_stellar_object) = ctx.db.player_controlled_stellar_object().identity().find(&self.identity) {
            Some(player_controlled_stellar_object.controlled_sobj_id)
        } else {
            None
        }
    }
}

//// Reducers ///

