use spacetimedb::{table, Identity, ReducerContext, Timestamp};
use spacetimedsl::{dsl, Wrapper};

use crate::types::factions::FactionId;

use super::{common::CurrentAction, ships::*, stellarobjects::*};

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
    #[referenced_by(path = crate::types::players, table = player_ship_controller)]
    #[referenced_by(path = crate::types::ships, table = ship)]
    #[referenced_by(path = crate::types::chats, table = global_chat_message)]
    #[referenced_by(path = crate::types::chats, table = sector_chat_message)]
    #[referenced_by(path = crate::types::chats, table = faction_chat_message)]
    #[referenced_by(path = crate::types::stellarobjects, table = sobj_player_window)]
    #[referenced_by(path = crate::types::server_messages, table = server_message_recipient)]
    id: Identity,

    #[unique]
    pub username: String,
    pub credits: u64,

    pub logged_in: bool,
    pub faction_id: FactionId,

    created_at: Timestamp,
    modified_at: Timestamp,
}

#[dsl(plural_name = player_ship_controllers)]
#[table(name = player_ship_controller, public)]
pub struct PlayerShipController {
    #[primary_key]
    #[use_wrapper(path = PlayerId)]
    #[foreign_key(path = crate::types::players, table = player, column = id, on_delete = Delete)]
    id: Identity,

    #[index(btree)]
    #[use_wrapper(path = StellarObjectId)]
    #[foreign_key(path = crate::types::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    pub stellar_object_id: u64,

    // Movement
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    /// Currently selected Autopilot Action
    pub current_action: CurrentAction,

    // Equipment
    pub activate_jump_drive: bool,
    pub tractor_beam_on: bool,
    pub mining_laser_on: bool,
    pub cargo_bay_open: bool,

    // Actions
    pub dock: bool,
    pub undock: bool,
    pub shield_boost: bool,
    pub fire_weapons: bool,
    pub fire_missle: bool,

    // Misc
    /// FK to StellarObject
    pub targetted_sobj_id: Option<u64>,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {
    Ok(())
}
