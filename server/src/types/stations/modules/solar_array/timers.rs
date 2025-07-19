use super::*;
use crate::types::sectors::{GetSectorRowOptionById, Sector, SectorId};

/// Calculate the energy production for a solar array module
pub fn calculate_solar_array_production(
    ctx: &ReducerContext,
    solar_array: &SolarArray,
    time_elapsed_hours: f32,
) -> Result<SolarArrayProductionResult, String> {
    let dsl = dsl(ctx);

    // Get sector information to determine sunlight availability
    let station_module = dsl.get_station_module_by_id(solar_array.get_id())?;
    let station = dsl.get_station_by_id(StationId::new(station_module.station_id))?;
    let sector = dsl.get_sector_by_id(SectorId::new(station.sector_id))?;

    // Calculate sunlight efficiency based on sector properties
    let sunlight_efficiency = calculate_sunlight_efficiency(&sector);

    // Calculate total efficiency
    let total_efficiency = solar_array.current_efficiency_modifier
        * sunlight_efficiency
        * calculate_station_health_modifier(ctx, &dsl, &station)?;

    // Module operational status
    let operational_efficiency = if station_module.is_operational {
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
            station_module.id,
            energy_cells_produced_float
        );
        return Ok(SolarArrayProductionResult {
            energy_cells_produced: 0.0,
            sunlight_efficiency,
            total_efficiency: operational_efficiency,
            was_limited_by_sunlight: sunlight_efficiency < 1.0,
        });
    }

    spacetimedb::log::info!(
        "Solar array module {} producing {} whole energy cells",
        station_module.id,
        energy_cells_produced_whole
    );

    Ok(SolarArrayProductionResult {
        energy_cells_produced: energy_cells_produced_whole,
        sunlight_efficiency,
        total_efficiency: operational_efficiency,
        was_limited_by_sunlight: sunlight_efficiency < 1.0,
    })
}

/// Calculate sunlight efficiency based on sector properties
fn calculate_sunlight_efficiency(sector: &Sector) -> f32 {
    // Base sunlight efficiency from sector's sunlight field
    let mut efficiency = sector.sunlight;

    // Reduce efficiency in nebula sectors (dust blocks sunlight)
    if sector.nebula > 0.5 {
        efficiency *= 1.0 - (sector.nebula * 0.3); // Up to 30% reduction
    }

    // Note: asteroid_percentage field doesn't exist in Sector, so we'll skip that check

    efficiency.max(0.1).min(1.2) // Minimum 10%, maximum 120% (some sectors might have enhanced sunlight)
}

/// Calculate station health modifier for efficiency
fn calculate_station_health_modifier(
    _ctx: &ReducerContext,
    dsl: &DSL,
    station: &Station,
) -> Result<f32, String> {
    if let Ok(station_status) = dsl.get_station_status_by_id(station.get_id()) {
        Ok((station_status.health / 100.0).max(0.1))
    } else {
        Ok(1.0) // Default to full health if status not found
    }
}

/// Apply the calculated production results to the solar array's inventory
pub fn apply_solar_array_production(
    ctx: &ReducerContext,
    solar_array: &SolarArray,
    production_result: &SolarArrayProductionResult,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if production_result.energy_cells_produced <= 0.0 {
        return Ok(());
    }

    let station_module = dsl.get_station_module_by_id(solar_array.get_id())?;

    // Add produced energy cells to output inventory
    if let Some(mut output_inventory) = dsl
        .get_all_station_module_inventory_items()
        .into_iter()
        .find(|item| {
            item.module_id == station_module.id
                && item.resource_item_id == solar_array.output_energy_cell_resource_id
        })
    {
        let cells_to_add = production_result.energy_cells_produced as u32;
        output_inventory.set_quantity(output_inventory.quantity + cells_to_add);
        dsl.update_station_module_inventory_item_by_id(output_inventory)?;
    }

    Ok(())
}

/// Calculate and update efficiency modifiers for solar array
pub fn calculate_solar_array_efficiency(
    ctx: &ReducerContext,
    solar_array: &SolarArray,
) -> Result<f32, String> {
    let dsl = dsl(ctx);

    let station_module = dsl.get_station_module_by_id(solar_array.get_id())?;
    let station = dsl.get_station_by_id(StationId::new(station_module.station_id))?;
    let sector = dsl.get_sector_by_id(SectorId::new(station.sector_id))?;

    // Calculate combined efficiency
    let sunlight_efficiency = calculate_sunlight_efficiency(&sector);
    let health_modifier = calculate_station_health_modifier(ctx, &dsl, &station)?;

    let total_efficiency =
        solar_array.current_efficiency_modifier * sunlight_efficiency * health_modifier;

    // Module operational status
    if !station_module.is_operational {
        Ok(0.0)
    } else {
        Ok(total_efficiency.max(0.0).min(2.0))
    }
}

#[derive(Clone, Debug)]
pub struct SolarArrayProductionResult {
    pub energy_cells_produced: f32,
    pub sunlight_efficiency: f32,
    pub total_efficiency: f32,
    pub was_limited_by_sunlight: bool,
}
