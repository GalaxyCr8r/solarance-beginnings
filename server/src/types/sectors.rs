use spacetimedb::{table, ReducerContext, SpacetimeType};
use spacetimedsl::{dsl, Wrapper};

use crate::types::{common, factions::*, jumpgates::reducers::*};

pub mod definitions; // Definitions for initial ingested data.
pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
                  //pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum SpectralKind {
    /// Hottest, brightest, and bluest stars.
    O,
    /// Blue-white Stars
    B,
    /// White Stars
    A,
    /// Yellow-white Stars
    F,
    /// Yellow Stars (Sol)
    G,
    /// Orange Stars
    K,
    /// Coolest, dimmest, and reddest stars.
    M,
}

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum StarSystemObjectKind {
    Star,
    Planet,
    Moon,
    AsteroidBelt,
    NebulaBelt,
}

#[dsl(plural_name = star_systems)]
#[table(name = star_system, public)]
pub struct StarSystem {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::types::sectors, table = star_system_object)]
    #[referenced_by(path = crate::types::sectors, table = sector)]
    id: u32,

    #[unique]
    pub name: String, // e.g., "Sol", "Alpha Centauri"

    pub map_coordinates: common::Vec2, // Coordinates on the galactic map
    pub spectral: SpectralKind,
    /// 0, Hypergiants ... 5, Main sequence (Sol) ... 7, White dwarfs
    pub luminosity: u8,

    #[index(btree)]
    #[use_wrapper(path = FactionId)]
    #[foreign_key(path = crate::types::factions, table = faction, column = id, on_delete = Error)]
    /// FK to Faction, can change
    pub controlling_faction_id: u32,
    //pub discovered_by_faction_id: Option<u32>, // First faction to chart it
}

#[dsl(plural_name = star_system_objects)]
#[table(name = star_system_object, public)]
pub struct StarSystemObject {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u32,

    #[index(btree)]
    #[use_wrapper(path = StarSystemId)]
    #[foreign_key(path = crate::types::sectors, table = star_system, column = id, on_delete = Delete)]
    /// FK to StarSystem
    pub system_id: u32,

    pub kind: StarSystemObjectKind,

    /// Object's star system position
    pub orbit_au: f32,
    /// Either the rotation in the orbit in radians, or the kilometers wide for asteroid/nebula belts.
    pub rotation_or_width_km: f32,

    pub gfx_key: Option<String>, // Key for client to look up image
}

#[dsl(plural_name = sectors)]
#[table(name = sector, public)]
pub struct Sector {
    #[primary_key] // NOT Auto-inc so it can be reloaded as-is
    #[create_wrapper]
    #[referenced_by(path = crate::types::sectors, table = asteroid_sector)]
    #[referenced_by(path = crate::types::stellarobjects, table = stellar_object)]
    #[referenced_by(path = crate::types::asteroids, table = asteroid)]
    #[referenced_by(path = crate::types::ships, table = ship)]
    #[referenced_by(path = crate::types::ships, table = docked_ship)]
    #[referenced_by(path = crate::types::stations, table = station)]
    #[referenced_by(path = crate::types::jumpgates, table = jump_gate)]
    #[referenced_by(path = crate::types::chats, table = sector_chat_message)]
    #[referenced_by(path = crate::types::items, table = cargo_crate)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(path = StarSystemId)]
    #[foreign_key(path = crate::types::sectors, table = star_system, column = id, on_delete = Error)]
    /// FK to StarSystem
    pub system_id: u32,

    pub name: String,
    pub description: Option<String>,

    #[index(btree)]
    #[use_wrapper(path = FactionId)]
    #[foreign_key(path = crate::types::factions, table = faction, column = id, on_delete = Error)]
    /// FK to Faction, can change
    pub controlling_faction_id: u32,
    /// 0 (lawless) to 10 (heavily policed)
    /// Depends on the adjecency a faction Garrison station
    pub security_level: u8,

    // Sector Potentials
    /// How much sunlight the current sector has.
    /// From 1.0 being in orbit around the sun, to 0.0 being outside a solar system.
    /// Most sectors will have 0.9 - 0.5 depending on how far from the center of the solar system it is.
    /// Solar power plants want to be in sectors of 0.5+
    pub sunlight: f32,
    /// How much weird stuff the current sector has going on.
    /// From 1.0 being inside the middle of eye of chaos, to 0.0 being a normal solar system.
    /// Most sectors will have 0.0 - 0.1, research stations want to be in sectors of 0.5+
    pub anomalous: f32,
    /// How much gas/dust the current sector has.
    /// From 1.0 being so thick you can't use your sensors, to 0.0 being a clear space.
    /// Most sectors will have 0.0 - 0.1, pirate stations want to be in sectors of 0.5+
    pub nebula: f32,
    /// How likely rare ore is to appear in the current sector.
    /// From 1.0 being ONLY rare ore, to 0.0 being only iron.
    /// Most sectors will have 0.0 - 0.1, refinery stations want to be in sectors of 0.5+
    pub rare_ore: f32,

    // Sector's star system position
    pub x: f32,
    pub y: f32,

    pub background_gfx_key: Option<String>, // Key for client to look up background image
}

#[dsl(plural_name = asteroid_sectors)]
#[table(name = asteroid_sector)]
pub struct AsteroidSector {
    #[primary_key] // NOT Auto-inc so it can be reloaded as-is
    #[use_wrapper(path = SectorId)]
    #[foreign_key(path = crate::types::sectors, table = sector, column = id, on_delete = Delete)]
    id: u64,

    pub sparseness: u8,             // Relative amount of asteroids to spawn
    pub rarity: u8,                 // Skews the amount of spawned asteroids with high rarity ores
    pub cluster_extent: f32,        // How far from 0,0 can asteroids spawn
    pub cluster_inner: Option<f32>, // How far from 0,0 can asteroids NOT spawn
}
//////////////////////////////////////////////////////////////
// Impls
//////////////////////////////////////////////////////////////

impl Sector {
    pub fn get(ctx: &ReducerContext, id: &SectorId) -> Result<Sector, String> {
        Ok(dsl(ctx).get_sector_by_id(id)?)
    }
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    timers::init(ctx)?;
    definitions::init(ctx)?;

    Ok(())
}

//////////////////////////////////////////////////////////////
// Utilities
//////////////////////////////////////////////////////////////

/// Creates a jumpgate in each sector, using the direction of the each other sector's position
fn connect_sectors_with_warpgates(
    ctx: &ReducerContext,
    a: &Sector,
    b: &Sector,
) -> Result<(), String> {
    let a_pos = glam::Vec2::new(a.x, a.y);
    let b_pos = glam::Vec2::new(b.x, b.y);
    //info!("Sector Positions: A{} B{}", a_pos, b_pos);

    let a_angle = (b_pos - a_pos).to_angle();
    let b_angle = (a_pos - b_pos).to_angle();
    //info!("Sector Angles: A{} B{}", a_angle, b_angle);

    let a_wp_pos = glam::Vec2::from_angle(a_angle) * 5000.0;
    let b_wp_pos = glam::Vec2::from_angle(b_angle) * 5000.0;
    //info!("Sector WP Pos: A{} B{}", a_wp_pos, b_wp_pos);

    create_jumpgate_in_sector(
        ctx, a.id, a_wp_pos.x, a_wp_pos.y, b.id, b_wp_pos.x, b_wp_pos.y,
    )?;
    create_jumpgate_in_sector(
        ctx, b.id, b_wp_pos.x, b_wp_pos.y, a.id, a_wp_pos.x, a_wp_pos.y,
    )?;

    Ok(())
}

/// For jumpdrive-enabled ships, calculates the incoming vector the ship should be entering from.
pub fn get_entrance_angle(departing: &Sector, destination: &Sector) -> f32 {
    let a_pos = glam::Vec2::new(departing.x, departing.y);
    let b_pos = glam::Vec2::new(destination.x, destination.y);

    // Destination entrance angle
    (a_pos - b_pos).to_angle()
}
