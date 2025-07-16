use super::*;

#[dsl(plural_name = storage_depot_modules)]
#[table(name = storage_depot_module, public)]
pub struct StorageDepot {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,
    // Configuration for item capacity is better in StationModuleBlueprint (max_internal_storage_slots/volume)

    // This table denotes that this station is a storage depot
}