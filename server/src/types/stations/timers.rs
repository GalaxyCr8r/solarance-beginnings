use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::types::stations::utility::*;

use super::*;

#[dsl(plural_name = station_production_schedules)]
#[table(name = station_production_schedule, scheduled(process_station_production_tick))]
pub struct StationProductionSchedule {
    #[primary_key]
    #[wrapped(path = StationId)]
    /// FK to SpaceStation
    pub station_id: u64,
    pub scheduled_at: ScheduleAt, // Periodic (e.g., every minute or 5 minutes)

    pub last_processed_timestamp: Timestamp,
}

#[dsl(plural_name = station_status_schedules)]
#[table(name = station_status_schedule, scheduled(process_station_status_tick))]
pub struct StationStatusSchedule {
    #[primary_key]
    #[wrapped(path = StationId)]
    /// FK to SpaceStation
    pub station_id: u64,
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
pub fn process_station_production_tick(
    ctx: &ReducerContext,
    timer: StationProductionSchedule,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Iterate through each station's modules.
    let station = dsl.get_station_by_id(timer.get_station_id()).unwrap();
    for module in dsl.get_station_modules_by_station_id(timer.get_station_id()) {
        let wrapped_blueprint = dsl.get_station_module_blueprint_by_id(&module.blueprint_id);
        if wrapped_blueprint.is_none() {
            info!("WARNING Station Module Blueprint #{} does not exist. Station #{} is looking for it.", module.blueprint_id, timer.station_id);
            continue;
        }
        let blueprint = wrapped_blueprint.unwrap();

        match blueprint.category {
            StationModuleCategory::LogisticsAndStorage => {
                update_logistics_and_storage(ctx, &station, &module, &blueprint)
            }
            StationModuleCategory::ResourceProductionAndRefining => {
                update_resource_production_and_refining(ctx, &station, &module, &blueprint)
            }
            StationModuleCategory::ManufacturingAndAssembly => {
                update_manufacturing_and_assembly(ctx, &station, &module, &blueprint)
            }
            StationModuleCategory::ResearchAndDevelopment => {
                update_research_and_development(ctx, &station, &module, &blueprint)
            }
            StationModuleCategory::CivilianAndSupportServices => {
                update_civilian_and_support_services(ctx, &station, &module, &blueprint)
            }
            StationModuleCategory::DiplomacyAndFaction => {
                update_diplomacy_and_faction(ctx, &station, &module, &blueprint)
            }
            StationModuleCategory::DefenseAndMilitary => {
                update_defense_and_military(ctx, &station, &module, &blueprint)
            }
        }?
    }

    Err("Not implemented".to_string())
}

#[spacetimedb::reducer]
pub fn process_station_status_tick(
    ctx: &ReducerContext,
    _timer: StationStatusSchedule,
) -> Result<(), String> {
    let _dsl = dsl(ctx);

    Err("Not implemented".to_string())
}
