use spacetimedb::{table, ReducerContext};
use spacetimedsl::{dsl, Wrapper};

use super::common::Vec2;

// pub mod definitions; // Definitions for initial ingested data.
// pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
                  // pub mod rls; // Row-level-security rules for this file's structs.
                  // pub mod timers; // Timers related to this file's structs.
                  // pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[dsl(plural_name = jump_gates)]
#[table(name = jump_gate, public)]
pub struct JumpGate {
    #[primary_key]
    #[use_wrapper(path = crate::types::stellarobjects::StellarObjectId)]
    #[foreign_key(path = crate::types::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    #[use_wrapper(path = crate::types::sectors::SectorId)]
    #[index(btree)] // To find gates in a specific sector
    #[foreign_key(path = crate::types::sectors, table = sector, column = id, on_delete = Error)]
    /// FK to SectorDefinition where this gate physically is
    pub current_sector_id: u64,

    #[index(btree)]
    #[use_wrapper(path = crate::types::sectors::SectorId)]
    #[foreign_key(path = crate::types::sectors, table = sector, column = id, on_delete = Error)]
    /// FK to SectorDefinition for the destination sector
    pub target_sector_id: u64,
    pub target_gate_arrival_pos: Vec2, // Position where ships appear in the target_sector_id

    pub gfx_key: Option<String>, // For clients

    pub is_active: bool, // Can the gate be used?

                         //pub required_item_id: Option<u32>, // Optional: item needed to use the gate (e.g., jump drive, key)
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {
    Ok(())
}
