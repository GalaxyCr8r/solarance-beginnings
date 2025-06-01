use spacetimedb::{table, Identity, ReducerContext};
use spacetimedsl::dsl;

#[dsl(plural_name = faction_definitions)]
#[table(name = faction_definition, public)]
pub struct FactionDefinition {
    #[primary_key]
    #[wrap]
    pub id: u32,

    pub name: String,
    pub description: Option<String>,
    // Other faction-specific data like relations, home sector, etc.
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
    #[wrapped(path = crate::types::factions::FactionDefinitionId)]
    pub faction_id: u32, // FK to FactionDefinition

    pub reputation_score: i32,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {

    Ok(())
}