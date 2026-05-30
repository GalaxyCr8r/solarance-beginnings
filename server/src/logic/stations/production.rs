use crate::*;
use spacetimedb::{log::info, *};

use super::*;

#[dsl(plural_name = station_production_schedules, method(update = true))]
#[spacetimedb::table(accessor = station_production_schedule, scheduled(station_production_schedule_reducer))]
pub struct StationProductionSchedule {
    #[primary_key]
    #[use_wrapper(StationId)]
    id: u64,
    pub scheduled_at: spacetimedb::ScheduleAt,
    pub last_processed_timestamp: spacetimedb::Timestamp,
}

#[spacetimedb::reducer]
pub fn station_production_schedule_reducer(ctx: &ReducerContext, timer: StationProductionSchedule) {
    let dsl = dsl(ctx);
    if let Err(e) = process_station_production_tick(&dsl, timer.get_id()) {
        spacetimedb::log::error!("Station production tick failed for station {}: {}", timer.get_id(), e);
    }
}

//////////////////////////////////////////////////////////////

/// Processes production for all modules in a station.
/// Handles resource production, manufacturing, logistics, and other station module operations.
pub fn process_station_production_tick<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station_id: StationId,
) -> Result<(), String> {
    // Get the station
    let station = dsl.get_station_by_id(&station_id)?;
    let modules: Vec<_> = dsl.get_station_modules_by_station_id(&station_id).collect();

    // info!(
    //     "Processing production tick for station #{}: {}",
    //     timer.id, station.name
    // );
    // info!(
    //     "Station {} has {} modules to process",
    //     timer.id,
    //     modules.len()
    // );

    // Iterate through each station's modules
    for module in modules {
        let wrapped_blueprint = dsl.get_station_module_blueprint_by_id(&module.get_blueprint());
        if wrapped_blueprint.is_err() {
            info!(
                "WARNING Station Module Blueprint #{} does not exist. Station #{} is looking for it.",
                module.get_blueprint(),
                &station_id
            );
            continue;
        }
        let blueprint = wrapped_blueprint.unwrap();

        info!(
            "Processing module {} of type {:?}",
            module.get_id(),
            blueprint.get_category()
        );

        let result = match blueprint.get_category() {
            StationModuleCategory::LogisticsAndStorage => {
                update_logistics_and_storage(&dsl, &station, &module, &blueprint)
            }
            StationModuleCategory::ResourceProductionAndRefining => {
                update_resource_production_and_refining(&dsl, &station, &module, &blueprint)
            }
            StationModuleCategory::ManufacturingAndAssembly => {
                update_manufacturing_and_assembly(&dsl, &station, &module, &blueprint)
            }
            StationModuleCategory::ResearchAndDevelopment => {
                update_research_and_development(&dsl, &station, &module, &blueprint)
            }
            StationModuleCategory::CivilianAndSupportServices => {
                update_civilian_and_support_services(&dsl, &station, &module, &blueprint)
            }
            StationModuleCategory::DiplomacyAndFaction => {
                update_diplomacy_and_faction(&dsl, &station, &module, &blueprint)
            }
            StationModuleCategory::DefenseAndMilitary => {
                update_defense_and_military(&dsl, &station, &module, &blueprint)
            }
        };

        if let Err(e) = result {
            info!("Error processing module {}: {}", module.get_id(), e);
        }
    }

    info!(
        "Completed production tick for station #{}: {} (Sector ID#:{})",
        &station_id,
        station.get_name(),
        station.get_sector_id()
    );
    Ok(())
}
