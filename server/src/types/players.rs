use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table, table};

#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    #[index(btree)]
    pub name: String,

    pub controlled_entity_id: Option<u64>, // FK to Entity
    pub current_sector: u64,               // FK to Sector
}

#[table(name = connection, public)]
pub struct Connection {
    #[unique]
    pub identity: Identity,
    pub entity_id: u64, // FK to Player
}