use std::time::Duration;

use spacetimedb::*;
use spacetimedsl::dsl;

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

    dsl.create_sector_upkeep_timer(spacetimedb::ScheduleAt::Interval(Duration::from_millis(750).into()));

    Ok(())
}

//////////////////////////////////////////////////////////////
// Reducers
//////////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn sector_upkeep(ctx: &ReducerContext, timer: SectorUpkeepTimer) -> Result<(), String> {
  // If a sector has an asteroid_sector entry associated with it, make sure it has an asteroid_sector_upkeep_timer row.

  // Do other sector upkeep stuff here.

  Ok(())
}