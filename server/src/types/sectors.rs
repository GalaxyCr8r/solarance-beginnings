use spacetimedb::{table, ReducerContext};
use spacetimedsl::dsl;

pub mod timers;

#[dsl(plural_name = sectors)]
#[table(name = sector, public)]
pub struct Sector {
    #[primary_key] // NOT Auto-inc so it can be reloaded as-is
    #[wrap]
    pub id: u64,

    pub name: String,
    pub description: Option<String>,

    pub x: f32, // For a sector map view
    pub y: f32,

    pub background_gfx_key: Option<String>, // Key for client to look up background image
}

#[dsl(plural_name = asteroid_sectors)]
#[table(name = asteroid_sector, public)]
pub struct AsteroidSector {
    #[primary_key] // NOT Auto-inc so it can be reloaded as-is
    #[wrapped(path = SectorId)]
    pub id: u64,

    pub sparseness: u8, // Relative amount of asteroids to spawn
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    timers::init(ctx)?;
    
    Ok(())
}

