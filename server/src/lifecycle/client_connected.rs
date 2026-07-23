use spacetimedb::*;
use crate::spacetimedsl::prelude::*;

use crate::logic::players::welcome_back::send_welcome_back_message;
use crate::tables::{global_config::*, players::*};

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);
    // Called everytime a new client connects

    // TODO: When someone logs in set their player to online

    if let Ok(mut player) = dsl.get_player_by_id(PlayerId::new(ctx.sender())) {
        // Add back timers and etc.

        // Compose the welcome-back message from the *pre-connect* baseline
        // (`player.last_login`), then re-stamp the baseline for next time. A
        // failed compose must not bounce the connection — log and carry on.
        if let Err(e) = send_welcome_back_message(&dsl, &player) {
            log::error!(
                "welcome-back compose failed for player {}: {}",
                ctx.sender().to_abbreviated_hex(),
                e
            );
        }

        player.last_login = Some(ctx.timestamp);
        dsl.update_player_by_id(player)?;
    }

    if let Some(mut config) = dsl.get_all_global_configurations().next() {
        config.set_active_players(config.get_active_players() + 1);
        dsl.update_global_config_by_id(config)?;
    }

    Ok(())
}
