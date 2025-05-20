use schedulers::stellarobject_timers::{self};
use spacetimedb::ReducerContext;
use types::{common::{CreateGlobalConfigRow, GetAllGlobalConfigRows, GetCountOfGlobalConfigRows, UpdateGlobalConfigRowById}, ships};
use spacetimedsl::dsl;

pub mod types;
pub mod schedulers;

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    log::info!("init identity: {:?}", ctx.identity());
    log::info!("init sender: {:?}", ctx.sender);

    stellarobject_timers::init(ctx)?;
    ships::init(ctx)?;

    // Create a Global Config row, or reinitalize the one if it exists.
    if dsl.get_count_of_global_configurations() == 0 {
        dsl.create_global_config(0, 0, 0)?;
    } else {
        let mut config = dsl.get_all_global_configurations().last().ok_or("Failed to find existing global configuration")?;
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
        config.set_active_players(config.active_players+1);
        dsl.update_global_config_by_id(config)?;
    }

    Ok(())
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);
    // Called everytime a client disconnects

    if let Some(mut config) = dsl.get_all_global_configurations().last() {
        if config.active_players > 0 {
            config.set_active_players(config.active_players-1);
            dsl.update_global_config_by_id(config)?;
        }
    }

    Ok(())
}