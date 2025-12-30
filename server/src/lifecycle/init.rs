use spacetimedb::*;
use spacetimedsl::*;

use crate::{definitions, tables::global_config::*, timers};

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    definitions::init(&dsl)?;
    timers::initialize(&dsl)?;

    // Create a Global Config row, or reinitalize the one if it exists.
    if dsl.get_all_global_configurations().count() == 0 {
        dsl.create_global_config(CreateGlobalConfig {
            id: 0,
            active_players: 0,
            old_gods_defeated: 0,
            version: env!("CARGO_PKG_VERSION").to_string(),
        })?;
    } else {
        let mut config = dsl
            .get_all_global_configurations()
            .into_iter()
            .last()
            .ok_or("Failed to find existing global configuration")?;
        config.set_active_players(0);
        dsl.update_global_config_by_id(config)?;
    }
    Ok(())
}
