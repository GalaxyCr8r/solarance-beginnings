use schedulers::stellarobject_timers::{self};
use spacetimedb::ReducerContext;

pub mod types;
pub mod schedulers;

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    log::info!("init identity: {:?}", ctx.identity());
    log::info!("init sender: {:?}", ctx.sender);

    stellarobject_timers::init(ctx);
}

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(_ctx: &ReducerContext) {
    // Called everytime a new client connects
    
    // TODO: When someone logs in set their player to online
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(_ctx: &ReducerContext) {
    // Called everytime a client disconnects
}