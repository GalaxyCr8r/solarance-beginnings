use spacetimedb::*;
use spacetimedsl::*;

use crate::tables::items::*;
use crate::tables::stellarobjects::StellarObjectId;

// pub mod definitions; // Definitions for initial ingested data.
pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
                  // pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum NpcArchetype {
    // Broader than NpcType, defines their role
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

#[dsl(plural_name = npc_ship_controllers)]
#[table(name = npc_ship_controller, public)]
pub struct NpcShipController {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(path = StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    pub stellar_object_id: u64,

    // Combat actions (same as player)
    pub fire_weapons: bool,
    pub fire_missiles: bool,
    pub targetted_sobj_id: Option<u64>,

    // AI state
    pub ai_behavior: NpcBehavior,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(dsl: &DSL) -> Result<(), String> {
    timers::init(dsl)?;

    Ok(())
}
