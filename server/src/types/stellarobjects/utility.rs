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
        sobj_id: 0, // Default id
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
    
    let _ = dsl.create_sobj_internal_transform(&sobj, transform.x, transform.y, transform.rotation_radians)?;
    let _ = dsl.create_sobj_velocity(&sobj, 0.0, 0.0, 0.0)?;

    spacetimedb::log::info!("Created stellar object #{}!", sobj.id);
    return Ok(sobj);
}