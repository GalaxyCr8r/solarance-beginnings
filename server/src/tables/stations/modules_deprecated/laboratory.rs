use super::*;

pub mod definitions;
pub mod timers;

#[dsl(plural_name = laboratory_modules, method(update = true))]
#[table(name = laboratory_module, public)]
pub struct Laboratory {
    #[primary_key]
    #[use_wrapper(StationModuleId)]
    /// FK to StationModule
    id: u64,

    pub base_research_points_per_hour: u32,

    #[use_wrapper(crate::tables::items::ItemDefinitionId)]
    /// Input resource ID (e.g., "Analyzed Data Cache") FK to ItemDefinition
    pub primary_input_resource_id: u32,

    #[use_wrapper(crate::tables::items::ItemDefinitionId)]
    /// Input resource ID (e.g., "Rare Crystal Sample") FK to ItemDefinition
    pub secondary_input_resource_id: Option<u32>,

    pub primary_input_consumption_rate: f32, // units per hour
    pub secondary_input_consumption_rate: Option<f32>, // units per hour
    pub current_efficiency_modifier: f32,    // Based on upgrades, staffing
}
