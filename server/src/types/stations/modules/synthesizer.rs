use super::*;

#[dsl(plural_name = synthesizer_modules)]
#[table(name = synthesizer_module, public)]
pub struct Synthesizer {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,

    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub input_exotic_matter_resource_id: u32,

    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub input_gas_resource_id: u32,

    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub output_jump_fuel_resource_id: u32,

    pub exotic_matter_per_fuel_unit: f32,
    pub gas_per_fuel_unit: f32,
    pub base_fuel_units_produced_per_hour: f32,
}