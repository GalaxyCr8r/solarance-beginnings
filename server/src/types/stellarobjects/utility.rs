use std::f32::consts::PI;

use glam::Vec2;
use spacetimedb::ReducerContext;

use crate::types::sectors::SectorId;

use super::*;

pub fn create_sobj_vec2(
    ctx: &ReducerContext,
    kind: StellarObjectKinds,
    sector_id: &SectorId,
    position: Vec2
) -> Result<StellarObject, String> {
    let transform = StellarObjectTransformInternal {
        x: position.x,
        y: position.y,
        rotation_radians: 0.0, // Default rotation
        id: 0, // Default id
    };

    create_sobj_internal(ctx, kind, sector_id, transform)
}

pub fn create_sobj_internal(
    ctx: &ReducerContext,
    kind: StellarObjectKinds,
    sector_id: &SectorId,
    transform: StellarObjectTransformInternal
) -> Result<StellarObject, String> {
    let dsl = dsl(ctx);

    let sobj = dsl.create_stellar_object(kind, sector_id)?;

    let _ = dsl.create_sobj_internal_transform(
        &sobj,
        transform.x,
        transform.y,
        transform.rotation_radians
    )?;
    let _ = dsl.create_sobj_velocity(&sobj, 0.0, 0.0, 0.0, None)?;

    //spacetimedb::log::info!("Created stellar object #{}!", sobj.id);
    return Ok(sobj);
}

pub fn create_sobj_pos(
    ctx: &ReducerContext,
    kind: StellarObjectKinds,
    sector_id: &SectorId,
    x: f32,
    y: f32
) -> Result<StellarObject, String> {
    let dsl = dsl(ctx);

    let sobj = dsl.create_stellar_object(kind, sector_id)?;

    let _ = dsl.create_sobj_internal_transform(&sobj, x, y, 0.0)?;
    let _ = dsl.create_sobj_velocity(&sobj, 0.0, 0.0, 0.0, None)?;

    //spacetimedb::log::info!("Created stellar object #{}!", sobj.id);
    return Ok(sobj);
}

/// Creates a stellar object
pub fn create_sobj_with_random_velocity(
    ctx: &ReducerContext,
    kind: StellarObjectKinds,
    sector_id: &SectorId,
    x: f32,
    y: f32,
    velocity: f32,
    auto_dampen: Option<f32>
) -> Result<StellarObject, String> {
    let dsl = dsl(ctx);

    let sobj = dsl.create_stellar_object(kind, sector_id)?;

    let _transform = dsl.create_sobj_internal_transform(&sobj, x, y, 0.0)?;
    let random_angle = Vec2::from_angle(ctx.rng().gen_range(0.0..2.0 * PI)) * velocity;
    let _velocity = dsl.create_sobj_velocity(
        &sobj,
        random_angle.x,
        random_angle.y,
        ctx.rng().gen_range(random_angle.to_angle()..2.1 * PI),
        auto_dampen
    )?;

    //spacetimedb::log::info!("Created stellar object #{}!", sobj.id);
    return Ok(sobj);
}

pub fn get_distance(a_sobj_id: &StellarObjectId, b_sobj_id: &StellarObjectId) -> Option<f32> {
    None
}
