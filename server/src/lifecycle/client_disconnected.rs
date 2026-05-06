use spacetimedb::*;
use spacetimedsl::*;

use crate::tables::{global_config::*, players::*};

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);
    // Called everytime a client disconnects

    if let Ok(_player) = dsl.get_player_by_id(PlayerId::new(ctx.sender())) {
        // Remove unneccessary timers and etc.
    }

    if let Some(mut config) = dsl.get_all_global_configurations().next() {
        if *config.get_active_players() > 0 {
            config.set_active_players(config.get_active_players() - 1);
            dsl.update_global_config_by_id(config)?;
        }
    }

    Ok(())
}
