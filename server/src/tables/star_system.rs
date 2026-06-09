use spacetimedb::{table, SpacetimeType};
use spacetimedsl::*;

use solarance_shared::Vec2;

use crate::tables::factions::*;

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

#[dsl(plural_name = star_systems, method(update = true))] // Keeping it true in case we want to edit the galaxy
#[table(accessor = star_system, public)]
pub struct StarSystem {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::tables::star_system, table = star_system_object)]
    #[referenced_by(path = crate::tables::sectors, table = sector)]
    #[referenced_by(path = crate::tables::messages, table = star_system_channel_message)]
    id: u32,

    #[unique]
    pub name: String, // e.g., "Sol", "Alpha Centauri"

    pub map_coordinates: Vec2, // Coordinates on the galactic map
    pub spectral: SpectralKind,
    /// 0, Hypergiants ... 5, Main sequence (Sol) ... 7, White dwarfs
    pub luminosity: u8,

    #[index(btree)]
    #[use_wrapper(FactionId)]
    #[foreign_key(path = crate::tables::factions, table = faction, column = id, on_delete = Error)]
    /// FK to Faction, can change
    pub controlling_faction_id: u32,
    //pub discovered_by_faction_id: Option<u32>, // First faction to chart it
}

#[dsl(plural_name = star_system_objects, method(update = true))] // Keeping it true in case we want to edit the galaxy
#[table(accessor = star_system_object, public)]
pub struct StarSystemObject {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u32,

    #[index(btree)]
    #[use_wrapper(StarSystemId)]
    #[foreign_key(path = crate::tables::star_system, table = star_system, column = id, on_delete = Delete)]
    /// FK to StarSystem
    pub system_id: u32,

    pub kind: StarSystemObjectKind,

    /// Object's star system position
    pub orbit_au: f32,
    /// Either the rotation in the orbit in radians, or the kilometers wide for asteroid/nebula belts.
    pub rotation_or_width_km: f32,

    pub gfx_key: Option<String>, // Key for client to look up image
}
