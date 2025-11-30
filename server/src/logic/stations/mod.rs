use crate::{
    definitions::item_types::*,
    logic::stations::module_types::{
        manufacturing::{self, *},
        refineries::{self as refinery, *},
        solar_arrays::{self as solar_array, *},
        trading_port,
    },
    tables::{
        factions::FactionId, items::*, sectors::Sector, stations::*, stellarobjects::StellarObject,
    },
    utility::try_server_only,
};

use log::info;
use spacetimedb::*;
use spacetimedsl::*;
use std::time::Duration;

pub mod buy_and_sell;
pub mod module_types;
pub mod production;
pub mod status;

///////////////////////////////////////////////////////////////////////////////////////////
/// Utilties

/// Admin reducer to create station timers for the given station ID.
/// Sets up both production and status schedules for the station.
#[spacetimedb::reducer]
pub fn add_station_timers(ctx: &ReducerContext, station_id: u64) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;
    let station_id = StationId::new(station_id);

    // Verify the station exists
    let _station = dsl.get_station_by_id(&station_id)?;

    // Check if production schedule already exists
    if dsl
        .get_station_production_schedule_by_id(&station_id)
        .is_ok()
    {
        info!(
            "Station production schedule already exists for station {}",
            station_id
        );
    } else {
        // Set up station production schedule (every 30 seconds)
        dsl.create_station_production_schedule(
            &station_id,
            ScheduleAt::Interval(Duration::from_secs(30).into()), // TODO: Make this dependant on a GlobalConfig value
            ctx.timestamp,
        )?;
    }

    // Check if status schedule already exists
    if dsl.get_station_status_schedule_by_id(&station_id).is_ok() {
        info!(
            "Station status schedule already exists for station {}",
            station_id
        );
    } else {
        // Set up station status schedule (every 10 seconds)
        dsl.create_station_status_schedule(
            &station_id,
            ScheduleAt::Interval(Duration::from_secs(10).into()), // TODO: Make this dependant on a GlobalConfig value
            ctx.timestamp,
        )?;
    }

    info!("Created station timers for station {}", station_id);

    Ok(())
}

/// Type alias for module creation functions
pub type ModuleCreationFn = Box<dyn Fn(&DSL, &Station) -> Result<(), String>>;

/// Helper function to create a basic trading module
pub fn create_trading_module() -> ModuleCreationFn {
    Box::new(|dsl, station| trading_port::create_basic_bazaar(dsl, station, false))
}

/// Helper function to create a basic refinery module for iron ore
pub fn create_iron_refinery_module() -> ModuleCreationFn {
    Box::new(|dsl, station| {
        create_basic_refinery_module(
            dsl,
            station,
            false,
            ItemDefinitionId::new(ITEM_IRON_ORE),
            ItemDefinitionId::new(ITEM_IRON_INGOT),
            None,
        )
    })
}

/// Helper function to create a basic refinery module for ice ore
pub fn create_ice_refinery_module() -> ModuleCreationFn {
    Box::new(|dsl, station| {
        create_basic_refinery_module(
            dsl,
            station,
            false,
            ItemDefinitionId::new(ITEM_ICE_ORE),
            ItemDefinitionId::new(ITEM_WATER),
            None,
        )
    })
}

/// Helper function to create a basic refinery module for silicon ore
pub fn create_silicon_refinery_module() -> ModuleCreationFn {
    Box::new(|dsl, station| {
        create_basic_refinery_module(
            dsl,
            station,
            false,
            ItemDefinitionId::new(ITEM_SILICON_ORE),
            ItemDefinitionId::new(ITEM_SILICON_RAW),
            None,
        )
    })
}

/// Helper function to create a station with modules and automatically set up schedules
pub fn create_station_with_modules(
    dsl: &DSL,
    size: StationSize,
    sector: &Sector,
    sobj: &StellarObject,
    owner_faction_id: FactionId,
    name: &str,
    description: Option<String>,
    module_creators: Vec<ModuleCreationFn>,
) -> Result<Station, String> {
    // Create the station
    let station = dsl.create_station(size, sector, sobj, owner_faction_id, name, description)?;

    // Create all modules
    for module_creator in module_creators {
        module_creator(dsl, &station)?;
    }

    // Set up station production schedule (every 30 seconds) TODO Tie this to GlobalConfig
    dsl.create_station_production_schedule(
        station.get_id(),
        ScheduleAt::Interval(Duration::from_secs(30).into()),
        dsl.ctx().timestamp,
    )?;

    // Set up station status schedule (every 60 seconds) TODO Tie this to GlobalConfig
    dsl.create_station_status_schedule(
        station.get_id(),
        ScheduleAt::Interval(Duration::from_secs(10).into()),
        dsl.ctx().timestamp,
    )?;

    // Verify station invariants
    verify(dsl, station.clone())?;

    Ok(station)
}

/// Verify the invariants of this class that Rust cannot guarantee due to the database limitations.
/// Should be called after modifying a station.
pub fn verify(dsl: &DSL, station: Station) -> Result<(), String> {
    // Verify the station does not have more modules than it should.
    let current_module_count = dsl
        .get_station_modules_by_station_id(station.get_id())
        .count();
    let max_modules = station.size.modules() as usize;

    if current_module_count > max_modules {
        return Err(format!(
            "Too many station modules attached. Found {} modules but station size {:?} only allows {} modules.",
            current_module_count,
            station.size,
            max_modules
        ));
    }

    Ok(())
}

/// LogisticsAndStorage,
pub fn update_logistics_and_storage(
    dsl: &DSL,
    _station: &Station,
    module: &StationModule,
    _blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    // Update cached prices for all inventory items in this module
    for mut inventory_item in dsl.get_station_module_inventory_items_by_module_id(module.get_id()) {
        if let Ok(item_def) = dsl.get_item_definition_by_id(inventory_item.get_resource_item_id()) {
            let current_price = inventory_item.calculate_current_price(&item_def);
            //info!("    Old Value : {}c", inventory_item.cached_price);
            inventory_item.set_cached_price(current_price);
            dsl.update_station_module_inventory_item_by_id(inventory_item)?;
        }
    }

    Ok(())
}

/// ResourceProductionAndRefining,
pub fn update_resource_production_and_refining(
    dsl: &DSL,
    _station: &Station,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    // Calculate time elapsed since last update (assuming 30 second intervals)
    let time_elapsed_hours = 30.0 / 3600.0; // 30 seconds in hours

    match blueprint.specific_type {
        StationModuleSpecificType::RefineryBasicOre => {
            // Handle refinery modules
            if let Ok(refinery) = dsl.get_refinery_module_by_id(module.get_id()) {
                let production_result = refinery::timers::calculate_refinery_production(
                    dsl,
                    &refinery,
                    time_elapsed_hours,
                )?;

                refinery::timers::apply_refinery_production(dsl, &refinery, &production_result)?;

                spacetimedb::log::info!(
                    "Refinery module {} produced {:.2} ingots, consumed {:.2} ore",
                    module.id,
                    production_result.ingots_produced,
                    production_result.ore_consumed
                );
            }
        }
        StationModuleSpecificType::FarmStandard | StationModuleSpecificType::FarmLuxury => {
            // Handle farm modules
            if let Ok(farm) = dsl.get_farm_module_by_id(module.get_id()) {
                let production_result =
                    farm::timers::calculate_farm_production(dsl, &farm, time_elapsed_hours)?;

                farm::timers::apply_farm_production(dsl, &farm, &production_result)?;

                spacetimedb::log::info!(
                    "Farm module {} produced {:.2} food units",
                    module.id,
                    production_result.food_produced
                );
            }
        }
        StationModuleSpecificType::SolarArray => {
            // Handle solar array modules
            if let Ok(solar_array) = dsl.get_solar_array_module_by_id(module.get_id()) {
                let production_result = solar_array::timers::calculate_solar_array_production(
                    dsl,
                    &solar_array,
                    time_elapsed_hours,
                )?;

                solar_array::timers::apply_solar_array_production(
                    dsl,
                    &solar_array,
                    &production_result,
                )?;

                spacetimedb::log::info!(
                    "Solar array module {} produced {:.2} energy cells",
                    module.id,
                    production_result.energy_cells_produced
                );
            }
        }
        _ => {
            // Not a resource production/refining module, skip
        }
    }

    Ok(())
}

/// ManufacturingAndAssembly,
pub fn update_manufacturing_and_assembly(
    dsl: &DSL,
    _station: &Station,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    // Calculate time elapsed since last update (assuming 30 second intervals)
    let time_elapsed_seconds = 30.0; // 30 seconds
    let manufacturing = dsl.get_manufacturing_module_by_id(module.get_id())?;
    info!(
        "Recipe: {:?} - Type: {:?}",
        manufacturing
            .get_current_recipe_id()
            .map(|r| dsl.get_production_recipe_definition_by_id(r)),
        blueprint.specific_type
    );

    match blueprint.specific_type {
        StationModuleSpecificType::FactoryBasicComponents
        | StationModuleSpecificType::FactoryAdvancedComponents => {
            // Handle manufacturing modules
            let production_result =
                calculate_manufacturing_production(dsl, &manufacturing, time_elapsed_seconds)?;

            info!("Production Result: {:?}", production_result);

            apply_manufacturing_production(dsl, &manufacturing, &production_result)?;
            if production_result.items_completed > 0 {
                spacetimedb::log::info!(
                    "Manufacturing module {} completed {} items, progress: {:.2}",
                    module.id,
                    production_result.items_completed,
                    production_result.progress_made
                );
            }
        }

        _ => {
            // Not a manufacturing/assembly module, skip
        }
    }

    Ok(())
}

/// ResearchAndDevelopment,
pub fn update_research_and_development(
    dsl: &DSL,
    _station: &Station,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    // Calculate time elapsed since last update (assuming 30 second intervals)
    let time_elapsed_hours = 30.0 / 3600.0; // 30 seconds in hours

    match blueprint.specific_type {
        StationModuleSpecificType::Laboratory => {
            // Handle laboratory modules
            if let Ok(laboratory) = dsl.get_laboratory_module_by_id(module.get_id()) {
                let production_result = laboratory::timers::calculate_laboratory_production(
                    dsl,
                    &laboratory,
                    time_elapsed_hours,
                )?;

                laboratory::timers::apply_laboratory_production(
                    dsl,
                    &laboratory,
                    &production_result,
                )?;

                if production_result.fragments_produced > 0 {
                    spacetimedb::log::info!(
                        "Laboratory module {} produced {:.2} research fragments ({:.2} points)",
                        module.id,
                        production_result.fragments_produced,
                        production_result.research_points_produced
                    );
                }
            }
        }
        _ => {
            // Not a research/development module, skip
        }
    }

    Ok(())
}

/// CivilianAndSupportServices,
pub fn update_civilian_and_support_services(
    _dsl: &DSL,
    _station: &Station,
    _module: &StationModule,
    _blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    //
    Ok(())
}

/// DiplomacyAndFaction,
pub fn update_diplomacy_and_faction(
    _dsl: &DSL,
    _station: &Station,
    _module: &StationModule,
    _blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    //
    Ok(())
}

/// DefenseAndMilitary,
pub fn update_defense_and_military(
    _dsl: &DSL,
    _station: &Station,
    _module: &StationModule,
    _blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    //
    Ok(())
}
/// Helper function to create a basic food farm module
pub fn create_basic_farm_module() -> ModuleCreationFn {
    Box::new(|dsl, station| {
        farm::definitions::create_basic_food_farm(
            dsl,
            station,
            false,
            farm::FarmOutputQuality::Average,
        )
    })
}

/// Helper function to create a luxury food farm module
pub fn create_luxury_farm_module() -> ModuleCreationFn {
    Box::new(|dsl, station| {
        farm::definitions::create_basic_food_farm(
            dsl,
            station,
            false,
            farm::FarmOutputQuality::Luxury,
        )
    })
}

/// Helper function to create a basic laboratory module
pub fn create_basic_laboratory_module() -> ModuleCreationFn {
    Box::new(|dsl, station| {
        laboratory::definitions::create_basic_laboratory(
            dsl,
            station,
            false,
            laboratory::definitions::LaboratoryType::Basic,
        )
    })
}

/// Helper function to create an advanced laboratory module
pub fn create_advanced_laboratory_module() -> ModuleCreationFn {
    Box::new(|dsl, station| {
        laboratory::definitions::create_basic_laboratory(
            dsl,
            station,
            false,
            laboratory::definitions::LaboratoryType::Advanced,
        )
    })
}

/// Helper function to create a basic manufacturing module
pub fn create_basic_manufacturing_module() -> ModuleCreationFn {
    Box::new(|dsl, station| {
        manufacturing::definitions::create_basic_manufacturing_module(
            dsl,
            station,
            false,
            manufacturing::definitions::ManufacturingType::BasicFactory,
        )
    })
}

/// Helper function to create an advanced manufacturing module
pub fn create_advanced_manufacturing_module() -> ModuleCreationFn {
    Box::new(|dsl, station| {
        manufacturing::definitions::create_basic_manufacturing_module(
            dsl,
            station,
            false,
            manufacturing::definitions::ManufacturingType::AdvancedFactory,
        )
    })
}

/// Helper function to create a small solar array module
pub fn create_small_solar_array_module() -> ModuleCreationFn {
    Box::new(|dsl, station| {
        solar_array::definitions::create_basic_solar_array(
            dsl,
            station,
            false,
            solar_array::definitions::SolarArraySize::Small,
        )
    })
}

/// Helper function to create a large solar array module
pub fn create_large_solar_array_module() -> ModuleCreationFn {
    Box::new(|dsl, station| {
        solar_array::definitions::create_basic_solar_array(
            dsl,
            station,
            false,
            solar_array::definitions::SolarArraySize::Large,
        )
    })
}

/// Helper function to create a metal plate manufacturing module
pub fn create_metal_plate_module() -> ModuleCreationFn {
    Box::new(|dsl, station| {
        manufacturing::definitions::create_metal_plate_module(dsl, station, false)
    })
}
