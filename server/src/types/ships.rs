use spacetimedb::{SpacetimeType, table};
use spacetimedsl::dsl;

use crate::types::stellarobjects::StellarObjectId;

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum ShipClass {
    Miner,
    Shuttle,
    Freighter,
    Fighter,
    Scout,
    Cruiser,
    BattleCruiser,
    Carrier,
}

#[dsl(plural_name = ships)]
#[table(name = ship, public)]
pub struct Ship {
    #[primary_key]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    pub owner_id: Option<u64>,      // FK to player.id
    pub faction_id: Option<u64>,    // FK to faction.id
    pub class: ShipClass,
    pub health: f32,
    pub max_health: f32,
    pub cargo_capacity: u32,
}
