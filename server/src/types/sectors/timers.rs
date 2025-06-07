use std::time::Duration;

use spacetimedb::*;
use spacetimedsl::dsl;

use crate::types::{asteroids::timers::{CreateAsteroidSectorUpkeepTimerRow, GetAsteroidSectorUpkeepTimerRowsBySectorId}, utility::try_server_only};

use super::GetAllAsteroidSectorRows;

#[dsl(plural_name = sector_upkeep_timers)]
#[spacetimedb::table(name = sector_upkeep_timer, scheduled(sector_upkeep))]
pub struct SectorUpkeepTimer {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx); // Waiting for DSL implementation of timers

    dsl.create_sector_upkeep_timer(spacetimedb::ScheduleAt::Interval(Duration::from_secs(60).into()))?;

    Ok(())
}

//////////////////////////////////////////////////////////////
// Reducers
//////////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn sector_upkeep(ctx: &ReducerContext, timer: SectorUpkeepTimer) -> Result<(), String> {
  try_server_only(ctx)?;
  let dsl = dsl(ctx);

  // If a sector has an asteroid_sector entry associated with it, make sure it has an asteroid_sector_upkeep_timer row.
  for sector in dsl.get_all_asteroid_sectors() {
    //if dsl.get_asteroid_sector_upkeep_timers_by_sector_id(&sector) {
    if dsl.get_asteroid_sector_upkeep_timers_by_sector_id(sector.get_id()).count() == 0 {
      dsl.create_asteroid_sector_upkeep_timer(spacetimedb::ScheduleAt::Interval(Duration::from_secs(6 * 6).into()), sector.get_id())?; // TODO revert to 60*60
    }
  }

  // Do other sector upkeep stuff here.

  Ok(())
}