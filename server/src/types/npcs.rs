use spacetimedb::*;
use spacetimedsl::*;

use crate::{types::items::*, *};

// pub mod definitions; // Definitions for initial ingested data.
// pub mod impls; // Impls for this file's structs
// pub mod reducers; // SpacetimeDB Reducers for this file's structs.
// pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
//pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum OrderType { // TODO move this our of economy and into an AI/NPC module
    Mine(OreType),
    HaulToStation(u64),       // station_id
    TradeAtStation(u64),      // station_id
    DefendSector(u64),        // sector_id
}