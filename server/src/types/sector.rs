use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table, table};

#[table(name = sector, public)]
pub struct Sector {
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    #[index(btree)]
    pub name: String,

    pub x: f32,
    pub y: f32, // For a sector map view
}
