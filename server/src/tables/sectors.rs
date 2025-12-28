use spacetimedb::table;
use spacetimedsl::*;

use crate::{
    admin::creation::create_jumpgate_internal,
    tables::{factions::*, star_system::*},
};

#[dsl(plural_name = sectors, method(update = true))]
#[table(name = sector, public)]
pub struct Sector {
    #[primary_key] // NOT Auto-inc so it can be reloaded as-is
    #[create_wrapper]
    #[referenced_by(path = crate::tables::sectors, table = asteroid_sector)]
    #[referenced_by(path = crate::tables::stellarobjects, table = stellar_object)]
    #[referenced_by(path = crate::tables::asteroids, table = asteroid)]
    #[referenced_by(path = crate::tables::ships, table = ship)]
    #[referenced_by(path = crate::tables::stations, table = station)]
    #[referenced_by(path = crate::tables::jumpgates, table = jump_gate)]
    #[referenced_by(path = crate::tables::combat, table = visual_effect)]
    #[referenced_by(path = crate::tables::chats, table = sector_chat_message)]
    #[referenced_by(path = crate::tables::items, table = cargo_crate)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(StarSystemId)]
    #[foreign_key(path = crate::tables::star_system, table = star_system, column = id, on_delete = Error)]
    /// FK to StarSystem
    system_id: u32,

    name: String,
    pub description: Option<String>,

    #[index(btree)]
    #[use_wrapper(FactionId)]
    #[foreign_key(path = crate::tables::factions, table = faction, column = id, on_delete = Error)]
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
    sunlight: f32,
    /// How much weird stuff the current sector has going on.
    /// From 1.0 being inside the middle of eye of chaos, to 0.0 being a normal solar system.
    /// Most sectors will have 0.0 - 0.1, research stations want to be in sectors of 0.5+
    anomalous: f32,
    /// How much gas/dust the current sector has.
    /// From 1.0 being so thick you can't use your sensors, to 0.0 being a clear space.
    /// Most sectors will have 0.0 - 0.1, pirate stations want to be in sectors of 0.5+
    nebula: f32,
    /// How likely rare ore is to appear in the current sector.
    /// From 1.0 being ONLY rare ore, to 0.0 being only iron.
    /// Most sectors will have 0.0 - 0.1, refinery stations want to be in sectors of 0.5+
    rare_ore: f32,

    // Sector's star system position
    x: f32,
    y: f32,

    background_gfx_key: Option<String>, // Key for client to look up background image
}

#[dsl(plural_name = asteroid_sectors, method(update = false))]
#[table(name = asteroid_sector)]
pub struct AsteroidSector {
    #[primary_key] // NOT Auto-inc so it can be reloaded as-is
    #[use_wrapper(SectorId)]
    #[foreign_key(path = crate::tables::sectors, table = sector, column = id, on_delete = Delete)]
    id: u64,

    sparseness: u8,             // Relative amount of asteroids to spawn
    rarity: u8,                 // Skews the amount of spawned asteroids with high rarity ores
    cluster_extent: f32,        // How far from 0,0 can asteroids spawn
    cluster_inner: Option<f32>, // How far from 0,0 can asteroids NOT spawn
}

//////////////////////////////////////////////////////////////
// Impls
//////////////////////////////////////////////////////////////

impl Sector {
    pub fn get<T: spacetimedsl::WriteContext>(
        dsl: &DSL<T>,
        id: &SectorId,
    ) -> Result<Sector, String> {
        Ok(dsl.get_sector_by_id(id)?)
    }
}

//////////////////////////////////////////////////////////////
// Utilities

/// Creates a jumpgate in each sector, using the direction of the each other sector's position
pub fn connect_sectors_with_warpgates<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
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

    create_jumpgate_internal(
        dsl, a.id, a_wp_pos.x, a_wp_pos.y, b.id, b_wp_pos.x, b_wp_pos.y,
    )?;
    create_jumpgate_internal(
        dsl, b.id, b_wp_pos.x, b_wp_pos.y, a.id, a_wp_pos.x, a_wp_pos.y,
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
