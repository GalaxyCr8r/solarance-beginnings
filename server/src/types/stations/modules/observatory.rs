use super::*;

#[dsl(plural_name = observatory_modules)]
#[table(name = observatory_module, public)]
pub struct Observatory {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,

    pub base_research_points_per_hour: u32,
    /// Efficiency based on sector type (nebula, anomaly) and module upgrades.
    pub current_efficiency_modifier: f32, // Default 1.0

    /// Input resource ID (e.g., "Advanced Sensor Crystal")
    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub primary_input_resource_id: u32,
    pub primary_input_consumption_rate: Option<f32>, // units per hour of operation

    /// Output resource ID (e.g., "Raw Astronomical Data")
    pub output_data_fragment_resource_id: u32,
}