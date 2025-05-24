use std::time::Duration;

use spacetimedb::*;
use spacetimedsl::dsl;

#[dsl(plural_name = asteroid_sector_upkeep_timers)]
#[spacetimedb::table(name = asteroid_sector_upkeep_timer, scheduled(asteroid_sector_upkeep))]
pub struct AsteroidSectorUpkeepTimer {
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

    dsl.create_asteroid_sector_upkeep_timer(spacetimedb::ScheduleAt::Interval(Duration::from_millis(750).into()));

    Ok(())
}

//////////////////////////////////////////////////////////////
// Reducers
//////////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn asteroid_sector_upkeep(ctx: &ReducerContext, timer: AsteroidSectorUpkeepTimer) -> Result<(), String> {

  Ok(())
}