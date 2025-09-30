use std::hash::Hasher;

use spacetimedb::*;
use spacetimedsl::dsl;

//use super::{ items::ItemDefinitionId, stations::StationId };

// pub mod definitions; // Definitions for initial ingested data.
pub mod impls; // Impls for this file's structs
               // pub mod reducers; // SpacetimeDB Reducers for this file's structs.
               // pub mod rls; // Row-level-security rules for this file's structs.
               // pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[derive(SpacetimeType, Clone, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[dsl(plural_name = global_configurations)]
#[table(name = global_config)]
pub struct GlobalConfig {
    #[primary_key]
    #[create_wrapper]
    id: u32,

    pub active_players: u32,
    pub old_gods_defeated: u8,
    pub version: String,

    created_at: Timestamp,
    modified_at: Timestamp,
}

// pub struct TradeCommand {
//     item_to_sell: ItemDefinitionId,
//     station: StationId,
// }

// Enum for AI states or player commands, can be expanded
#[derive(SpacetimeType, Clone, Debug, PartialEq, Hash)]
pub enum CurrentAction {
    Idle,
    Patrolling(Vec<Vec2>),
    MiningAsteroid(u64),  // target asteroid_id
    AttackingTarget(u64), // target sobj_id
    MovingToPosition(Vec2),
    JumpingWithGate(u64),       // target gate_id
    JumpingWithHyperdrive(u64), // target gate_id
    Docking(u64),               // target station_id
    Undocking(u64),             // target station_id
    Fleeing(u64),               // target sobj_id
    Trading(u64),               // target station_id
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

impl Eq for Vec2 {
    // The PartialEq impl fulfills Eq's requirements.
}

impl std::hash::Hash for Vec2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the bit patterns of the floats.
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl Vec2 {
    pub fn to_glam(&self) -> glam::Vec2 {
        glam::Vec2 {
            x: self.x,
            y: self.y,
        }
    }
    pub fn from_glam(vec: glam::Vec2) -> Vec2 {
        Vec2 { x: vec.x, y: vec.y }
    }
}

///////////////////////////////////////////////////////////
// Reducers
///////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////
// Utility
///////////////////////////////////////////////////////////

pub fn are_there_active_players(ctx: &ReducerContext) -> bool {
    if let Some(config) = ctx.db().global_config().id().find(0) {
        if config.active_players == 0 {
            return false;
        }
    }
    true
}
