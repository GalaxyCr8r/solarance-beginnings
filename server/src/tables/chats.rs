use spacetimedb::{table, Identity, Timestamp};
use spacetimedsl::*;

#[dsl(plural_name = global_chat_messages, method(update = false))]
#[table(name = global_chat_message, public)]
pub struct GlobalChatMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(crate::players::PlayerId)]
    #[foreign_key(path = crate::players, table = player, column = id, on_delete = Delete)]
    /// FK to Player
    player_id: Identity,

    message: String,

    created_at: Timestamp,
}

#[dsl(plural_name = sector_chat_messages, method(update = false))]
#[table(name = sector_chat_message, public)]
pub struct SectorChatMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(crate::players::PlayerId)]
    #[foreign_key(path = crate::players, table = player, column = id, on_delete = Delete)]
    /// FK to Player
    player_id: Identity,

    #[index(btree)] // To find asteroids in a specific sector
    #[use_wrapper(crate::tables::sectors::SectorId)]
    #[foreign_key(path = crate::tables::sectors, table = sector, column = id, on_delete = Delete)]
    /// FK to Sector.id
    sector_id: u64,

    message: String,

    created_at: Timestamp,
}

#[dsl(plural_name = faction_chat_messages, method(update = false))]
#[table(name = faction_chat_message, public)]
pub struct FactionChatMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(crate::players::PlayerId)]
    #[foreign_key(path = crate::players, table = player, column = id, on_delete = Delete)]
    /// FK to Player
    player_id: Identity,

    #[index(btree)]
    #[use_wrapper(crate::tables::factions::FactionId)]
    #[foreign_key(path = crate::tables::factions, table = faction, column = id, on_delete = Error)]
    /// FK to FactionDefinition
    faction_id: u32,

    message: String,

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

pub fn init<T: spacetimedsl::WriteContext>(_dsl: &DSL<T>) -> Result<(), String> {
    Ok(())
}

//////////////////////////////////////////////////////////////
// Reducers ///
//////////////////////////////////////////////////////////////
