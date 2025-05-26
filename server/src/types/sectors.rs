use spacetimedb::{table, ReducerContext};
use spacetimedsl::{dsl, Wrapper};

pub mod definitions;
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
    pub cluster_extent: f32, // How far from 0,0 can asteroids spawn
    pub cluster_inner: Option<f32> // How far from 0,0 can asteroids NOT spawn
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);
    
    timers::init(ctx)?;

    dsl.create_asteroid_sector(SectorId::new(0), 1, 3000.0, Some(1000.0))?;
    
    Ok(())
}

