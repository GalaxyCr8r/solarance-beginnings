use super::*;

pub mod definitions;
pub mod timers;

#[dsl(plural_name = solar_array_modules)]
#[table(name = solar_array_module, public)]
pub struct SolarArray {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,

    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub output_energy_cell_resource_id: u32, // FK to ResourceDefinition

    pub base_energy_cells_produced_per_hour: u32,
    /// Efficiency based on sector's sunlight_percentage and module health/upgrades.
    pub current_efficiency_modifier: f32, // Default 1.0
}
