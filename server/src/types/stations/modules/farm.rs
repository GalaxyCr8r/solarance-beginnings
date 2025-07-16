use super::*;

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum FarmOutputQuality {
    Lower,
    Average,
    Upper,
    Luxury,
}

#[dsl(plural_name = farm_modules)]
#[table(name = farm_module, public)]
pub struct Farm {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,

    /// Defined by the StationModuleBlueprint.specific_type (e.g., LuxuryFoodFarm produces Luxury Food)
    /// FK to ResourceDefinition (e.g., "Luxury Food", "Standard Wood")
    pub output_resource_id: u32,

    pub output_quality: FarmOutputQuality,

    /// Resource ID for the primary input (e.g., "Raw Biomatter Type A")
    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub primary_input_resource_id: u32,
    pub primary_input_conversion_rate: f32, // units of primary input per unit of output

    /// Resource ID for a secondary input (e.g., "Purified Water")
    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub secondary_input_resource_id: Option<u32>,
    pub secondary_input_conversion_rate: Option<f32>,

    pub base_production_units_per_hour: f32,
    pub current_efficiency_modifier: f32, // Based on sector, upgrades, staffing. Default 1.0
}