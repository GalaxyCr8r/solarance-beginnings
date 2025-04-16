use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table, table

#[table(name = asteroid, public)]
pub struct Asteroid {
    #[primary_key]
    pub entity_id: u64,

    pub resource_type: ResourceType,
    pub amount_remaining: u32,
}
