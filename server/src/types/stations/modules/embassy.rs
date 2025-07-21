use super::*;
use crate::factions::FactionId;

#[dsl(plural_name = embassy_presences)]
#[table(name = embassy_presence, public)]
pub struct EmbassyPresence {
    #[primary_key] // Composite key: "embassy_module_id:representing_faction_id" - Must be MANUALLY enforced
    #[create_wrapper]
    id: String,

    #[index(btree)]
    #[use_wrapper(path = StationModuleId)]
    pub embassy_module_id: u64,

    #[index(btree)]
    #[use_wrapper(path = FactionId)]
    pub representing_faction_id: u32,

    pub established_at_timestamp: Timestamp,
    pub diplomatic_status_notes: Option<String>, // e.g., "Ambassadorial level", "Trade mission"
}

#[dsl(plural_name = embassy_modules)]
#[table(name = embassy_module, public)]
pub struct Embassy {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,
    // Configuration for item capacity is better in StationModuleBlueprint (max_internal_storage_slots/volume)

    // This table denotes that this station is a storage depot
}