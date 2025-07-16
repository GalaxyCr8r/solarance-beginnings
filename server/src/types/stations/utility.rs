use super::*;
use crate::types::{
    factions::FactionId,
    items::GetItemDefinitionRowOptionById,
    sectors::Sector,
    stations::timers::{CreateStationProductionScheduleRow, CreateStationStatusScheduleRow},
    stellarobjects::StellarObject,
};
use spacetimedb::ScheduleAt;
use std::time::Duration;

/// Type alias for module creation functions
pub type ModuleCreationFn = Box<dyn Fn(&ReducerContext, &Station) -> Result<(), String>>;

/// Helper function to create a basic trading module
pub fn create_trading_module() -> ModuleCreationFn {
    Box::new(|ctx, station| modules::trading_port::create_basic_bazaar(ctx, station, false))
}

/// Helper function to create a basic refinery module for iron ore
pub fn create_iron_refinery_module() -> ModuleCreationFn {
    Box::new(|ctx, station| {
        modules::refinery::definitions::create_basic_refinery_module(
            ctx,
            station,
            false,
            crate::types::items::ItemDefinitionId::new(
                crate::types::items::definitions::ITEM_IRON_ORE,
            ),
            crate::types::items::ItemDefinitionId::new(
                crate::types::items::definitions::ITEM_IRON_INGOT,
            ),
            None,
        )
    })
}

/// Helper function to create a basic refinery module for ice ore
pub fn create_ice_refinery_module() -> ModuleCreationFn {
    Box::new(|ctx, station| {
        modules::refinery::definitions::create_basic_refinery_module(
            ctx,
            station,
            false,
            crate::types::items::ItemDefinitionId::new(
                crate::types::items::definitions::ITEM_ICE_ORE,
            ),
            crate::types::items::ItemDefinitionId::new(
                crate::types::items::definitions::ITEM_WATER,
            ),
            None,
        )
    })
}

/// Helper function to create a basic refinery module for silicon ore
pub fn create_silicon_refinery_module() -> ModuleCreationFn {
    Box::new(|ctx, station| {
        modules::refinery::definitions::create_basic_refinery_module(
            ctx,
            station,
            false,
            crate::types::items::ItemDefinitionId::new(
                crate::types::items::definitions::ITEM_SILICON_ORE,
            ),
            crate::types::items::ItemDefinitionId::new(
                crate::types::items::definitions::ITEM_SILICON_RAW,
            ),
            None,
        )
    })
}

/// Helper function to create a station with modules and automatically set up schedules
pub fn create_station_with_modules(
    ctx: &ReducerContext,
    size: StationSize,
    sector: &Sector,
    sobj: &StellarObject,
    owner_faction_id: FactionId,
    name: &str,
    description: Option<String>,
    module_creators: Vec<ModuleCreationFn>,
) -> Result<Station, String> {
    let dsl = dsl(ctx);

    // Create the station
    let station = dsl.create_station(size, sector, sobj, owner_faction_id, name, description)?;

    // Create all modules
    for module_creator in module_creators {
        module_creator(ctx, &station)?;
    }

    // Set up station production schedule (every 30 seconds)
    dsl.create_station_production_schedule(
        station.get_id(),
        ScheduleAt::Interval(Duration::from_secs(30).into()),
        ctx.timestamp,
    )?;

    // Set up station status schedule (every 60 seconds)
    dsl.create_station_status_schedule(
        station.get_id(),
        ScheduleAt::Interval(Duration::from_secs(60).into()),
        ctx.timestamp,
    )?;

    // Verify station invariants
    verify(ctx, station.clone())?;

    Ok(station)
}

/// Verify the invariants of this class that Rust cannot guarantee due to the database limitations.
/// Should be called after modifying a station.
pub fn verify(ctx: &ReducerContext, station: Station) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Verify the station does not have more modules than it should.
    if dsl
        .get_station_modules_by_station_id(station.get_id())
        .count()
        > (station.size.modules() as usize)
    {
        return Err("Too many station modules attached.".to_string());
    }

    Ok(())
}

/// LogisticsAndStorage,
pub fn update_logistics_and_storage(
    ctx: &ReducerContext,
    station: &Station,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Update cached prices for all inventory items in this module
    for mut inventory_item in dsl.get_station_module_inventory_items_by_module_id(module.get_id()) {
        if let Ok(item_def) = dsl.get_item_definition_by_id(inventory_item.get_resource_item_id()) {
            let current_price = inventory_item.calculate_current_price(&item_def);
            inventory_item.set_cached_price(current_price);
            dsl.update_station_module_inventory_item_by_id(inventory_item)?;
        }
    }

    Ok(())
}

/// ResourceProductionAndRefining,
pub fn update_resource_production_and_refining(
    ctx: &ReducerContext,
    station: &Station,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    //
    Ok(())
}
/// ManufacturingAndAssembly,
pub fn update_manufacturing_and_assembly(
    ctx: &ReducerContext,
    station: &Station,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    //
    Ok(())
}

/// ResearchAndDevelopment,
pub fn update_research_and_development(
    ctx: &ReducerContext,
    station: &Station,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    //
    Ok(())
}

/// CivilianAndSupportServices,
pub fn update_civilian_and_support_services(
    ctx: &ReducerContext,
    station: &Station,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    //
    Ok(())
}

/// DiplomacyAndFaction,
pub fn update_diplomacy_and_faction(
    ctx: &ReducerContext,
    station: &Station,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    //
    Ok(())
}

/// DefenseAndMilitary,
pub fn update_defense_and_military(
    ctx: &ReducerContext,
    station: &Station,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
) -> Result<(), String> {
    //
    Ok(())
}
/// Helper function to create a basic food farm module
pub fn create_basic_farm_module() -> ModuleCreationFn {
    Box::new(|ctx, station| {
        modules::farm::definitions::create_basic_food_farm(
            ctx,
            station,
            false,
            modules::farm::FarmOutputQuality::Average,
        )
    })
}

/// Helper function to create a luxury food farm module
pub fn create_luxury_farm_module() -> ModuleCreationFn {
    Box::new(|ctx, station| {
        modules::farm::definitions::create_basic_food_farm(
            ctx,
            station,
            false,
            modules::farm::FarmOutputQuality::Luxury,
        )
    })
}

/// Helper function to create a basic laboratory module
pub fn create_basic_laboratory_module() -> ModuleCreationFn {
    Box::new(|ctx, station| {
        modules::laboratory::definitions::create_basic_laboratory(
            ctx,
            station,
            false,
            modules::laboratory::definitions::LaboratoryType::Basic,
        )
    })
}

/// Helper function to create an advanced laboratory module
pub fn create_advanced_laboratory_module() -> ModuleCreationFn {
    Box::new(|ctx, station| {
        modules::laboratory::definitions::create_basic_laboratory(
            ctx,
            station,
            false,
            modules::laboratory::definitions::LaboratoryType::Advanced,
        )
    })
}

/// Helper function to create a basic manufacturing module
pub fn create_basic_manufacturing_module() -> ModuleCreationFn {
    Box::new(|ctx, station| {
        modules::manufacturing::definitions::create_basic_manufacturing_module(
            ctx,
            station,
            false,
            modules::manufacturing::definitions::ManufacturingType::BasicFactory,
        )
    })
}

/// Helper function to create an advanced manufacturing module
pub fn create_advanced_manufacturing_module() -> ModuleCreationFn {
    Box::new(|ctx, station| {
        modules::manufacturing::definitions::create_basic_manufacturing_module(
            ctx,
            station,
            false,
            modules::manufacturing::definitions::ManufacturingType::AdvancedFactory,
        )
    })
}

/// Helper function to create a small solar array module
pub fn create_small_solar_array_module() -> ModuleCreationFn {
    Box::new(|ctx, station| {
        modules::solar_array::definitions::create_basic_solar_array(
            ctx,
            station,
            false,
            modules::solar_array::definitions::SolarArraySize::Small,
        )
    })
}

/// Helper function to create a large solar array module
pub fn create_large_solar_array_module() -> ModuleCreationFn {
    Box::new(|ctx, station| {
        modules::solar_array::definitions::create_basic_solar_array(
            ctx,
            station,
            false,
            modules::solar_array::definitions::SolarArraySize::Large,
        )
    })
}
