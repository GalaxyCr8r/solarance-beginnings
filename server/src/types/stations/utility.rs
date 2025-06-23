use super::*;

/// Verify the invariants of this class that Rust cannot guarantee due to the database limitations.
/// Should be called after modifying a station.
pub fn verify(ctx: &ReducerContext, station: Station) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Verify the station does not have more modules than it should.
    if dsl
        .get_station_modules_by_station_id(station.get_id())
        .count()
        > station.size.modules() as usize
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
    //
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
