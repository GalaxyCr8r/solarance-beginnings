use spacetimedb::{table, Identity, ReducerContext, Timestamp};
use spacetimedsl::dsl;

use super::ships::ship_object;

#[dsl(plural_name = players)]
#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,

    #[unique]
    pub username: String,
    pub credits: u64,

    created_at: Timestamp,
    modified_at: Timestamp,
}

//// Impls ///

impl Player {
    pub fn get_ship_id(&self, ctx: &ReducerContext) -> Option<u64> {
        if let Some(player_controlled_stellar_object) = ctx.db.ship_object().player_id().filter(self.identity).last() {
            Some(player_controlled_stellar_object.sobj_id)
        } else {
            None
        }
    }
}

pub fn get_username(ctx: &ReducerContext, id:Identity) -> String {
    if let Some(player) = ctx.db.player().identity().find(id) {
        player.username
    } else {
        if ctx.sender == ctx.identity() {
            "SERVER".to_string()
        } else {
            id.to_abbreviated_hex().to_string()
        }
    }
}

//// Reducers ///

