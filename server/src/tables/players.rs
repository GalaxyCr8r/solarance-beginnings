use spacetimedb::{table, Identity, Timestamp};
use spacetimedsl::*;

use crate::tables::factions::FactionId;

use super::stellarobjects::*;

// Re-export PlayerShipController for referenced_by attributes
pub use crate::logic::ships::player_controller::PlayerShipController;

#[dsl(plural_name = players, method(update = true))]
#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    #[create_wrapper]
    #[referenced_by(path = crate::logic::ships::player_controller, table = player_ship_controller)]
    #[referenced_by(path = crate::tables::ships, table = ship)]
    #[referenced_by(path = crate::tables::chats, table = global_chat_message)]
    #[referenced_by(path = crate::tables::chats, table = sector_chat_message)]
    #[referenced_by(path = crate::tables::chats, table = faction_chat_message)]
    #[referenced_by(path = crate::tables::stellarobjects, table = sobj_player_window)]
    #[referenced_by(path = crate::tables::server_messages, table = server_message_recipient)]
    id: Identity,

    #[unique]
    pub username: String,
    pub credits: u64,

    pub logged_in: bool,
    pub faction_id: FactionId,

    created_at: Timestamp,
    modified_at: Timestamp,
}

impl Player {
    pub fn get_ship_id<T>(&self, dsl: &DSL<T>) -> Option<u64> {
        if let Ok(window) = dsl.get_sobj_player_window_by_id(&self.get_id()) {
            Some(window.get_sobj_id().value())
        } else {
            None
        }
    }
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init<T>(_dsl: &DSL<T>) -> Result<(), String> {
    Ok(())
}

pub fn get_username(dsl: &DSL<T>, id: Identity) -> String {
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
