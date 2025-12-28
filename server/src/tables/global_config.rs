use spacetimedb::{table, Timestamp};
use spacetimedsl::*;

#[dsl(plural_name = global_configurations, method(update = true))]
#[table(name = global_config)]
pub struct GlobalConfig {
    #[primary_key]
    #[create_wrapper]
    id: u32,

    pub active_players: u32,
    pub old_gods_defeated: u8,
    pub version: String,

    created_at: Timestamp,
    modified_at: Timestamp,
}

///////////////////////////////////////////////////////////
// Utility
///////////////////////////////////////////////////////////

pub fn global_config_any_active_players<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> bool {
    match dsl.get_global_config_by_id(GlobalConfigId::new(0)) {
        Ok(config) => {
            if config.active_players == 0 {
                return false;
            }
        }
        Err(_) => {}
    };

    true
}
