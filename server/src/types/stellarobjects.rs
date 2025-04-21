use std::time::Duration;
use spacetimedb::{rand::{self, RngCore}, Identity, ReducerContext, SpacetimeType, Table};

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

#[derive(SpacetimeType, PartialEq)]
enum TransformResolution {
    Internal = 0, // Internal transform state
    High = 1, // Nearby objects
    Low = 2 // For out-of-sector or far-away objects
}

#[spacetimedb::table(name = stellar_object_internal)]
#[spacetimedb::table(name = stellar_object_hi_res, public)]
#[spacetimedb::table(name = stellar_object_low_res, public)]
pub struct StellarObjectTransform {
    #[unique]
    pub sobj_id: u64,
    pub transform: StellarTransform
}


#[spacetimedb::reducer]
pub fn update_object_transform(ctx: &ReducerContext, object: StellarObject, transform: StellarTransform) {
    // We'll update this user's internal position, not their public position. Public positions will be updated in the scheduled timer.

    //// TODO: Add checking so you can only update the transform of your own ship.. or something better entirely

    if ctx
        .db
        .stellar_object_internal()
        .sobj_id()
        .find(object.id)
        .is_some()
    {
        ctx.db
            .stellar_object_internal()
            .sobj_id()
            .update(StellarObjectTransform {
                sobj_id: object.id,
                transform,
            });
    } else {
        ctx.db
            .stellar_object_internal()
            .insert(StellarObjectTransform {
                sobj_id: object.id,
                transform,
            });
    }
}

#[spacetimedb::reducer]
pub fn create_stellar_object(ctx: &ReducerContext, kind: StellarObjectKinds, sector_id: u64, transform: StellarTransform) -> Result<(), String> {
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
        ctx.db.stellar_object_internal().insert(StellarObjectTransform {
            sobj_id: object.unwrap().id,
            transform: transform
        });
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
    
    create_stellar_object(ctx, StellarObjectKinds::Ship, 0, StellarTransform { 
        position: StellarPosition { 
            x: f32::from_bits(ctx.rng().next_u32() % 1024), 
            y: f32::from_bits(ctx.rng().next_u32() % 512) 
        }, 
        rotation_radians: 0.
    })
}