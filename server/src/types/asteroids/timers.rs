use std::{f32::consts::PI, time::Duration};

use glam::Vec2;
use spacetimedb::{rand::Rng, *};
use spacetimedsl::*;

use crate::types::{items::ItemDefinitionId, sectors::{GetAsteroidSectorRowOptionById}};

use super::{utility::create_asteroid, GetAsteroidRowsByCurrentSectorId};

#[dsl(plural_name = asteroid_sector_upkeep_timers)]
#[spacetimedb::table(name = asteroid_sector_upkeep_timer, scheduled(asteroid_sector_upkeep))]
pub struct AsteroidSectorUpkeepTimer {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[index(btree)] // To find asteroids in a specific sector
    #[wrapped(path = crate::types::sectors::SectorId)]
    pub sector_id: u64, // FK to Sector.id
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx); // Waiting for DSL implementation of timers

    // Timers are created by Sector Upkeep

    Ok(())
}

//////////////////////////////////////////////////////////////
// Reducers
//////////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn asteroid_sector_upkeep(ctx: &ReducerContext, timer: AsteroidSectorUpkeepTimer) -> Result<(), String> {
  let dsl = dsl(ctx);
  
  if let Some(asteroid_sector) = dsl.get_asteroid_sector_by_id(timer.get_sector_id()) {

    if dsl.get_asteroids_by_current_sector_id(timer.get_sector_id()).count() < (asteroid_sector.get_sparseness() * 25).into() { // 100
      let field = asteroid_sector.cluster_extent;
      let pos = match asteroid_sector.cluster_inner {
        Some(inner_extent) => {
          let dist = ctx.rng().gen_range(inner_extent..field); // Pick a distance between inner and outer bounds.

          Vec2::from_angle(ctx.rng().gen_range(0.0..(2.0*PI))) * dist
        },
        None => Vec2::from_angle(ctx.rng().gen_range(0.0..(2.0*PI))) * field, // Pick a random angle and then multiply it by the extent of the cluster.
      };

      let item = ItemDefinitionId::new(match ctx.rng().gen_range(0..100) {
        0 .. 75 => 1001,
        75 .. 99 => 1002,
        _ => 1003,
      });

      let amount = ctx.rng().gen_range(500..2000);

      create_asteroid(ctx, pos, timer.get_sector_id(), item, amount);
    } 
  } else {
    return Err("Failed to find AsteroidSector".to_string());
  }

  Ok(())
}