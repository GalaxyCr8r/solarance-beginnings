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

pub fn get_username(ctx: &ReducerContext, id:Identity) -> String {
    if let Some(player) = ctx.db.player().identity().find(id) {
        player.username
    } else {
        id.to_abbreviated_hex().to_string()
    }
}

//// Reducers ///

