use crate::tables::sectors::SectorUpkeepTimer;

pub mod asteroid_fields;

/// Scheduled reducer that performs sector maintenance and upkeep tasks.
/// Runs every 60 seconds to ensure asteroid sectors have proper upkeep timers.
#[spacetimedb::reducer]
pub fn sector_upkeep(ctx: &ReducerContext, timer: SectorUpkeepTimer) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    // If a sector has an asteroid_sector entry associated with it, then update it
    for sector in dsl.get_all_asteroid_sectors() {
        asteroid_sector_upkeep(&sector);
    }

    // Do other sector upkeep stuff here.

    Ok(())
}
