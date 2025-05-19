use spacetimedb::{table, ReducerContext, SpacetimeType, Timestamp};
use spacetimedsl::{dsl, Wrapper};

use super::common::Vec2;

#[dsl(plural_name = jump_gates)]
#[table(name = jump_gate, public)]
pub struct JumpGate {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[index(btree)] // To find gates in a specific sector
    pub current_sector_id: u32, // FK to SectorDefinition where this gate physically is

    #[wrapped(path = crate::types::stellarobjects::StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    pub target_sector_id: u32, // FK to SectorDefinition for the destination sector
    pub target_gate_arrival_pos: Vec2, // Position where ships appear in the target_sector_id

    pub gfx_key: Option<String>, // For clients

    pub is_active: bool, // Can the gate be used?

    //pub required_item_id: Option<u32>, // Optional: item needed to use the gate (e.g., jump drive, key)
}