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
pub struct ItemStack { // TODO Do we even need this? ShipCargoItem does this exact thing.
    pub resource: ItemCategory,
    pub quantity: u32,
}

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum OrderType { // TODO move this our of economy and into an AI/NPC module
    Mine(OreType),
    HaulToStation(u64),       // station_id
    TradeAtStation(u64),      // station_id
    DefendSector(u64),        // sector_id
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {

    Ok(())
}