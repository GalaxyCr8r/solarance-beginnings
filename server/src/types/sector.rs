use spacetimedb::table;
use spacetimedsl::dsl;

#[dsl(plural_name = sectors)]
#[table(name = sector, public)]
pub struct Sector {
    #[primary_key] // NOT Auto-inc so it can be reloaded as-is
    #[wrap]
    pub id: u64,

    #[index(btree)]
    pub name: String,
    pub description: Option<String>,

    pub x: f32, // For a sector map view
    pub y: f32,

    pub background_gfx_key: Option<String>, // Key for client to look up background image
}
