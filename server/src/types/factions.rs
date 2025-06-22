use spacetimedb::{table, Identity, ReducerContext, SpacetimeType};
use spacetimedsl::dsl;

// pub mod definitions; // Definitions for initial ingested data.
// pub mod impls; // Impls for this file's structs
// pub mod reducers; // SpacetimeDB Reducers for this file's structs.
// pub mod rls; // Row-level-security rules for this file's structs.
// pub mod timers; // Timers related to this file's structs.
// pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum FactionTier {
    /// The largest type of faction - new players can spawn into it.
    Universal,
    /// Multi-corporation faction, but still under a universal faction's aegis
    Conglomerate,
    /// A large player-controlled faction.
    Guild,
    /// A small/medium player-controlled faction.
    Corporation,
    /// A small, usually temporary, group within a faction.
    Squad, // e.g., local corporation, pirate clan
}

#[dsl(plural_name = faction_definitions)]
#[table(name = faction_definition, public)]
pub struct Faction {
    #[primary_key]
    #[wrap]
    pub id: u32,

    pub name: String,
    pub description: String,

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
    #[wrap]
    pub id: u64,

    #[index(btree)] // To find all players with standing for a faction
    #[wrapped(path = crate::types::factions::FactionId)]
    /// FK to FactionDefinition
    pub faction_one_id: u32,

    #[index(btree)] // To find all players with standing for a faction
    #[wrapped(path = crate::types::factions::FactionId)]
    /// FK to FactionDefinition
    pub faction_two_id: u32,

    /// How the two factions regard each other.
    /// -100 is hated enemies.
    /// 100 is erstwhile allies.
    pub reputation_score: i32,
}

#[dsl(plural_name = player_faction_standings)]
#[table(name = player_faction_standing, public)]
pub struct PlayerFactionStanding {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[index(btree)] // To find all standings for a player
    pub player_identity: Identity,

    #[index(btree)] // To find all players with standing for a faction
    #[wrapped(path = crate::types::factions::FactionId)]
    /// FK to FactionDefinition
    pub faction_id: u32,

    pub reputation_score: i32,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {

    Ok(())
}