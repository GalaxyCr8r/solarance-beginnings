use spacetimedb::{table, ReducerContext};
use spacetimedsl::{dsl, Wrapper};

use super::jumpgates::create_jumpgate_in_sector;

pub mod definitions; // Definitions for initial ingested data.
pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
//pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

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
// Impls
//////////////////////////////////////////////////////////////

impl Sector {
    pub fn get(ctx: &ReducerContext, id: &SectorId) -> Option<Sector> {
        dsl(ctx).get_sector_by_id(id)
    }
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);
    
    timers::init(ctx)?;
    definitions::init(ctx)?;
    
    Ok(())
}


//////////////////////////////////////////////////////////////
// Utilities
//////////////////////////////////////////////////////////////

/// Creates a jumpgate in each sector, using the direction of the each other sector's position
fn connect_sectors_with_warpgates(ctx: &ReducerContext, a: &Sector, b: &Sector) -> Result<(), String> {
    let a_pos = glam::Vec2::new(a.x, a.y);
    let b_pos = glam::Vec2::new(b.x, b.y);
    //info!("Sector Positions: A{} B{}", a_pos, b_pos);

    let a_angle = (b_pos-a_pos).to_angle();
    let b_angle = (a_pos-b_pos).to_angle();
    //info!("Sector Angles: A{} B{}", a_angle, b_angle);

    let a_wp_pos = glam::Vec2::from_angle(a_angle) * 5000.0;
    let b_wp_pos = glam::Vec2::from_angle(b_angle) * 5000.0;
    //info!("Sector WP Pos: A{} B{}", a_wp_pos, b_wp_pos);

    create_jumpgate_in_sector(ctx, a.id, a_wp_pos.x, a_wp_pos.y, b.id, b_wp_pos.x, b_wp_pos.y)?;
    create_jumpgate_in_sector(ctx, b.id, b_wp_pos.x, b_wp_pos.y, a.id, a_wp_pos.x, a_wp_pos.y)?;

    Ok(())
}

/// For jumpdrive-enabled ships, calculates the incoming vector the ship should be entering from.
pub fn get_entrance_angle(departing: &Sector, destination: &Sector) -> f32 {
    let a_pos = glam::Vec2::new(departing.x, departing.y);
    let b_pos = glam::Vec2::new(destination.x, destination.y);

    // Destination entrance angle
    (a_pos-b_pos).to_angle()
}