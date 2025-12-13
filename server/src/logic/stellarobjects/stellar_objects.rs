use crate::tables::sectors::SectorId;
use crate::tables::stellarobjects::*;
use crate::utility::try_server_only;
use glam::Vec2;
use spacetimedb::{rand::Rng, *};
use spacetimedsl::*;
use std::f32::consts::PI;

/// Toggles a turn-left controller for a stellar object (used for testing/debugging).
/// Creates the controller if it doesn't exist, or removes it if it does.
#[spacetimedb::reducer]
pub fn create_turn_left_controller_for(
    ctx: &ReducerContext,
    sobj_id: StellarObjectId,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    try_server_only(&dsl)?;

    if let Ok(controller) = dsl.get_sobj_turn_left_controller_by_id(&sobj_id) {
        dsl.delete_sobj_turn_left_controller_by_id(controller.get_id())?;
        spacetimedb::log::info!("Deleted controller #{:?}", sobj_id.value());
    } else {
        let controller = dsl.create_sobj_turn_left_controller(sobj_id)?;
        spacetimedb::log::info!("Created controller #{}", controller.id());
    }
    Ok(())
}

/// Creates a new stellar object of the specified type in a given sector with initial transform.
/// This is a general-purpose reducer for creating any type of stellar object (ships, asteroids, stations, etc.).
#[spacetimedb::reducer]
pub fn create_stellar_object(
    ctx: &ReducerContext,
    kind: StellarObjectKinds,
    sector_id: SectorId,
    transform: StellarObjectTransformInternal,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    try_server_only(&dsl)?;

    match create_sobj_internal(&dsl, kind, &sector_id, transform) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Creates a random stellar object (ship) in the specified sector for testing purposes.
/// Generates random position and rotation within predefined bounds.
#[spacetimedb::reducer]
pub fn create_sobj_random(ctx: &ReducerContext, sector_id: u64) -> Result<(), String> {
    try_server_only(&dsl(ctx))?;

    create_stellar_object(
        ctx,
        StellarObjectKinds::Ship,
        SectorId::new(sector_id),
        StellarObjectTransformInternal::new(
            ctx.rng().gen_range(0.0..1024.0),
            ctx.rng().gen_range(0.0..512.0),
            ctx.rng()
                .gen_range(-std::f32::consts::PI..std::f32::consts::PI),
        ),
    )
}

pub fn create_sobj_vec2(
    dsl: &DSL,
    kind: StellarObjectKinds,
    sector_id: &SectorId,
    position: Vec2,
) -> Result<StellarObject, String> {
    let transform = StellarObjectTransformInternal::new(position.x, position.y, 0.0);

    create_sobj_internal(dsl, kind, sector_id, transform)
}

pub fn create_sobj_internal(
    dsl: &DSL,
    kind: StellarObjectKinds,
    sector_id: &SectorId,
    transform: StellarObjectTransformInternal,
) -> Result<StellarObject, String> {
    let sobj = dsl.create_stellar_object(kind, sector_id)?;

    dsl.create_sobj_internal_transform(
        &sobj,
        *transform.get_x(),
        *transform.get_y(),
        *transform.get_rotation_radians(),
    )?;
    dsl.create_sobj_velocity(&sobj, 0.0, 0.0, 0.0, None)?;

    //spacetimedb::log::info!("Created stellar object #{}!", sobj.id);
    return Ok(sobj);
}

pub fn create_sobj_pos(
    dsl: &DSL,
    kind: StellarObjectKinds,
    sector_id: &SectorId,
    x: f32,
    y: f32,
) -> Result<StellarObject, String> {
    let sobj = dsl.create_stellar_object(kind, sector_id)?;

    dsl.create_sobj_internal_transform(&sobj, x, y, 0.0)?;
    dsl.create_sobj_velocity(&sobj, 0.0, 0.0, 0.0, None)?;

    //spacetimedb::log::info!("Created stellar object #{}!", sobj.id);
    return Ok(sobj);
}

/// Creates a stellar object
pub fn create_sobj_with_random_velocity(
    dsl: &DSL,
    kind: StellarObjectKinds,
    sector_id: &SectorId,
    x: f32,
    y: f32,
    velocity: f32,
    auto_dampen: Option<f32>,
) -> Result<StellarObject, String> {
    let sobj = dsl.create_stellar_object(kind, sector_id)?;

    let _transform = dsl.create_sobj_internal_transform(&sobj, x, y, 0.0)?;
    let random_angle = Vec2::from_angle(dsl.ctx().rng().gen_range(0.0..2.0 * PI)) * velocity;
    dsl.create_sobj_velocity(
        &sobj,
        random_angle.x,
        random_angle.y,
        dsl.ctx().rng().gen_range(random_angle.to_angle()..2.1 * PI),
        auto_dampen,
    )?;

    //spacetimedb::log::info!("Created stellar object #{}!", sobj.id);
    return Ok(sobj);
}
