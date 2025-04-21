use std::time::Duration;
use spacetimedb::{client_visibility_filter, rand::{self, Rng, RngCore}, Filter, Identity, ReducerContext, SpacetimeType, Table};


#[derive(SpacetimeType, PartialEq)]
enum TransformResolution {
    Internal = 0, // Internal transform state
    High = 1, // Nearby objects
    Low = 2 // For out-of-sector or far-away objects
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
    pub id: u64,
    pub kind: StellarObjectKinds,
    #[index(btree)]
    pub sector_id: u64, // Foreign key to a Sector
}

#[spacetimedb::table(name = stellar_object_internal)]
#[spacetimedb::table(name = stellar_object_hi_res, public)]
#[spacetimedb::table(name = stellar_object_low_res, public)]
pub struct StellarObjectTransform {
    #[unique]
    pub sobj_id: u64,
    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

#[client_visibility_filter]
const HR_OBJECT_FILTER: Filter = Filter::Sql(
    "SELECT * FROM stellar_object_hi_res WHERE x < 200 AND y < 200"
);
#[client_visibility_filter]
const LR_OBJECT_FILTER: Filter = Filter::Sql(
    "SELECT * FROM stellar_object_hi_res WHERE x >= 200 AND y >= 200"
);

/// Impls ///

impl StellarObjectTransform {
    // pub fn new(x: f32, y: f32) -> Self {
    //     Self { x, y }
    // }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 { x: self.x, y: self.y }
    }
}

/// Reducers ///

#[spacetimedb::reducer]
pub fn update_object_transform(ctx: &ReducerContext, transform: StellarObjectTransform) {
    // We'll update this user's internal position, not their public position. Public positions will be updated in the scheduled timer.

    //// TODO: Add checking so you can only update the transform of your own ship.. or something better entirely

    if ctx
        .db
        .stellar_object_internal()
        .sobj_id()
        .find(transform.sobj_id)
        .is_some()
    {
        ctx.db
            .stellar_object_internal()
            .sobj_id()
            .update(transform);
    } else {
        ctx.db
            .stellar_object_internal()
            .insert(transform);
    }
}

#[spacetimedb::reducer]
pub fn create_stellar_object(ctx: &ReducerContext, kind: StellarObjectKinds, sector_id: u64, transform: StellarObjectTransform) -> Result<(), String> {
    // if ctx.sender != ctx.identity() {
    //     panic!("This reducer can only be called by SpacetimeDB!");
    // }

    let object = ctx.db.stellar_object().try_insert(StellarObject
    {
        id: 0,
        kind: kind,
        sector_id: sector_id
    });
    if object.is_ok() {
        let sobj = object.unwrap();
        let transform = ctx.db.stellar_object_internal().insert(StellarObjectTransform {
            sobj_id: sobj.id, // TODO MAKE SURE THIS GETS  THE PROPER ID!
            .. transform
        });
        if sobj.id != transform.sobj_id {
            panic!("At the disco");
        }
        spacetimedb::log::info!("Success!")
    } else {
        spacetimedb::log::error!("Failed to create stellar object!")
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn create_stellar_object_random(ctx: &ReducerContext) -> Result<(), String> {
    // if ctx.sender != ctx.identity() {
    //     panic!("This reducer can only be called by SpacetimeDB!");
    // }
    
    create_stellar_object(ctx, StellarObjectKinds::Ship, 0, StellarObjectTransform { 
        sobj_id: 0,
        x: ctx.rng().gen_range(0.0 .. 1024.0), 
        y: ctx.rng().gen_range(0.0 .. 512.0),
        rotation_radians: ctx.rng().gen_range(-std::f32::consts::PI .. std::f32::consts::PI)
    })
}