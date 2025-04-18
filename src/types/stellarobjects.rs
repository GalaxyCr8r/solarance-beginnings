use std::time::Duration;
use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table};

#[derive(SpacetimeType, Clone)]
pub struct StellarPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(SpacetimeType, Clone)]
pub struct StellarTransform {
    position: StellarPosition,
    rotation_radians: f32,
}

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum StellarObjectKinds {
    Ship,
    Asteroid,
    Station,
}

#[spacetimedb::table(name = stellar_object, public)]
pub struct StellarObject {
    #[primary_key]
    #[auto_inc]
    id: u64,
    pub kind: StellarObjectKinds,
    #[index(btree)]
    sector_id: u64, // Foreign key to a Sector
}

#[spacetimedb::table(name = stellar_object_internal)]
#[spacetimedb::table(name = stellar_object_hi_res, public)]
#[spacetimedb::table(name = stellar_object_low_res, public)]
pub struct StellarObjectTransform {
    #[unique]
    pub sobj_id: u32,
    pub transform: StellarTransform,
}
