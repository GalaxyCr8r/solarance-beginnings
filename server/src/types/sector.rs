use spacetimedb::table;

#[table(name = sector_location, public)]
pub struct SectorLocation {
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    #[index(btree)]
    pub name: String,

    pub x: f32, // For a sector map view
    pub y: f32,
}
