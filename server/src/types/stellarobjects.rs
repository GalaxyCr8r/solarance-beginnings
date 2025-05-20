use log::info;
use spacetimedb::{
    client_visibility_filter, rand::Rng, table, Filter, Identity, ReducerContext, SpacetimeType
};
use spacetimedsl::{dsl, Wrapper};

use super::{utility::{ is_server_or_owner, server_only }, sector::SectorId};

pub mod impls;
pub mod reducers;
pub mod rls;
pub mod timers;
pub mod utility;

/// What kind of stellar object it is. OBE with the advent of asteroid/ship/station tables?
#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum StellarObjectKinds {
    Ship,
    Asteroid,
    CargoCrate,
    Station,
}

/// An object that exists inside a sector.
#[dsl(plural_name = stellar_objects)]
#[table(name = stellar_object, public)]
pub struct StellarObject {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[index(btree)]
    pub kind: StellarObjectKinds,

    #[index(btree)]
    #[wrapped(path = crate::types::sector::SectorId)]
    pub sector_id: u64, // FK: SectorLocation
}

/// A stellar object directly controlled by a player.
#[dsl(plural_name = player_controlled_stellar_objects)]
#[table(name = player_controlled_stellar_object, public)]
pub struct PlayerControlledStellarObject {
    #[primary_key]
    pub identity: Identity,

    #[unique]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK to Entity

    #[index(btree)]
    #[wrapped(path = crate::types::sector::SectorId)]
    pub sector_id: u64 // FK to Sector ID - Only because actually referencing the player's stellar object would require three table hits.
}

/// The current velocity of a stellar object.
#[dsl(plural_name = sobj_velocities)]
#[table(name = sobj_velocity, public)]
#[derive(Default)]
pub struct StellarObjectVelocity {
    #[primary_key]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

/// The current exact transform of a stellar object. Used to populate low/high resolution tables.
#[dsl(plural_name = sobj_internal_transforms)]
#[table(name = sobj_internal_transform)]
#[derive(Default)]
pub struct StellarObjectTransformInternal {
    #[primary_key]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

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
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

#[dsl(plural_name = sobj_low_res_transforms)]
#[table(name = sobj_low_res_transform, public)]
#[derive(Default)]
pub struct StellarObjectTransformLowRes {
    #[primary_key]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

#[dsl(plural_name = sobj_turn_left_controllers)]
#[table(name = sobj_turn_left_controller)]
pub struct StellarObjectControllerTurnLeft {
    #[primary_key]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject
}

#[dsl(plural_name = sobj_player_windows)]
#[table(name = sobj_player_window, public)]
pub struct StellarObjectPlayerWindow {
    #[primary_key]
    pub identity: Identity,

    #[unique]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

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
