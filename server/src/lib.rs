use schedulers::stellarobject_timers::{self};
use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table};

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
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(_ctx: &ReducerContext) {
    // Called everytime a client disconnects
}