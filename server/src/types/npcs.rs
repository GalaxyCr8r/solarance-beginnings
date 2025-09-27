use spacetimedb::*;
use spacetimedsl::*;

use crate::types::items::*;

// pub mod definitions; // Definitions for initial ingested data.
pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
                  // pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum NpcBehavior {
    Idle,
    Patrol,
    Attack,
    Flee,
}

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum OrderType {
    // TODO move this our of economy and into an AI/NPC module
    Mine(OreType),
    HaulToStation(u64),  // station_id
    TradeAtStation(u64), // station_id
    DefendSector(u64),   // sector_id
}

// NpcShipController will be implemented in a later task
// This requires adding a referenced_by attribute to stellar_object table
/*
#[dsl(plural_name = npc_ship_controllers)]
#[table(name = npc_ship_controller, public)]
pub struct NpcShipController {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(path = StellarObjectId)]
    #[foreign_key(path = crate::types::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    pub stellar_object_id: u64,

    // Combat actions (same as player)
    pub fire_weapons: bool,
    pub fire_missiles: bool,
    pub targetted_sobj_id: Option<u64>,

    // AI state
    pub ai_behavior: NpcBehavior,
}
*/

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    timers::init(ctx)?;

    Ok(())
}
