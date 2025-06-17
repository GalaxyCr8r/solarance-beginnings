use spacetimedb::{table, ReducerContext, SpacetimeType};
use spacetimedsl::*;

use crate::*;

// pub mod definitions; // Definitions for initial ingested data.
// pub mod impls; // Impls for this file's structs
// pub mod reducers; // SpacetimeDB Reducers for this file's structs.
// pub mod rls; // Row-level-security rules for this file's structs.
// pub mod timers; // Timers related to this file's structs.
// pub mod utility; // Utility functions (NOT reducers) for this file's structs.

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
    #[wrapped(path = sectors::SectorId)]
    pub sector_id: u64, // FK to Sector.id

    #[index(btree)]
    #[wrapped(path = stellarobjects::StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    #[index(btree)]
    #[wrapped(path = factions::FactionDefinitionId)]
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