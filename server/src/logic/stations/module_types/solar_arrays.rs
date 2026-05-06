use spacetimedb::table;
use spacetimedsl::*;

use crate::tables::sectors::*;
use crate::tables::stations::*;
use crate::{definitions::item_types::*, tables::items::*};

#[derive(Clone, Debug)]
pub enum SolarArraySize {
    Small,      // 100 energy cells/hour
    Medium,     // 250 energy cells/hour
    Large,      // 500 energy cells/hour
    Industrial, // 1000 energy cells/hour
}

// Add missing module constants
pub const MODULE_SOLAR_ARRAY_SMALL: u32 = 7_000;
pub const MODULE_SOLAR_ARRAY_MEDIUM: u32 = 7_001;
pub const MODULE_SOLAR_ARRAY_LARGE: u32 = 7_002;
pub const MODULE_SOLAR_ARRAY_INDUSTRIAL: u32 = 7_003;

#[dsl(plural_name = solar_array_modules, method(update = true))]
#[table(accessor = solar_array_module, public)]
pub struct SolarArray {
    #[primary_key]
    #[use_wrapper(StationModuleId)]
    /// FK to StationModule
    id: u64,

    #[use_wrapper(crate::tables::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    output_energy_cell_resource_id: u32, // FK to ResourceDefinition

    base_energy_cells_produced_per_hour: u32,
    /// Efficiency based on sector's sunlight_percentage and module health/upgrades.
    pub current_efficiency_modifier: f32, // Default 1.0
}

#[derive(Clone, Debug)]
pub struct SolarArrayProductionResult {
    pub energy_cells_produced: u32,
    pub sunlight_efficiency: f32,
    pub total_efficiency: f32,
    pub was_limited_by_sunlight: bool,
}

////////////////////////////////////
/// Create Module

pub fn create_simple_solar_array_module<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station: &Station,
    under_construction: bool,
    array_size: SolarArraySize,
) -> Result<(), String> {
    if under_construction {
        return Err("Not yet implemented".to_string());
    }

    let (blueprint_id, base_production) = match array_size {
        SolarArraySize::Small => (MODULE_SOLAR_ARRAY_SMALL, 100),
        SolarArraySize::Medium => (MODULE_SOLAR_ARRAY_MEDIUM, 250),
        SolarArraySize::Large => (MODULE_SOLAR_ARRAY_LARGE, 500),
        SolarArraySize::Industrial => (MODULE_SOLAR_ARRAY_INDUSTRIAL, 1000),
    };

    let blueprint =
        dsl.get_station_module_blueprint_by_id(StationModuleBlueprintId::new(blueprint_id))?;
    let identifier = format!("{:?} Solar Array", array_size);

    let module = dsl.create_station_module(CreateStationModule {
        station_id: station.get_id(),
        blueprint: blueprint.get_id(),
        station_slot_identifier: identifier.as_str().to_string(),
        is_operational: true,
        built_at_timestamp: None,
        last_status_update_timestamp: dsl.ctx().timestamp()?,
    })?;

    // Create solar array submodule
    let solar_array = dsl.create_solar_array_module(CreateSolarArrayModule {
        id: module.get_id(),
        output_energy_cell_resource_id: ItemDefinitionId::new(ITEM_ENERGY_CELL),
        base_energy_cells_produced_per_hour: base_production,
        current_efficiency_modifier: 1.0,
    })?;

    // Create output inventory for energy cells
    let mut item = dsl.create_station_module_inventory_item(CreateStationModuleInventoryItem {
        module_id: module.get_id(),
        resource_item_id: ItemDefinitionId::new(ITEM_ENERGY_CELL),
        quantity: 0,
        max_quantity: blueprint
            .get_max_internal_storage_volume_per_slot_m3()
            .unwrap(),
        storage_purpose_tag: format!("{};{};output", module.get_id().value(), solar_array.id),
        cached_price: 0,
    })?;

    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_ENERGY_CELL)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////
/// Utilities

/// Calculate the energy production for a solar array module
pub fn calculate_solar_array_production<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    solar_array: &SolarArray,
    time_elapsed_hours: f32,
) -> Result<SolarArrayProductionResult, String> {
    // Get sector information to determine sunlight availability
    let station_module = dsl.get_station_module_by_id(solar_array.get_id())?;
    let station = dsl.get_station_by_id(station_module.get_station_id())?;
    let sector = dsl.get_sector_by_id(station.get_sector_id())?;

    // Calculate sunlight efficiency based on sector properties
    let sunlight_efficiency = calculate_sunlight_efficiency(&sector);

    // Calculate total efficiency
    let total_efficiency = solar_array.current_efficiency_modifier
        * sunlight_efficiency
        * calculate_station_health_modifier(&dsl, &station)?;

    // Module operational status
    let operational_efficiency = if *station_module.get_is_operational() {
        total_efficiency
    } else {
        0.0
    };

    // Calculate energy production (as whole units only)
    let energy_cells_produced_float = (solar_array.base_energy_cells_produced_per_hour as f32)
        * operational_efficiency
        * time_elapsed_hours;
    let energy_cells_produced_whole = energy_cells_produced_float.floor();

    // Only produce if we can make at least 1 whole energy cell
    if energy_cells_produced_whole < 1.0 {
        spacetimedb::log::info!(
            "Solar array module {} cannot produce whole energy cells: calculated {:.2}",
            station_module.get_id(),
            energy_cells_produced_float
        );
        return Ok(SolarArrayProductionResult {
            energy_cells_produced: 0,
            sunlight_efficiency,
            total_efficiency: operational_efficiency,
            was_limited_by_sunlight: sunlight_efficiency < 1.0,
        });
    }

    spacetimedb::log::info!(
        "Solar array module {} producing {} whole energy cells",
        station_module.get_id(),
        energy_cells_produced_whole
    );

    Ok(SolarArrayProductionResult {
        energy_cells_produced: energy_cells_produced_whole as u32,
        sunlight_efficiency,
        total_efficiency: operational_efficiency,
        was_limited_by_sunlight: sunlight_efficiency < 1.0,
    })
}

/// Calculate sunlight efficiency based on sector properties
fn calculate_sunlight_efficiency(sector: &Sector) -> f32 {
    // Base sunlight efficiency from sector's sunlight field
    let mut efficiency = *sector.get_sunlight();

    // Reduce efficiency in nebula sectors (dust blocks sunlight)
    if *sector.get_nebula() > 0.5 {
        efficiency *= 1.0 - (sector.get_nebula() * 0.3); // Up to 30% reduction
    }

    // Note: asteroid_percentage field doesn't exist in Sector, so we'll skip that check

    efficiency.max(0.1).min(1.2) // Minimum 10%, maximum 120% (some sectors might have enhanced sunlight)
}

/// Calculate station health modifier for efficiency
pub fn calculate_station_health_modifier<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station: &Station,
) -> Result<f32, String> {
    if let Ok(station_status) = dsl.get_station_status_by_id(station.get_id()) {
        Ok((station_status.health / 100.0).max(0.1))
    } else {
        Ok(1.0) // Default to full health if status not found
    }
}

/// Apply the calculated production results to the solar array's inventory
pub fn apply_solar_array_production<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    solar_array: &SolarArray,
    production_result: &SolarArrayProductionResult,
) -> Result<(), String> {
    if production_result.energy_cells_produced == 0 {
        return Ok(());
    }

    let station_module = dsl.get_station_module_by_id(solar_array.get_id())?;

    // Add produced energy cells to output inventory
    if let Some(mut output_inventory) = dsl
        .get_all_station_module_inventory_items()
        .into_iter()
        .find(|item: &StationModuleInventoryItem| {
            item.get_module_id() == station_module.get_id()
                && item.get_resource_item_id() == solar_array.get_output_energy_cell_resource_id()
        })
    {
        let cells_to_add = production_result.energy_cells_produced;
        output_inventory.quantity = output_inventory.quantity + cells_to_add;
        dsl.update_station_module_inventory_item_by_id(output_inventory)?;
    }

    Ok(())
}

/// Calculate and update efficiency modifiers for solar array
pub fn calculate_solar_array_efficiency<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    solar_array: &SolarArray,
) -> Result<f32, String> {
    let station_module = dsl.get_station_module_by_id(solar_array.get_id())?;
    let station = dsl.get_station_by_id(station_module.get_station_id())?;
    let sector = dsl.get_sector_by_id(station.get_sector_id())?;

    // Calculate combined efficiency
    let sunlight_efficiency = calculate_sunlight_efficiency(&sector);
    let health_modifier = calculate_station_health_modifier(&dsl, &station)?;

    let total_efficiency =
        solar_array.current_efficiency_modifier * sunlight_efficiency * health_modifier;

    // Module operational status
    if !*station_module.get_is_operational() {
        Ok(0.0)
    } else {
        Ok(total_efficiency.max(0.0).min(2.0))
    }
}
