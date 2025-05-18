use spacetimedb::table;
use spacetimedsl::dsl;

use super::economy::ResourceType;
use crate::types::stellarobjects::StellarObjectId;

#[dsl(plural_name = asteroids)]
#[table(name = asteroid, public)]
pub struct Asteroid {
    #[primary_key]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    pub resource_type: ResourceType,
    pub amount_remaining: u32,
}
