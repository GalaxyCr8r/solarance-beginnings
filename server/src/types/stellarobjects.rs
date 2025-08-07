use log::info;
use spacetimedb::{rand::Rng, table, Identity, ReducerContext, SpacetimeType};
use spacetimedsl::*;

use super::sectors::SectorId;

//pub mod definitions; // Definitions for initial ingested data.
pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

/// What kind of stellar object it is. OBE with the advent of asteroid/ship/station tables?
#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum StellarObjectKinds {
    Ship,
    Asteroid,
    CargoCrate,
    Station,
    JumpGate,
}

/// An object that exists inside a sector.
#[dsl(plural_name = stellar_objects)]
#[table(name = stellar_object, public)]
pub struct StellarObject {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::types::stellarobjects, table = sobj_velocity)]
    #[referenced_by(path = crate::types::stellarobjects, table = sobj_internal_transform)]
    #[referenced_by(path = crate::types::stellarobjects, table = sobj_hi_res_transform)]
    #[referenced_by(path = crate::types::stellarobjects, table = sobj_low_res_transform)]
    #[referenced_by(path = crate::types::stellarobjects, table = sobj_turn_left_controller)]
    #[referenced_by(path = crate::types::stellarobjects, table = sobj_player_window)]
    #[referenced_by(path = crate::types::asteroids, table = asteroid)]
    #[referenced_by(path = crate::types::ships, table = ship)]
    #[referenced_by(path = crate::types::stations, table = station)]
    #[referenced_by(path = crate::types::jumpgates, table = jump_gate)]
    #[referenced_by(path = crate::types::items, table = cargo_crate)]
    #[referenced_by(path = crate::types::players, table = player_ship_controller)]
    id: u64,

    #[index(btree)]
    pub kind: StellarObjectKinds,

    #[index(btree)]
    #[use_wrapper(path = crate::types::sectors::SectorId)]
    #[foreign_key(path = crate::types::sectors, table = sector, column = id, on_delete = Delete)]
    /// FK to SectorLocation
    pub sector_id: u64,
}

/// The current velocity of a stellar object.
#[dsl(plural_name = sobj_velocities)]
#[table(name = sobj_velocity, public)]
#[derive(Default)]
pub struct StellarObjectVelocity {
    #[primary_key]
    #[use_wrapper(path = StellarObjectId)]
    #[foreign_key(path = crate::types::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,

    pub auto_dampen: Option<f32>,
}

/// The current exact transform of a stellar object. Used to populate low/high resolution tables.
#[dsl(plural_name = sobj_internal_transforms)]
#[table(name = sobj_internal_transform)]
#[derive(Default)]
pub struct StellarObjectTransformInternal {
    #[primary_key]
    #[use_wrapper(path = StellarObjectId)]
    #[foreign_key(path = crate::types::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

/// The position of a stellar object that has a high rate of updates
#[dsl(plural_name = sobj_hi_res_transforms)]
#[table(name = sobj_hi_res_transform, public)]
#[derive(Default)]
pub struct StellarObjectTransformHiRes {
    #[primary_key]
    #[use_wrapper(path = StellarObjectId)]
    #[foreign_key(path = crate::types::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

#[dsl(plural_name = sobj_low_res_transforms)]
#[table(name = sobj_low_res_transform, public)]
#[derive(Default)]
pub struct StellarObjectTransformLowRes {
    #[primary_key]
    #[use_wrapper(path = StellarObjectId)]
    #[foreign_key(path = crate::types::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

#[dsl(plural_name = sobj_turn_left_controllers)]
#[table(name = sobj_turn_left_controller)]
pub struct StellarObjectControllerTurnLeft {
    #[primary_key]
    #[use_wrapper(path = StellarObjectId)]
    #[foreign_key(path = crate::types::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,
}

#[dsl(plural_name = sobj_player_windows)]
#[table(name = sobj_player_window, public)]
pub struct StellarObjectPlayerWindow {
    #[primary_key]
    #[use_wrapper(path = crate::players::PlayerId)]
    #[foreign_key(path = crate::players, table = player, column = id, on_delete = Delete)]
    id: Identity,

    #[unique]
    #[use_wrapper(path = StellarObjectId)]
    #[foreign_key(path = crate::types::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    pub sobj_id: u64,

    pub window: f32, // How big of a rectangular window should be
    pub margin: f32, // How much space can you move within the window before recalculating
    // Top Left
    pub tl_x: f32,
    pub tl_y: f32,
    // Bottom Right
    pub br_x: f32,
    pub br_y: f32,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    timers::init(ctx)?;

    Ok(())
}
