use spacetimedb::{table, ReducerContext, SpacetimeType};
use spacetimedsl::dsl;

pub mod definitions; // Definitions for initial ingested data.
                     // pub mod impls; // Impls for this file's structs
                     // pub mod reducers; // SpacetimeDB Reducers for this file's structs.
                     // pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum FactionTier {
    /// A meta-type of faction. Should always be unjoinable. Made up of other factions.
    Alliance,
    /// The largest type of faction - new players can spawn into it.
    Galactic,
    /// Multi-corporation faction, but still under a galactic faction's aegis
    Conglomerate,
    /// A large player-controlled faction.
    Guild,
    /// A small/medium player-controlled faction.
    Corporation,
    /// A small, usually temporary, group within a faction.
    Squad, // e.g., local corporation, pirate clan
}

#[dsl(plural_name = factions)]
#[table(name = faction, public)]
pub struct Faction {
    #[primary_key]
    #[create_wrapper]
    #[referenced_by(path = crate::types::sectors, table = star_system)]
    #[referenced_by(path = crate::types::sectors, table = sector)]
    #[referenced_by(path = crate::types::ships, table = ship)]
    #[referenced_by(path = crate::types::stations, table = station)]
    #[referenced_by(path = crate::types::chats, table = faction_chat_message)]
    #[referenced_by(path = crate::types::factions, table = faction_standing)]
    //#[referenced_by(path = crate::types::factions, table = player_faction_standing)]
    id: u32,

    /// The Faction ID of the parent faction, if any.
    pub parent_id: Option<FactionId>,

    pub name: String,
    pub short_name: String,
    pub description: String,

    /// The tier/size of this faction.
    pub tier: FactionTier,

    /// Whether players can join this faction.
    /// Some factions like Vancellan are antagonistic, or not large enough, and not joinable by players.
    pub joinable: bool,

    /// Faction's Capital Station, if it exists.
    /// FK to Station.
    pub capital_station_id: Option<u64>,
    // Other faction-specific data like relations, home sector, etc.
}

#[dsl(plural_name = faction_standings)]
#[table(name = faction_standing, public)]
pub struct FactionStanding {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)] // To find all players with standing for a faction
    #[use_wrapper(path = FactionId)]
    #[foreign_key(path = crate::types::factions, table = faction, column = id, on_delete = Error)]
    /// FK to FactionDefinition
    pub faction_one_id: u32,

    #[index(btree)] // To find all players with standing for a faction
    #[use_wrapper(path = FactionId)]
    #[foreign_key(path = crate::types::factions, table = faction, column = id, on_delete = Error)]
    /// FK to FactionDefinition
    pub faction_two_id: u32,

    /// How the two factions regard each other.
    /// -100 is hated enemies.
    /// 100 is erstwhile allies.
    pub reputation_score: i32,
}

// We are ignoring player-level faction standings at the moment.

// #[dsl(plural_name = player_faction_standings)]
// #[table(name = player_faction_standing, public)]
// pub struct PlayerFactionStanding {
//     #[primary_key]
//     #[auto_inc]
//     #[create_wrapper]
//     id: u64,

//     #[index(btree)] // To find all standings for a player
//     #[use_wrapper(path = crate::players::PlayerId)]
//     #[foreign_key(path = crate::types::players, table = player, column = id, on_delete = Delete)]
//     pub player_identity: Identity,

//     #[index(btree)] // To find all players with standing for a faction
//     #[use_wrapper(path = FactionId)]
//     #[foreign_key(path = crate::types::factions, table = faction, column = id, on_delete = Error)]
//     /// FK to FactionDefinition
//     pub faction_id: u32,

//     pub reputation_score: i32,
// }

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    definitions::init(ctx)?;
    timers::init(ctx)?;

    // Initialize faction timers for existing factions
    utility::initialize_faction_timers(ctx)?;

    Ok(())
}
