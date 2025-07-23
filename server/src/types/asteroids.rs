use spacetimedb::{table, ReducerContext};
use spacetimedsl::dsl;

use crate::types::stellarobjects::StellarObjectId;

pub mod definitions; // Definitions for initial ingested data.
pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[dsl(plural_name = asteroids)]
#[table(name = asteroid, public)]
pub struct Asteroid {
    #[primary_key]
    #[use_wrapper(path = StellarObjectId)]
    #[foreign_key(path = crate::types::stellarobjects, table = stellar_object, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    #[index(btree)] // To find asteroids in a specific sector
    #[use_wrapper(path = crate::types::sectors::SectorId)]
    #[foreign_key(path = crate::types::sectors, table = sector, on_delete = Delete)]
    /// FK to Sector.id // Because asteroid_sector.id exists, this can't be named sector_id.
    pub current_sector_id: u64,

    pub size_radius: f32, // For collision

    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    #[index(btree)]
    #[foreign_key(path = crate::types::items, table = item_definition, on_delete = Delete)]
    /// FK to ItemDefinition (e.g., Iron Ore, Silicon)
    pub resource_item_id: u32,

    pub current_resources: u16, // Amount of resources left
    pub initial_resources: u16, // Original amount, for reference or respawn logic

    pub gfx_key: Option<String>, // For client side
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    timers::init(ctx)?;

    Ok(())
}
