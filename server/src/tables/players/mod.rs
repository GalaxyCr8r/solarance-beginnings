use spacetimedb::{table, Identity, Timestamp};
use spacetimedsl::*;

use crate::tables::factions::FactionId;

use super::stellarobjects::*;

// Re-export PlayerShipController for referenced_by attributes
pub use crate::logic::ships::player_controller::PlayerShipController;

//pub mod definitions; // Definitions for initial ingested data.
pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[dsl(plural_name = players)]
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

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_dsl: &DSL) -> Result<(), String> {
    Ok(())
}
