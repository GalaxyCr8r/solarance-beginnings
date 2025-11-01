use super::*;

#[dsl(plural_name = residential_modules)]
#[table(name = residential_module, public)]
pub struct Residential {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,

    /// Base max occupancy from blueprint, actual can be affected by upgrades/morale.
    pub base_max_occupancy: u32,
    pub current_occupancy: u32, // Generic population/crew
    /// Morale specifically within this residential module. Affects overall station morale.
    pub current_internal_morale_percentage: f32, // 0.0 to 100.0
    pub crew_replenishment_pool: u32, // Available crew for players to hire
    pub crew_generation_rate_per_hour: f32,

    // --- Fields specific to Spacious Residential ---
    /// For Spacious/Luxury, the level of park/amenity upgrades.
    pub amenity_upgrade_level: Option<u8>,
    /// Additional morale boost from amenities.
    pub amenity_morale_boost: Option<i16>,

    // --- Fields specific to Luxury Residential ---
    pub max_luxury_npc_slots: Option<u8>,
    pub current_luxury_npc_count: Option<u8>,

    /// FKs to ResourceDefinition (luxury food, drinks etc.)
    pub luxury_upkeep_requirements: Option<Vec<ResourceAmount>>,
}