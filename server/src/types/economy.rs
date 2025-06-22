use spacetimedb::{ReducerContext, SpacetimeType};

use crate::types::items::*;

// pub mod definitions; // Definitions for initial ingested data.
// pub mod impls; // Impls for this file's structs
// pub mod reducers; // SpacetimeDB Reducers for this file's structs.
// pub mod rls; // Row-level-security rules for this file's structs.
// pub mod timers; // Timers related to this file's structs.
// pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum NpcArchetype { // Broader than NpcType, defines their role
    Trader,
    Miner,
    PirateRaider,
    PirateSmuggler,
    FactionMilitaryPatrol,
    FactionMilitaryEliteGuard,
    CivilianTransportFreighter,
    ExplorerScientist,
    QuestGiverStationBound,
    QuestGiverFieldOperative,
    BountyHunter,
}

#[derive(SpacetimeType, Debug, Clone)]
pub struct ResourceAmount {
    //#[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub resource_item_id: u32,

    pub quantity: u32,
}

impl PartialEq for ResourceAmount {
    fn eq(&self, other: &Self) -> bool {
        self.resource_item_id == other.resource_item_id && self.quantity == other.quantity
    }
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {

    Ok(())
}