use super::*;

pub mod definitions;
pub mod timers;

/// Defines a recipe that a manufacturing module can use.
#[dsl(plural_name = production_recipe_definitions)]
#[table(name = production_recipe_definition, public)]
pub struct ProductionRecipeDefinition {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u32,

    #[unique]
    pub name: String, // e.g., "Basic Hull Plating", "Mk1 Laser Cannon Assembly"

    pub input_resources: Vec<ResourceAmount>,

    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub output_resource_id: u32, // FK to ResourceDefinition

    pub output_quantity: u32,
    pub base_production_time_seconds: u32,
    /// Which type of module can use this recipe (e.g., Factory, Assembler)
    pub required_module_specific_type: StationModuleSpecificType,
    pub required_tech_id_to_unlock: Option<u32>, // FK to TechnologyTreeNode
}

/// Data for a generic manufacturing module instance (Factory, Assembler, Fabricator).
#[dsl(plural_name = manufacturing_modules)]
#[table(name = manufacturing_module, public)]
pub struct Manufacturing {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,

    #[use_wrapper(path = ProductionRecipeDefinitionId)]
    /// The recipe this specific module instance is currently configured to produce. FK to ProductionRecipeDefinition
    pub current_recipe_id: Option<u32>,

    pub is_producing: bool,
    pub production_queue_count: u32, // Number of items queued for production
    pub current_production_progress_seconds: f32,
    /// Modifier based on upgrades, staffing, etc. Affects production_time_seconds.
    pub production_speed_modifier: f32, // Default 1.0
}
