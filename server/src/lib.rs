use schedulers::stellarobject_timers::{self};
use spacetimedb::{ReducerContext, Table};
use types::common::{global_config, GlobalConfig};

pub mod types;
pub mod schedulers;

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    log::info!("init identity: {:?}", ctx.identity());
    log::info!("init sender: {:?}", ctx.sender);

    stellarobject_timers::init(ctx);

    // Create a Global Config row, or reinitalize the one if it exists.
    if ctx.db.global_config().iter().count() == 0 {
        ctx.db.global_config().insert(GlobalConfig {id:0, active_players:0, old_gods_defeated: 0 });
    } else {
        if let Some(config) = ctx.db.global_config().id().find(0) {
            ctx.db.global_config().id().update(GlobalConfig {
                active_players: 0,
                ..config
            });
        }
    }
}

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) {
    // Called everytime a new client connects
    
    // TODO: When someone logs in set their player to online

    if let Some(config) = ctx.db.global_config().id().find(0) {
        ctx.db.global_config().id().update(GlobalConfig {
            active_players: config.active_players+1,
            ..config
        });
    }
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    // Called everytime a client disconnects

    if let Some(config) = ctx.db.global_config().id().find(0) {
        if config.active_players > 0 {
            ctx.db.global_config().id().update(GlobalConfig {
                active_players: config.active_players-1,
                ..config
            });
        }
    }
}