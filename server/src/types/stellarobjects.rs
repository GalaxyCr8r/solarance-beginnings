use glam::Vec2;
use spacetimedb::{
    client_visibility_filter,
    rand::{ Rng },
    Filter,
    ReducerContext,
    SpacetimeType,
    Table,
};

use super::common::{ is_server_or_owner, server_only };

#[derive(SpacetimeType, PartialEq)]
enum TransformResolution {
    Internal = 0, // Internal transform state
    High = 1, // Nearby objects
    Low = 2, // For out-of-sector or far-away objects
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
    pub sector_id: u64, // FK: SectorLocation
}

#[spacetimedb::table(name = stellar_object_internal)]
#[spacetimedb::table(name = stellar_object_velocity, public)]
#[spacetimedb::table(name = stellar_object_hi_res, public)]
#[spacetimedb::table(name = stellar_object_low_res, public)]
#[derive(Default)]
pub struct StellarObjectTransform {
    #[unique]
    pub sobj_id: u64, // FK: StellarObject
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
    "SELECT * FROM stellar_object_low_res WHERE x >= 200 OR y >= 200"
);

/// Impls ///

impl StellarObjectTransform {
    // pub fn new(x: f32, y: f32) -> Self {
    //     Self { x, y }
    // }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 { x: self.x, y: self.y }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectTransform {
        StellarObjectTransform { x: vec.x, y: vec.y, ..*self }
    }
}

/// Reducers ///

#[spacetimedb::reducer]
pub fn update_object_transform(ctx: &ReducerContext, transform: StellarObjectTransform) {
    // We'll update this user's internal position, not their public position. Public positions will be updated in the scheduled timer.

    //// TODO: Add checking so you can only update the transform of your own ship.. or something better entirely

    if ctx.db.stellar_object_internal().sobj_id().find(transform.sobj_id).is_some() {
        ctx.db.stellar_object_internal().sobj_id().update(transform);
    } else {
        ctx.db.stellar_object_internal().insert(transform);
    }
}

#[spacetimedb::reducer]
pub fn create_stellar_object(
    ctx: &ReducerContext,
    kind: StellarObjectKinds,
    sector_id: u64,
    transform: StellarObjectTransform,
    forward_velocity: f32
) -> Result<(), String> {
    server_only(ctx);

    let object = ctx.db.stellar_object().try_insert(StellarObject {
        id: 0,
        kind: kind,
        sector_id: sector_id,
    });
    if object.is_ok() {
        let sobj = object.unwrap();
        let new_transform = ctx.db.stellar_object_internal().insert(StellarObjectTransform {
            sobj_id: sobj.id, // TODO MAKE SURE THIS GETS  THE PROPER ID!
            ..transform
        });
        if sobj.id != new_transform.sobj_id {
            panic!("At the disco");
        }
        let velocity = (StellarObjectTransform {
            sobj_id: sobj.id,
            ..Default::default()
        }).from_vec2(Vec2::from_angle(transform.rotation_radians) * forward_velocity);

        ctx.db.stellar_object_velocity().insert(velocity);
        spacetimedb::log::info!("Success!");
    } else {
        spacetimedb::log::error!("Failed to create stellar object!");
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn create_stellar_object_random(ctx: &ReducerContext) -> Result<(), String> {
    server_only(ctx);

    create_stellar_object(
        ctx,
        StellarObjectKinds::Ship,
        0,
        StellarObjectTransform {
            sobj_id: 0,
            x: ctx.rng().gen_range(0.0..1024.0),
            y: ctx.rng().gen_range(0.0..512.0),
            rotation_radians: ctx.rng().gen_range(-std::f32::consts::PI..std::f32::consts::PI),
        },
        1.337
    )
}

pub fn stellar_object_increase_forward_velocity(ctx: &ReducerContext, sobj_id: u64, amount: f32) -> Result<(), String> {
    is_server_or_owner(ctx)?;
    
    match ctx.db.stellar_object_velocity().sobj_id().find(sobj_id) {
        Some(velocity) => {
            // Calculate the new velocity by adding the forward vector scaled by the amount
            let new_velocity = velocity.to_vec2() + Vec2::from_angle(velocity.rotation_radians) * amount;
            velocity.from_vec2(new_velocity);
            ctx.db.stellar_object_velocity().sobj_id().update(velocity);

            Ok(())
        }
        None => Err("Stellar object not found!".to_string()),
    }
}

pub fn stellar_object_increase_rotational_velocity(ctx: &ReducerContext, sobj_id: u64, amount_radians: f32) -> Result<(), String> {
    is_server_or_owner(ctx)?;
    
    match ctx.db.stellar_object_velocity().sobj_id().find(sobj_id) {
        Some(velocity) => {
            ctx.db.stellar_object_velocity().sobj_id().update(StellarObjectTransform {
                rotation_radians: amount_radians, ..velocity
            });

            Ok(())
        }
        None => Err("Stellar object not found!".to_string()),
    }
}

#[spacetimedb::reducer]
pub fn update_stellar_object_velocity(
    ctx: &ReducerContext,
    velocity: StellarObjectTransform
) -> Result<(), String> {
    is_server_or_owner(ctx)?;
    if ctx.db.stellar_object_velocity().sobj_id().find(velocity.sobj_id).is_none() {
        return Err("Stellar object not found!".to_string());
    }
    match ctx.db.stellar_object_velocity().sobj_id().find(velocity.sobj_id) {
        Some(prev_velocity) => {
            // Check if the acceleration required for the velocity change is too high
            let acceleration = (velocity.to_vec2() - prev_velocity.to_vec2()).length();
            if acceleration > 10.0 {
                //// TODO: Make this variable per ship type
                log::info!("Acceleration too high! {:?}", acceleration);

                // Reduce the acceleration down
                let new_velocity =
                    prev_velocity.to_vec2() +
                    (velocity.to_vec2() - prev_velocity.to_vec2()).normalize() * 10.0;
                velocity.from_vec2(new_velocity);
            }

            // Check if the absolute velocity is too fast for the ship.
            if velocity.to_vec2().length() > 100.0 {
                //// TODO: Make this variable per ship type
                log::info!("Velocity too high! {:?}", velocity.to_vec2().length());

                // Reduce the velocity down
                let new_velocity = velocity.to_vec2().normalize() * 100.0;
                velocity.from_vec2(new_velocity);
            }
        }
        None => {
            return Err("Stellar object not found!".to_string());
        }
    }

    ctx.db.stellar_object_velocity().sobj_id().update(velocity);
    Ok(())
}
