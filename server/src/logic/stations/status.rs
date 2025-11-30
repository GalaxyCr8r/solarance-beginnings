use crate::tables::stations::StationId;
use spacetimedb::{table, ReducerContext, ScheduleAt, Timestamp};
use spacetimedsl::dsl;

#[dsl(plural_name = station_status_schedules)]
#[table(name = station_status_schedule, scheduled(process_station_status_tick))]
pub struct StationStatusSchedule {
    #[primary_key]
    #[use_wrapper(path = StationId)]
    /// FK to SpaceStation
    id: u64,
    pub scheduled_at: ScheduleAt, // Periodic (e.g., every minute or 5 minutes)

    pub last_processed_timestamp: Timestamp,
}

//////////////////////////////////////////////////////////////

/// Scheduled reducer that processes station status updates and maintenance.
/// Currently not implemented - placeholder for future station health/status monitoring.
#[spacetimedb::reducer]
pub fn process_station_status_tick(
    ctx: &ReducerContext,
    _timer: StationStatusSchedule,
) -> Result<(), String> {
    let _dsl = dsl(ctx);

    // TODO: Implement station shields
    //Err("Not implemented".to_string())

    Ok(())
}
