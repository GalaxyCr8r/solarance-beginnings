use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table, table};

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum ShipClass {
    Miner,
    Freighter,
    Fighter,
    Scout,
}

#[table(name = ship, public)]
pub struct Ship {
    #[primary_key]
    pub entity_id: u64,

    pub owner_id: Option<u64>,      // FK to player.id
    pub faction_id: Option<u64>,    // FK to faction.id
    pub class: ShipClass,
    pub health: f32,
    pub max_health: f32,
    pub cargo_capacity: u32,
}
