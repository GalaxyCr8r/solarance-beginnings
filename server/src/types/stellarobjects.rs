use glam::Vec2;
use log::info;
use spacetimedb::{
    client_visibility_filter,
    rand::Rng,
    Filter,
    Identity,
    ReducerContext,
    SpacetimeType,
    Table,
};

use super::{ common::{ is_server_or_owner, server_only }, players::player };

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
#[derive(Default, Clone)]
pub struct StellarObjectTransform {
    #[unique]
    pub sobj_id: u64, // FK: StellarObject
    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

#[spacetimedb::table(name = stellar_object_controller_turn_left)]
pub struct StellarObjectControllerTurnLeft {
    #[primary_key]
    pub sobj_id: u64, // FK: StellarObject
}

#[spacetimedb::table(name = stellar_object_player_window)]
pub struct StellarObjectPlayerWindow {
    #[primary_key]
    pub identity: Identity,
    #[unique]
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

#[client_visibility_filter]
const HR_OBJECT_FILTER: Filter = Filter::Sql(
    "SELECT o.* 
    FROM stellar_object_hi_res o
    JOIN stellar_object_player_window w
    WHERE w.identity = :sender AND
          (o.x > w.tl_x OR 
          o.y > w.tl_y OR 
          o.x < w.br_x OR 
          o.y < w.br_y)
    "
);
#[client_visibility_filter]
const LR_OBJECT_FILTER: Filter = Filter::Sql(
    "SELECT o.* 
    FROM stellar_object_low_res o
    JOIN stellar_object_player_window w
    WHERE w.identity = :sender AND
          (o.x <= w.tl_x OR 
          o.y <= w.tl_y OR 
          o.x >= w.br_x OR 
          o.y >= w.br_y)
    "
);

/*
const ACCOUNT_FILTER_FOR_ADMINS: Filter = Filter::Sql(
    "SELECT account.* FROM account JOIN admin WHERE admin.identity = :sender"
);

const PLAYER_FILTER: Filter = Filter::Sql("
    SELECT q.*
    FROM account a
    JOIN player p ON a.id = p.id
    JOIN player q on p.level = q.level
    WHERE a.identity = :sender
");
*/

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
pub fn create_stellar_object_player_window_for(ctx: &ReducerContext, sobj_id: u64) {
    // Find who owns the object, if any
    let mut owning_player = None;
    for player in ctx.db.player().iter() {
        if let Some(object) = player.controlled_entity_id {
            if object == sobj_id {
                owning_player = Some(player);
                break;
            }
        }
    }
    if owning_player.is_none() {
        info!("Welp we tried and failed");
        return;
    }

    // Create the window for the object
    ctx.db
        .stellar_object_player_window()
        .insert(StellarObjectPlayerWindow {
            identity: owning_player.unwrap().identity,
            sobj_id,
            window: 2000.0,
            margin: 1000.0,
            tl_x: -2000.0,
            tl_y: -2000.0,
            br_x: 2000.0,
            br_y: 2000.0,
        });
}

#[spacetimedb::reducer]
pub fn create_turn_left_controller_for(ctx: &ReducerContext, sobj_id: u64) {
    if let Some(controller) = ctx.db.stellar_object_controller_turn_left().sobj_id().find(sobj_id) {
        ctx.db.stellar_object_controller_turn_left().delete(controller);
        spacetimedb::log::info!("Deleted controller #{}", sobj_id);
    } else {
        let controller = ctx.db
            .stellar_object_controller_turn_left()
            .insert(StellarObjectControllerTurnLeft {
                sobj_id: sobj_id,
            });
        spacetimedb::log::info!("Created controller #{}", controller.sobj_id);
    }
}

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

    match create_stellar_object_internal(ctx, kind, sector_id, transform, forward_velocity) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
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

#[spacetimedb::reducer]
pub fn update_stellar_object_velocity(
    ctx: &ReducerContext,
    velocity: StellarObjectTransform
) -> Result<(), String> {
    is_server_or_owner(ctx, velocity.sobj_id)?;
    if ctx.db.stellar_object_velocity().sobj_id().find(velocity.sobj_id).is_none() {
        return Err("Stellar object not found!".to_string());
    }
    let mut update_velocity = velocity.clone();
    match ctx.db.stellar_object_velocity().sobj_id().find(velocity.sobj_id) {
        Some(prev_velocity) => {
            // Check if the acceleration required for the velocity change is too high
            let acceleration = (velocity.to_vec2() - prev_velocity.to_vec2()).length();
            if acceleration > 2.0 {
                //// TODO: Make this variable per ship type
                log::info!("Acceleration too high! {:?}", acceleration);

                // Reduce the acceleration down
                let new_velocity =
                    prev_velocity.to_vec2() +
                    (update_velocity.to_vec2() - prev_velocity.to_vec2()).normalize() * 2.0;
                update_velocity = update_velocity.from_vec2(new_velocity);
            }

            // Check if the absolute velocity is too fast for the ship.
            if update_velocity.to_vec2().length() > 100.0 {
                //// TODO: Make this variable per ship type
                log::info!("Velocity too high! {:?}", update_velocity.to_vec2().length());

                // Reduce the velocity down
                let new_velocity = update_velocity.to_vec2().normalize() * 100.0;
                update_velocity = update_velocity.from_vec2(new_velocity);
            }
        }
        None => {
            return Err("Stellar object's velocity table entry was not found!".to_string());
        }
    }

    log::info!(
        "SObj ID #{} - New Velocity: {}, {}",
        update_velocity.sobj_id,
        update_velocity.x,
        update_velocity.y
    );
    ctx.db.stellar_object_velocity().sobj_id().update(update_velocity);
    Ok(())
}

/// Helper Functions ///

pub fn create_stellar_object_internal(
    ctx: &ReducerContext,
    kind: StellarObjectKinds,
    sector_id: u64,
    transform: StellarObjectTransform,
    forward_velocity: f32
) -> Result<StellarObject, String> {
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
        return Ok(sobj);
    }
    Err("Failed to create stellar object!".to_string())
}
