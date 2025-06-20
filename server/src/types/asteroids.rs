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
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    #[index(btree)] // To find asteroids in a specific sector
    #[wrapped(path = crate::types::sectors::SectorId)]
    pub current_sector_id: u64, // FK to Sector.id // Because asteroid_sector.id exists, this can't be named sector_id.

    pub size_radius: f32, // For collision
    
    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    pub resource_item_id: u32, // FK to ItemDefinition (e.g., Iron Ore, Silicon)

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

