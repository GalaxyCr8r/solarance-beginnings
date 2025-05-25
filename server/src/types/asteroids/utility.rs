
use glam::Vec2;
use log::info;

use crate::types::{items::*, sectors::SectorId, stellarobjects::{utility::create_sobj_internal, *}};

use super::{*};

pub fn create_asteroid(ctx: &ReducerContext, position: Vec2, sector: SectorId, item: ItemDefinitionId, resource_amount: u32) -> Option<Asteroid> {
  let dsl = dsl(ctx);

  // TODO Pick a random asteroid image
  let gfx_key = "asteroid.1".to_string();

  let sobj = create_sobj_internal(ctx,
    StellarObjectKinds::Asteroid, 
    &sector, 
    StellarObjectTransformInternal::default().from_vec2(position));
  if sobj.is_err() {
    info!("ERROR: Couldn't create stellar object for asteroid: {}", sobj.unwrap_err());
    return None;
  }

  match dsl.create_asteroid(&sobj.unwrap(), sector, 16.0, item, resource_amount, resource_amount, Some(gfx_key)) {
    Ok(ast) => Some(ast),
    Err(e) => {
      info!("ERROR: Failed to create asteroid: {}", e);
      None
    }
  }
}