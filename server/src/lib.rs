use crate::{tables::global_config::*, timers::initialize_timers};
use spacetimedb::ReducerContext;
use spacetimedsl::*;
use tables::*;

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
    initialize_timers(&dsl)?;

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

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);
    // Called everytime a new client connects

    // TODO: When someone logs in set their player to online

    if let Some(mut config) = dsl.get_all_global_configurations().next() {
        config.set_active_players(config.get_active_players() + 1);
        dsl.update_global_config_by_id(config)?;
    }

    Ok(())
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);
    // Called everytime a client disconnects

    if let Some(mut config) = dsl.get_all_global_configurations().next() {
        if *config.get_active_players() > 0 {
            config.set_active_players(config.get_active_players() - 1);
            dsl.update_global_config_by_id(config)?;
        }
    }

    Ok(())
}
