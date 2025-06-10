
use glam::Vec2;
use log::info;
use spacetimedb::{rand::Rng, spacetimedb_lib::operator::Op};

use crate::types::{items::*, sectors::SectorId, stellarobjects::{utility::create_sobj_internal, *}};

use super::{*};

pub fn create_asteroid(ctx: &ReducerContext, position: Vec2, sector: SectorId, item: ItemDefinitionId, resource_amount: u16) -> Option<Asteroid> {
  let dsl = dsl(ctx);

  let gfx_key = format!("asteroid.{}", ctx.rng().gen_range(1..=5)).to_string();

  let sobj = create_sobj_internal(ctx,
    StellarObjectKinds::Asteroid, 
    &sector, 
    StellarObjectTransformInternal::default().from_vec2(position));
  if sobj.is_err() {
    info!("ERROR: Couldn't create stellar object for asteroid: {}", sobj.unwrap_err());
    return None;
  }

  match dsl.create_asteroid(
    &sobj.unwrap(),
    sector,
    16.0,
    item,
    resource_amount,
    resource_amount,
    Some(gfx_key)) {
    Ok(ast) => Some(ast),
    Err(e) => {
      info!("ERROR: Failed to create asteroid: {}", e);
      None
    }
  }
}