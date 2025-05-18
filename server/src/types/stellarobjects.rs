use log::info;
use spacetimedb::{
    client_visibility_filter,
    rand::Rng,
    Filter,
    Identity,
    ReducerContext,
    SpacetimeType
};
use spacetimedsl::{dsl, Wrapper};

use super::{common::{ is_server_or_owner, server_only }, sector::SectorLocationId};

#[derive(SpacetimeType, PartialEq)]
enum TransformResolution { // TODO Delete?
    Internal = 0, // Internal transform state
    High = 1, // Nearby objects
    Low = 2, // For out-of-sector or far-away objects
}

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum StellarObjectKinds {
    Ship,
    Asteroid,
    Station,
}

#[dsl(plural_name = stellar_objects)]
#[spacetimedb::table(name = stellar_object, public)]
pub struct StellarObject {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[index(btree)]
    pub kind: StellarObjectKinds,

    #[index(btree)]
    #[wrapped(path = crate::types::sector::SectorLocationId)]
    pub sector_id: u64, // FK: SectorLocation
}

#[dsl(plural_name = player_controlled_stellar_objects)]
#[spacetimedb::table(name = player_controlled_stellar_object, public)]
pub struct PlayerControlledStellarObject {
    #[primary_key]
    pub identity: Identity,

    #[unique]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK to Entity

    #[index(btree)]
    #[wrapped(path = crate::types::sector::SectorLocationId)]
    pub sector_id: u64 // FK to Sector ID - Only because actually referencing the player's stellar object would require three table hits.
}

// You can only see sector objects in your current sector. TODO: In the future, this might be expanded to include anyone in your faction.
#[client_visibility_filter]
const SO_OBJECT_FILTER: Filter = Filter::Sql(
    "SELECT o.* 
    FROM stellar_object o
    JOIN player_controlled_stellar_object p ON p.sector_id = o.sector_id
    WHERE p.identity = :sender"
);

#[dsl(plural_name = sobj_internal_transforms)]
#[spacetimedb::table(name = sobj_internal_transform)]
#[derive(Default)]
pub struct StellarObjectTransformInternal {
    #[primary_key]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

#[dsl(plural_name = sobj_velocities)]
#[spacetimedb::table(name = sobj_velocity, public)]
#[derive(Default)]
pub struct StellarObjectVelocity {
    #[primary_key]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

#[dsl(plural_name = sobj_hi_res_transforms)]
#[spacetimedb::table(name = sobj_hi_res_transform, public)]
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
#[spacetimedb::table(name = sobj_low_res_transform, public)]
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
#[spacetimedb::table(name = sobj_turn_left_controller)]
pub struct StellarObjectControllerTurnLeft {
    #[primary_key]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject
}

#[dsl(plural_name = sobj_player_windows)]
#[spacetimedb::table(name = sobj_player_window, public)]
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

#[client_visibility_filter]
const HR_OBJECT_FILTER: Filter = Filter::Sql(
    "SELECT o.* 
    FROM sobj_hi_res_transform o
    JOIN sobj_player_window w
    WHERE w.identity = :sender AND
          (o.x > w.tl_x AND 
          o.y > w.tl_y AND 
          o.x < w.br_x AND 
          o.y < w.br_y)"
);

#[client_visibility_filter]
const LR_OBJECT_FILTER: Filter = Filter::Sql(
    "SELECT o.* 
    FROM sobj_low_res_transform o
    JOIN sobj_player_window w
    WHERE w.identity = :sender AND
          (o.x <= w.tl_x OR 
          o.y <= w.tl_y OR 
          o.x >= w.br_x OR 
          o.y >= w.br_y)"
);

/// Impls ///

impl StellarObjectVelocity {
    // pub fn new(x: f32, y: f32) -> Self {
    //     Self { x, y }
    // }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 { x: self.x, y: self.y }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectVelocity {
        StellarObjectVelocity { x: vec.x, y: vec.y, ..*self }
    }
}

impl StellarObjectTransformInternal {
    // pub fn new(x: f32, y: f32) -> Self {
    //     Self { x, y }
    // }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 { x: self.x, y: self.y }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectTransformInternal {
        StellarObjectTransformInternal { x: vec.x, y: vec.y, ..*self }
    }
}

/// Reducers ///

#[spacetimedb::reducer]
pub fn create_sobj_player_window_from(ctx: &ReducerContext, sobj_id: u64) -> Result<(), String> {
    let dsl = dsl(ctx);
    
    // Find who owns the object, if any
    let mut owning_player = None;
    for controlled in dsl.get_all_player_controlled_stellar_objects() {
        if controlled.sobj_id == sobj_id {
            owning_player = Some(controlled);
            break;
        }
    }
    if owning_player.is_none() {
        return Err("Couldn't find owning player to create player window".to_string());
    }

    // Create the window for the object
    create_sobj_player_window_for(ctx, owning_player.unwrap())
}

#[spacetimedb::reducer]
pub fn create_sobj_player_window_for(ctx: &ReducerContext, controlled_sobj: PlayerControlledStellarObject) -> Result<(), String> {
    let dsl = dsl(ctx);

    dsl.create_sobj_player_window(
        controlled_sobj.identity, 
        controlled_sobj.get_sobj_id(), 
        4000.0,
        2000.0,
        -2000.0,
        -2000.0,
        2000.0,
        2000.0)?;
    info!("Created player window for {} and object #{}!", controlled_sobj.identity.to_abbreviated_hex().to_string(), controlled_sobj.sobj_id);
    Ok(())
}

#[spacetimedb::reducer]
pub fn create_turn_left_controller_for(ctx: &ReducerContext, sobj_id: StellarObjectId) -> Result<(), String> {
    let dsl = dsl(ctx);

    if let Some(controller) = dsl.get_sobj_turn_left_controller_by_sobj_id(&sobj_id) {
        dsl.delete_sobj_turn_left_controller_by_sobj_id(controller.get_sobj_id());
        spacetimedb::log::info!("Deleted controller #{:?}", sobj_id.value);
    } else {
        let controller = dsl.create_sobj_turn_left_controller(sobj_id)?;
        spacetimedb::log::info!("Created controller #{}", controller.sobj_id);
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn create_stellar_object(
    ctx: &ReducerContext,
    kind: StellarObjectKinds,
    sector_id: SectorLocationId,
    transform: StellarObjectTransformInternal
) -> Result<(), String> {
    server_only(ctx);

    match create_sobj_internal(ctx, kind, sector_id, transform) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[spacetimedb::reducer]
pub fn create_sobj_random(ctx: &ReducerContext, sector_id: u64) -> Result<(), String> {
    server_only(ctx);

    create_stellar_object(
        ctx,
        StellarObjectKinds::Ship,
        SectorLocationId::new(sector_id),
        StellarObjectTransformInternal {
            sobj_id: 0,
            x: ctx.rng().gen_range(0.0..1024.0),
            y: ctx.rng().gen_range(0.0..512.0),
            rotation_radians: ctx.rng().gen_range(-std::f32::consts::PI..std::f32::consts::PI),
        }
    )
}

/// Called by clients to update their ships. Will limit the acceleration and etc.
#[spacetimedb::reducer]
pub fn update_sobj_velocity(
    ctx: &ReducerContext,
    velocity: StellarObjectVelocity
) -> Result<(), String> {
    let dsl = dsl(ctx);

    is_server_or_owner(ctx, velocity.get_sobj_id())?;

    let mut update_velocity = velocity.clone();
    
    match dsl.get_sobj_velocity_by_sobj_id(velocity.get_sobj_id()) {
        Some(prev_velocity) => {
            // Check if the acceleration required for the velocity change is too high
            let acceleration = (velocity.to_vec2() - prev_velocity.to_vec2()).length();
            if acceleration > 2.0 {
                //// TODO: Make this variable per ship type
                //log::info!("Acceleration too high! {:?}", acceleration);

                // Reduce the acceleration down                
                update_velocity = update_velocity.from_vec2(
                    prev_velocity.to_vec2() +
                    (update_velocity.to_vec2() - prev_velocity.to_vec2()).normalize() * 2.0);
            }

            // Check if the absolute velocity is too fast for the ship.
            if update_velocity.to_vec2().length() > 50.0 {
                //// TODO: Make this variable per ship type
                //log::info!("Velocity too high! {:?}", update_velocity.to_vec2().length());

                // Reduce the velocity down
                let new_velocity = update_velocity.to_vec2().normalize() * 50.0;
                update_velocity = update_velocity.from_vec2(new_velocity);
            }
        }
        None => {
            return Err("Stellar object's velocity table entry was not found!".to_string());
        }
    }

    if let Err(e) = dsl.update_sobj_velocity_by_sobj_id(update_velocity) {
        return Err(e.to_string())
    }
    Ok(())
}

/// Helper Functions ///

pub fn create_sobj_internal(
    ctx: &ReducerContext,
    kind: StellarObjectKinds,
    sector_id: SectorLocationId,
    transform: StellarObjectTransformInternal
) -> Result<StellarObject, String> {
    let dsl = dsl(ctx);

    let sobj = dsl.create_stellar_object(kind, sector_id)?;
    
    let _ = dsl.create_sobj_internal_transform(&sobj, transform.x, transform.y, transform.rotation_radians);
    let _ = dsl.create_sobj_velocity(&sobj, 0.0, 0.0, 0.0);

    spacetimedb::log::info!("Created stellar object #{}!", sobj.id);
    return Ok(sobj);
}
