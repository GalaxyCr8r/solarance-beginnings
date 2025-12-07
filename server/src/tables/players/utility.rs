use spacetimedb::*;
use spacetimedsl::*;

use super::*;

pub fn get_username(dsl: &DSL, id: Identity) -> String {
    if let Some(player) = dsl.get_player_by_id(&PlayerId::new(id)).ok() {
        player.username
    } else {
        if dsl.ctx().sender == dsl.ctx().identity() {
            "SERVER".to_string()
        } else {
            id.to_abbreviated_hex().to_string()
        }
    }
}
