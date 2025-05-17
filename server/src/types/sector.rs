use spacetimedb::table;
use spacetimedsl::dsl;

#[dsl(plural_name = sector_locations)]
#[table(name = sector_location, public)]
pub struct SectorLocation {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[index(btree)]
    pub name: String,

    pub x: f32, // For a sector map view
    pub y: f32,
}
