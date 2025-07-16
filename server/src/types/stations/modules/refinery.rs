use super::*;

pub mod definitions;
pub mod timers;

#[dsl(plural_name = refinery_modules)]
#[table(name = refinery_module, public)]
pub struct Refinery {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,

    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub input_ore_resource_id: u32, // FK to ResourceDefinition

    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub output_ingot_resource_id: u32, // FK to ResourceDefinition

    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub waste_resource_id: Option<u32>, // FK to ResourceDefinition

    /// How many units of ore to make 1 unit of ingot.
    pub ore_to_ingot_ratio: f32,
    /// How many units of waste are produced to make 1 unit of ingot.
    pub waste_per_ingot_ratio: f32,

    pub base_ingots_produced_per_hour: f32,
    pub current_efficiency_modifier: f32, // Default 1.0
}