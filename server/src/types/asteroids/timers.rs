use std::time::Duration;

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
  let asteroid_sector = dsl.get_asteroid_sector_by_id(timer.get_sector_id());
  if asteroid_sector.is_none() {
    return Err("Failed to find AsteroidSector".to_string());
  }

  if dsl.get_asteroids_by_current_sector_id(timer.get_sector_id()).count() < (asteroid_sector.unwrap().get_sparseness() * 100).into() {
    let field = 1024.0;//8192.0;
    let pos = Vec2::new(
      ctx.rng().gen_range(-field..field),
      ctx.rng().gen_range(-field..field),
    );
    create_asteroid(ctx, pos, timer.get_sector_id(), ItemDefinitionId::new(1001), 1000);
  }

  Ok(())
}