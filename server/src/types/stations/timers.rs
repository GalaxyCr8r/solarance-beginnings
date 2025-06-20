use std::{f32::consts::PI, time::Duration};
use glam::Vec2;
use spacetimedb::*;
use spacetimedsl::*;

#[dsl(plural_name = station_production_schedules)]
#[table(name = station_production_schedule, scheduled(process_station_production_tick))]
pub struct StationProductionSchedule {
    #[primary_key]
    pub station_id: u64, // FK to SpaceStation
    pub scheduled_at: ScheduleAt, // Periodic (e.g., every minute or 5 minutes)
    pub last_processed_timestamp: Timestamp,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let _dsl = dsl(ctx);

    Ok(())
}

//////////////////////////////////////////////////////////////
// Reducers
//////////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn process_station_production_tick(ctx: &ReducerContext, _timer: StationProductionSchedule) -> Result<(), String> {
    let _dsl = dsl(ctx);
    
    Err("Not implemented".to_string())
}