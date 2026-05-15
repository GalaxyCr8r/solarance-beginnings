use spacetimedb::*;
use spacetimedsl::*;

use crate::physics;

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum VisitedStatus {
    /// The player has not visited this system or sector.
    Unvisited,
    /// The player has observed this system or sector, either via intel report or sensors.
    Observed,
    /// The player has visited this system or sector.
    Visited,
}

impl VisitedStatus {
    pub fn is_visible(&self) -> bool {
        match self {
            VisitedStatus::Observed => true,
            VisitedStatus::Visited => true,
            _ => false,
        }
    }
}

#[dsl(plural_name = ship_configs, method(update = false, delete = true))]
#[table(accessor = ship_config, public)]
pub struct ShipConfig {
    #[primary_key]
    #[create_wrapper]
    id: u32,
    max_health: u16,
    max_speed: f32,                // meters per second
    max_turn_rate: f32,            // radians per second
    max_acceleration: f32,         // meters per second²
    max_angular_acceleration: f32, // radians per second²
}

#[dsl(plural_name = space_ships, method(update = true, delete = true))]
#[table(accessor = space_ship)]
pub struct SpaceShip {
    #[primary_key]
    #[create_wrapper]
    #[auto_inc]
    pub id: u64,

    #[index(btree)]
    #[unique]
    pub player_id: Identity,

    #[index(btree)]
    #[use_wrapper(crate::SectorId)]
    //#[foreign_key(path = crate, column = id, table = sector, on_delete = Error)]
    pub sector_id: u64,

    #[index(btree)]
    #[use_wrapper(crate::ShipConfigId)]
    //#[foreign_key(path = crate, column = id, table = ship_config, on_delete = Error)]
    pub ship_config_id: u32,
    pub health: f32,
    pub movement: physics::MovementState,
    pub input_state: physics::InputState,
    pub last_fired: spacetimedb::Timestamp,
}

#[dsl(plural_name = bullets, method(update = true, delete = true))]
#[table(accessor = bullet)]
pub struct Bullet {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    //#[referenced_by(path = crate, table = sector)]
    pub id: u32,

    #[index(btree)]
    //#[foreign_key(path = crate, column = id, table = sector, on_delete = Error)]
    pub player_id: Identity,

    #[index(btree)]
    #[use_wrapper(crate::SectorId)]
    //#[foreign_key(path = crate, column = id, table = sector, on_delete = Error)]
    pub sector_id: u64,
    /// Microseconds since unix epoch
    pub lifetime: i64,
    pub damage: f32,
    pub movement: physics::MovementState,
    created_at: spacetimedb::Timestamp,
}

#[dsl(plural_name = systems, method(update = false, delete = true))]
#[table(accessor = system)]
pub struct System {
    #[primary_key]
    #[create_wrapper]
    //#[referenced_by(path = crate, table = sector)]
    id: u32,
    name: String,
}

#[dsl(plural_name = sectors, method(update = true, delete = true), unique_index(name = position))]
#[table(accessor = sector, index(accessor = position, btree(columns=[x,y])))]
pub struct Sector {
    #[primary_key]
    #[create_wrapper]
    pub id: u64,

    #[index(btree)]
    #[use_wrapper(crate::SystemId)]
    //#[foreign_key(path = crate, column = id, table = system, on_delete = Error)]
    system_id: u32,
    x: i32,
    y: i32,
    /// If true, this sector contains a station/gate and is visible
    /// to anyone who has visited the system.
    #[index(btree)]
    is_public: bool,
}

/// Tracks where a player's ship is currently located.
#[dsl(plural_name = player_states, method(update = true, delete = true))]
#[table(accessor = player_state)]
pub struct PlayerState {
    #[primary_key]
    #[create_wrapper]
    pub id: Identity,

    // This table doesn't need STDSL foreign key constraints
    //#[foreign_key(path = crate, table = system, on_delete = Error)]
    #[index(btree)]
    pub current_system_id: u32,

    // This table doesn't need STDSL foreign key constraints
    //#[foreign_key(path = crate, table = sector, on_delete = Error)]
    #[index(btree)]
    pub current_sector_id: u64,
}

/// Private relationship table: Who has visited which system.
#[dsl(plural_name = visited_systems, method(update = true, delete = false))]
#[table(accessor = visited_system)]
pub struct VisitedSystem {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    #[index(btree)]
    player_id: Identity,
    system_id: u32,
    visited_status: VisitedStatus,
}

/// Private relationship table: Who has visited which specific sector.
#[dsl(plural_name = visited_sectors, method(update = true, delete = false))]
#[table(accessor = visited_sector)]
pub struct VisitedSector {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    #[index(btree)]
    player_id: Identity,
    #[index(btree)]
    sector_id: u64,
    visited_status: VisitedStatus,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq, Eq)]
pub enum EventType {
    Bullet,
    Explosion,
    Warpgate,
}

/// A short-lived event used to broadcast visual effects to players in a sector.
#[dsl(plural_name = damage_events, method(update = true, delete = false))]
#[table(accessor = damage_event, public, event)]
pub struct DamageEvent {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(SectorId)]
    //#[foreign_key(path = crate, table = sector, on_delete = Error)]
    sector_id: u64,
    event_type: EventType,
    pos: physics::Vec2,
    /// The timestamp when the event occurred (in microseconds).
    timestamp: i64,
}
