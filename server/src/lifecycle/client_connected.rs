use spacetimedb::*;
use spacetimedsl::*;

use crate::tables::global_config::*;

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
