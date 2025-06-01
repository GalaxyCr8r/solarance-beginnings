use std::hash::Hasher;

use spacetimedb::{table, ReducerContext, SpacetimeType, Timestamp};
use spacetimedsl::{dsl};

#[derive(SpacetimeType, Clone, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[dsl(plural_name = global_configurations)]
#[table(name = global_config)]
pub struct GlobalConfig {
    #[primary_key]
    #[wrap]
    pub id: u32,
    
    pub active_players: u32,
    pub old_gods_defeated: u8,

    created_at: Timestamp,
    modified_at: Timestamp,
}

// Enum for different types of equipment slots on a ship
#[derive(SpacetimeType, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EquipmentSlotType {
    Weapon,
    Shield,
    Engine,
    MiningLaser,
    Special, // For things like cloaking devices, tractor beams etc.
    CargoExpansion,
}

// Enum for AI states or player commands, can be expanded
#[derive(SpacetimeType, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EntityAIState {
    Idle,
    Patrolling,
    MiningAsteroid(u64), // target asteroid_id
    AttackingTarget(u64), // target ship_id or entity_id
    MovingToPosition(Vec2),
    Jumping(u64), // target gate_id
    Docked(u64), // target station_id
    Fleeing,
    Trading,
}

///////////////////////////////////////////////////////////
// Impl
///////////////////////////////////////////////////////////

impl PartialEq for Vec2 {
    fn eq(&self, other: &Self) -> bool {
        // Compare the bit patterns of the floats.
        // This means 0.0 and -0.0 are different, and NaN == NaN.
        self.x.to_bits() == other.x.to_bits() && self.y.to_bits() == other.y.to_bits()
    }
}

impl Eq for Vec2 {}

impl std::hash::Hash for Vec2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the bit patterns of the floats.
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

///////////////////////////////////////////////////////////
// Reducers
///////////////////////////////////////////////////////////


///////////////////////////////////////////////////////////
// Utility
///////////////////////////////////////////////////////////

pub fn are_there_active_players(ctx: &ReducerContext) -> bool {
    if let Some(config) = ctx.db.global_config().id().find(0) {
        if config.active_players == 0 {
            return false
        }
    }
    true
}