use spacetimedb::{table, Timestamp};
use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = global_configurations, method(update = true))]
#[table(accessor = global_config)]
pub struct GlobalConfig {
    #[primary_key]
    #[create_wrapper]
    id: u32,

    pub active_players: u32,
    pub old_gods_defeated: u8,
    pub version: String,

    // ── Cargo-crate jettison & lifecycle tunables ─────────────────────────
    // Read by Phase 3 snapshot helpers (re-stamp `max_turn_rate`) and by the
    // Phase 5 jettison reducer (toss velocity, brake rate). Phase 7's sweeper
    // reads `cargo_crate_ttl_secs`.
    pub cargo_crate_ttl_secs: u64,
    pub cargo_crate_toss_speed: f32,
    pub cargo_crate_toss_speed_variance: f32,
    pub cargo_crate_brake_rate: f32,
    pub cargo_crate_brake_rate_variance: f32,
    pub cargo_crate_max_turn_rate: f32,

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
