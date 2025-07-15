use log::info;
use spacetimedb::{ table, Identity, ReducerContext, Timestamp };
use spacetimedsl::*;

use crate::types::{ players::{ utility::get_username, PlayerId }, sectors::SectorId, ships::* };

pub mod definitions; // Definitions for initial ingested data.
pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[dsl(plural_name = global_chat_messages)]
#[table(name = global_chat_message, public)]
pub struct GlobalChatMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[use_wrapper(path = crate::players::PlayerId)]
    /// FK to Player
    pub player_id: Identity,

    pub message: String,

    created_at: Timestamp,
}

#[dsl(plural_name = sector_chat_messages)]
#[table(name = sector_chat_message, public)]
pub struct SectorChatMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[use_wrapper(path = crate::players::PlayerId)]
    /// FK to Player
    pub player_id: Identity,

    #[index(btree)] // To find asteroids in a specific sector
    #[use_wrapper(path = crate::types::sectors::SectorId)]
    /// FK to Sector.id
    pub sector_id: u64,

    pub message: String,

    created_at: Timestamp,
}

#[dsl(plural_name = faction_chat_messages)]
#[table(name = faction_chat_message, public)]
pub struct FactionChatMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[use_wrapper(path = crate::players::PlayerId)]
    /// FK to Player
    pub player_id: Identity,

    #[index(btree)]
    #[use_wrapper(path = crate::types::factions::FactionId)]
    /// FK to FactionDefinition
    pub faction_id: u32,

    pub message: String,

    created_at: Timestamp,
}

//////////////////////////////////////////////////////////////
// Impls ///
//////////////////////////////////////////////////////////////

impl GlobalChatMessage {
    //
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {
    Ok(())
}

//////////////////////////////////////////////////////////////
// Reducers ///
//////////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn send_global_chat(ctx: &ReducerContext, chat_message: String) -> Result<(), String> {
    let dsl = dsl(ctx);

    // If ctx.sender is a valid, unbanned, unmuted player
    info!("GlobalChat [{}]: {}", get_username(ctx, ctx.sender), chat_message);

    dsl.create_global_chat_message(PlayerId::new(ctx.sender), &chat_message)?;
    Ok(())
}

#[spacetimedb::reducer]
pub fn send_sector_chat(
    ctx: &ReducerContext,
    chat_message: String,
    sector_id: u64
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let sender = PlayerId::new(ctx.sender);
    let username = get_username(ctx, ctx.sender);

    if let Some(player) = dsl.get_ships_by_player_id(&sender).next() {
        if player.get_sector_id().value() != sector_id {
            return Err(format!("Player {} is not in sector {}", username, sector_id));
        }
    } else {
        return Err(format!("Player {} does not have a ship object", username));
    }

    // If ctx.sender is a valid, unbanned, unmuted player
    info!("SectorChat #{} [{}]: {}", sector_id, username, chat_message);

    dsl.create_sector_chat_message(sender, SectorId::new(sector_id), &chat_message)?;
    Ok(())
}
