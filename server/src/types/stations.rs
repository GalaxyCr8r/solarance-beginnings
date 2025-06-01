use spacetimedb::{table, ReducerContext, SpacetimeType};
use spacetimedsl::{dsl};

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum StationKind {
    TradeHub,
    Refinery,
    Factory,
    StorageDepot,
}

#[dsl(plural_name = stations)]
#[table(name = station, public)]
pub struct Station {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[index(btree)]
    pub kind: StationKind,

    #[index(btree)]
    pub current_sector_id: u32, // FK to SectorDefinition

    #[index(btree)]
    #[wrapped(path = crate::types::stellarobjects::StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    #[index(btree)]
    #[wrapped(path = crate::types::factions::FactionDefinitionId)]
    pub owner_faction_id: u32, // FK to FactionDefinition

    pub name: String,
    
    // services_offered: Vec<StationServiceType>, // Could be an enum or FKs to service definitions
    
    pub gfx_key: Option<String>,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {

    Ok(())
}