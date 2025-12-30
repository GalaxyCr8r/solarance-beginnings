use spacetimedb::table;
use spacetimedsl::*;

use super::common_types::Vec2;

#[dsl(plural_name = jump_gates, method(update = true))]
#[table(name = jump_gate, public)]
pub struct JumpGate {
    #[primary_key]
    #[use_wrapper(crate::tables::stellarobjects::StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    #[use_wrapper(crate::tables::sectors::SectorId)]
    #[index(btree)] // To find gates in a specific sector
    #[foreign_key(path = crate::tables::sectors, table = sector, column = id, on_delete = Error)]
    /// FK to SectorDefinition where this gate physically is
    pub current_sector_id: u64,

    #[index(btree)]
    #[use_wrapper(crate::tables::sectors::SectorId)]
    #[foreign_key(path = crate::tables::sectors, table = sector, column = id, on_delete = Error)]
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

pub fn init<T: spacetimedsl::WriteContext>(_dsl: &DSL<T>) -> Result<(), String> {
    Ok(())
}
