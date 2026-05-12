use crate::{
    definitions::item_types::*,
    logic::stations::{
        module_types::{manufacturing::*, refineries::*, solar_arrays::*, trading_port},
        production::*,
        status::*,
    },
    tables::{factions::*, items::*, sectors::*, stations::*, stellarobjects::*},
    utility::*,
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

/// Type alias for module creation functions
pub type ModuleCreationFn<T> = Box<dyn Fn(&DSL<T>, &Station) -> Result<(), String>>;

/// Helper function to create a basic trading module
pub fn create_trading_module<T: spacetimedsl::WriteContext + 'static>() -> ModuleCreationFn<T> {
    Box::new(|dsl, station| trading_port::create_basic_bazaar(dsl, station, false))
}

/// Helper function to create a basic refinery module for iron ore
pub fn create_iron_refinery_module<T: spacetimedsl::WriteContext + 'static>() -> ModuleCreationFn<T>
{
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
pub fn create_ice_refinery_module<T: spacetimedsl::WriteContext + 'static>() -> ModuleCreationFn<T>
{
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
pub fn create_silicon_refinery_module<T: spacetimedsl::WriteContext + 'static>(
) -> ModuleCreationFn<T> {
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
pub fn create_station_with_modules<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    size: StationSize,
    sector: &Sector,
    sobj: &StellarObject,
    owner_faction_id: FactionId,
    name: &str,
    _description: Option<String>,
    position: solarance_shared::Vec2,
    rotation: f32,
    module_creators: Vec<ModuleCreationFn<T>>,
) -> Result<Station, String> {
    let station = dsl.create_station(CreateStation {
        size,
        sector_id: sector.get_id(),
        sobj_id: sobj.get_id(),
        owner_faction_id: FactionId::new(owner_faction_id.value()),
        name: name.to_string(),
        gfx_key: None,
        position,
        rotation,
    })?;

    // Create all modules
    for module_creator in module_creators {
        module_creator(dsl, &station)?;
    }

    // Set up station production schedule (every 30 seconds) TODO Tie this to GlobalConfig
    dsl.create_station_production_schedule(CreateStationProductionSchedule {
        id: station.get_id(),
        scheduled_at: ScheduleAt::Interval(Duration::from_secs(30).into()),
        last_processed_timestamp: dsl.ctx().timestamp()?,
    })?;

    // Set up station status schedule (every 60 seconds) TODO Tie this to GlobalConfig
    dsl.create_station_status_schedule(CreateStationStatusSchedule {
        id: station.get_id(),
        scheduled_at: ScheduleAt::Interval(Duration::from_secs(10).into()),
        last_processed_timestamp: dsl.ctx().timestamp()?,
    })?;

    // Verify station invariants
    verify(dsl, &station)?;

    Ok(station)
}

/// Verify the invariants of this class that Rust cannot guarantee due to the database limitations.
/// Should be called after modifying a station.
pub fn verify<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station: &Station,
) -> Result<(), String> {
    // Verify the station does not have more modules than it should.
    let current_module_count = dsl
        .get_station_modules_by_station_id(station.get_id())
        .count();
    let max_modules = station.get_size().max_module_amount() as usize;

    if current_module_count > max_modules {
        return Err(format!(
            "Too many station modules attached. Found {} modules but station size {:?} only allows {} modules.",
            current_module_count,
            station.get_size(),
            max_modules
        ));
    }

    Ok(())
}

/// LogisticsAndStorage,
pub fn update_logistics_and_storage<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    _station: &Station,
    module: &StationModule,
    _blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    // Update cached prices for all inventory items in this module
    for mut inventory_item in dsl.get_station_module_inventory_items_by_module_id(module.get_id()) {
        if let Ok(item_def) =
            dsl.get_item_definition_by_id(ItemDefinitionId::new(inventory_item.resource_item_id))
        {
            let current_price = inventory_item.calculate_current_price(&item_def);
            //info!("    Old Value : {}c", inventory_item.cached_price);
            inventory_item.set_cached_price(current_price);
            dsl.update_station_module_inventory_item_by_id(inventory_item)?;
        }
    }

    Ok(())
}

/// ResourceProductionAndRefining,
pub fn update_resource_production_and_refining<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    _station: &Station,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    // Calculate time elapsed since last update (assuming 30 second intervals)
    let time_elapsed_hours = 30.0 / 3600.0; // 30 seconds in hours

    match blueprint.get_specific_type() {
        StationModuleSpecificType::RefineryBasicOre => {
            // Handle refinery modules
            if let Ok(refinery) = dsl.get_refinery_module_by_id(module.get_id()) {
                let production_result =
                    calculate_refinery_production(dsl, &refinery, time_elapsed_hours)?;

                apply_refinery_production(dsl, &refinery, &production_result)?;

                spacetimedb::log::info!(
                    "Refinery module {} produced {:.2} ingots, consumed {:.2} ore",
                    module.get_id(),
                    production_result.ingots_produced,
                    production_result.ore_consumed
                );
            }
        }
        StationModuleSpecificType::FarmStandard | StationModuleSpecificType::FarmLuxury => {
            // TODO: Farm modules not yet implemented
            // Handle farm modules
            // if let Ok(farm) = dsl.get_farm_module_by_id(module.get_id()) {
            //     let production_result =
            //         farm::timers::calculate_farm_production(dsl, &farm, time_elapsed_hours)?;
            //
            //     farm::timers::apply_farm_production(dsl, &farm, &production_result)?;
            //
            //     spacetimedb::log::info!(
            //         "Farm module {} produced {:.2} food units",
            //         module.id,
            //         production_result.food_produced
            //     );
            // }
        }
        StationModuleSpecificType::SolarArray => {
            // Handle solar array modules
            if let Ok(solar_array) = dsl.get_solar_array_module_by_id(module.get_id()) {
                let production_result =
                    calculate_solar_array_production(dsl, &solar_array, time_elapsed_hours)?;

                apply_solar_array_production(dsl, &solar_array, &production_result)?;

                spacetimedb::log::info!(
                    "Solar array module {} produced {:.2} energy cells",
                    module.get_id(),
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
pub fn update_manufacturing_and_assembly<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
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
        blueprint.get_specific_type()
    );

    match blueprint.get_specific_type() {
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
                    module.get_id(),
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
pub fn update_research_and_development<T: spacetimedsl::WriteContext>(
    _dsl: &DSL<T>,
    _station: &Station,
    _module: &StationModule,
    blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    // Calculate time elapsed since last update (assuming 30 second intervals)
    let _time_elapsed_hours = 30.0 / 3600.0; // 30 seconds in hours

    match blueprint.get_specific_type() {
        StationModuleSpecificType::Laboratory => {
            // TODO: Laboratory modules not yet implemented
            // Handle laboratory modules
            // if let Ok(laboratory) = dsl.get_laboratory_module_by_id(module.get_id()) {
            //     let production_result = laboratory::timers::calculate_laboratory_production(
            //         dsl,
            //         &laboratory,
            //         time_elapsed_hours,
            //     )?;
            //
            //     laboratory::timers::apply_laboratory_production(
            //         dsl,
            //         &laboratory,
            //         &production_result,
            //     )?;
            //
            //     if production_result.fragments_produced > 0 {
            //         spacetimedb::log::info!(
            //             "Laboratory module {} produced {:.2} research fragments ({:.2} points)",
            //             module.id,
            //             production_result.fragments_produced,
            //             production_result.research_points_produced
            //         );
            //     }
            // }
        }
        _ => {
            // Not a research/development module, skip
        }
    }

    Ok(())
}

/// CivilianAndSupportServices,
pub fn update_civilian_and_support_services<T: spacetimedsl::WriteContext>(
    _dsl: &DSL<T>,
    _station: &Station,
    _module: &StationModule,
    _blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    //
    Ok(())
}

/// DiplomacyAndFaction,
pub fn update_diplomacy_and_faction<T: spacetimedsl::WriteContext>(
    _dsl: &DSL<T>,
    _station: &Station,
    _module: &StationModule,
    _blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    //
    Ok(())
}

/// DefenseAndMilitary,
pub fn update_defense_and_military<T: spacetimedsl::WriteContext>(
    _dsl: &DSL<T>,
    _station: &Station,
    _module: &StationModule,
    _blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    //
    Ok(())
}
// TODO: Farm modules not yet implemented
// /// Helper function to create a basic food farm module
// pub fn create_basic_farm_module() -> ModuleCreationFn {
//     Box::new(|dsl, station| {
//         farm::definitions::create_basic_food_farm(
//             dsl,
//             station,
//             false,
//             farm::FarmOutputQuality::Average,
//         )
//     })
// }
//
// /// Helper function to create a luxury food farm module
// pub fn create_luxury_farm_module() -> ModuleCreationFn {
//     Box::new(|dsl, station| {
//         farm::definitions::create_basic_food_farm(
//             dsl,
//             station,
//             false,
//             farm::FarmOutputQuality::Luxury,
//         )
//     })
// }

// TODO: Laboratory modules not yet implemented
// /// Helper function to create a basic laboratory module
// pub fn create_basic_laboratory_module() -> ModuleCreationFn {
//     Box::new(|dsl, station| {
//         laboratory::definitions::create_basic_laboratory(
//             dsl,
//             station,
//             false,
//             laboratory::definitions::LaboratoryType::Basic,
//         )
//     })
// }
//
// /// Helper function to create an advanced laboratory module
// pub fn create_advanced_laboratory_module() -> ModuleCreationFn {
//     Box::new(|dsl, station| {
//         laboratory::definitions::create_basic_laboratory(
//             dsl,
//             station,
//             false,
//             laboratory::definitions::LaboratoryType::Advanced,
//         )
//     })
// }

/// Helper function to create a basic manufacturing module
pub fn create_basic_manufacturing_module_fn<T: spacetimedsl::WriteContext + 'static>(
) -> ModuleCreationFn<T> {
    Box::new(|dsl, station| {
        create_basic_manufacturing_module(dsl, station, false, ManufacturingType::BasicFactory)
    })
}

/// Helper function to create an advanced manufacturing module
pub fn create_advanced_manufacturing_module<T: spacetimedsl::WriteContext + 'static>(
) -> ModuleCreationFn<T> {
    Box::new(|dsl, station| {
        create_basic_manufacturing_module(dsl, station, false, ManufacturingType::AdvancedFactory)
    })
}

/// Helper function to create a small solar array module
pub fn create_small_solar_array_module<T: spacetimedsl::WriteContext + 'static>(
) -> ModuleCreationFn<T> {
    Box::new(|dsl, station| {
        create_simple_solar_array_module(dsl, station, false, SolarArraySize::Small)
    })
}

/// Helper function to create a large solar array module
pub fn create_large_solar_array_module<T: spacetimedsl::WriteContext + 'static>(
) -> ModuleCreationFn<T> {
    Box::new(|dsl, station| {
        create_simple_solar_array_module(dsl, station, false, SolarArraySize::Large)
    })
}

/// Helper function to create a metal plate manufacturing module
pub fn create_metal_plate_module_fn<T: spacetimedsl::WriteContext + 'static>() -> ModuleCreationFn<T>
{
    Box::new(|dsl, station| create_metal_plate_module(dsl, station, false))
}
