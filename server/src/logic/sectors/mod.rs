use log::warn;
use spacetimedb::ReducerContext;
use crate::spacetimedsl::prelude::*;

use crate::{logic::sectors::asteroid_fields::*, tables::sectors::*, utility::try_server_only};

pub mod asteroid_fields;

/////////////////////////////////////////////////////////////
/// Timers
///

#[spacetimedsl::dsl(plural_name = sector_upkeep_timers, method(update = false))]
#[spacetimedb::table(accessor = sector_upkeep_timer, scheduled(sector_upkeep))]
pub struct SectorUpkeepTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

/////////////////////////////////////////////////////////////
/// Timer Reducers

/// Scheduled reducer that performs sector maintenance and upkeep tasks.
/// Runs every 60 seconds to ensure asteroid sectors have proper upkeep timers.
#[spacetimedb::reducer]
pub fn sector_upkeep(ctx: &ReducerContext, _timer: SectorUpkeepTimer) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    // If a sector has an asteroid_sector entry associated with it, then update it
    for sector in dsl.get_all_asteroid_sectors() {
        if let Err(e) = asteroid_sector_upkeep(&dsl, &sector.get_id()) {
            warn!(
                "asteroid_sector_upkeep failed for sector {}: {}",
                sector.get_id().value(),
                e
            );
        }
    }

    // Do other sector upkeep stuff here.

    Ok(())
}
