use spacetimedb::{Identity, table};

#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,

    #[index(btree)]
    pub username: String,

    pub controlled_entity_id: Option<u64>, // FK to Entity
    pub current_sector: u64,               // FK to Sector
}
