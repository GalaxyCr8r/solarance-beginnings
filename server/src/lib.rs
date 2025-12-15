use spacetimedb::ReducerContext;
use spacetimedsl::*;
use tables::*;

use crate::tables::global_config::*;

pub mod admin;
pub mod definitions;
pub mod logic;
pub mod tables;
pub mod timers;
pub mod utility;

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    definitions::init(&dsl)?;

    // Create a Global Config row, or reinitalize the one if it exists.
    if dsl.count_of_all_global_configurations() == 0 {
        dsl.create_global_config(0, 0, 0, env!("CARGO_PKG_VERSION"))?;
    } else {
        let mut config = dsl
            .get_all_global_configurations()
            .last()
            .ok_or("Failed to find existing global configuration")?;
        config.set_active_players(0);
        dsl.update_global_config_by_id(config)?;
    }
    Ok(())
}

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);
    // Called everytime a new client connects

    // TODO: When someone logs in set their player to online

    if let Some(mut config) = dsl.get_all_global_configurations().last() {
        config.set_active_players(config.get_active_players() + 1);
        dsl.update_global_config_by_id(config)?;
    }

    Ok(())
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);
    // Called everytime a client disconnects

    if let Some(mut config) = dsl.get_all_global_configurations().last() {
        if *config.get_active_players() > 0 {
            config.set_active_players(config.get_active_players() - 1);
            dsl.update_global_config_by_id(config)?;
        }
    }

    Ok(())
}
