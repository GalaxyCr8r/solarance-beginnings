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

#[spacetimedb::table(name = update_sobj_transform_timer, scheduled(update_sobj_transforms))]
pub struct UpdatePositionTimer {
    #[primary_key]
    #[auto_inc]
    scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

#[spacetimedb::reducer]
pub fn update_sobj_transforms(ctx: &ReducerContext, _arg: UpdatePositionTimer) {
    // TODO
}