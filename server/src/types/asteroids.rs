use spacetimedb::{table, Timestamp};
use spacetimedsl::dsl;

use crate::types::stellarobjects::StellarObjectId;

#[dsl(plural_name = asteroids)]
#[table(name = asteroid, public)]
pub struct Asteroid {
    #[primary_key]
    #[auto_inc]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    #[index(btree)] // To find asteroids in a specific sector
    pub current_sector_id: u32, // FK to Sector.id

    pub size_radius: f32, // For collision, visual scaling, and maybe initial resource amount
    
    pub resource_item_id: u32, // FK to ItemDefinition (e.g., Iron Ore, Silicon)
    pub current_resources: u32, // Amount of resources left
    pub initial_resources: u32, // Original amount, for reference or respawn logic

    pub respawn_cooldown_ts: Option<Timestamp>, // When it can respawn if depleted

    pub gfx_key: Option<String>, // For client side
}
