use spacetimedb::{Identity, ReducerContext, log};
use spacetimedsl::*;

use crate::definitions::factions::FACTION_FACTIONLESS;
use crate::tables::{
    factions::FactionId,
    messages::{post_faction_channel, MessageSender},
};

use crate::players::*;
use crate::tables::players::{CreatePlayer, PlayerId};

//////////////////////////////////////////////////////////////
// Reducers ///
//////////////////////////////////////////////////////////////

/// Registers a new player with a unique username and creates their player account.
/// Validates username uniqueness and initializes the player with starting credits.
#[spacetimedb::reducer]
pub fn register_playername(
    ctx: &ReducerContext,
    identity: Identity,
    username: String,
    faction_id: u32,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // TODO: Check if the identity already has a player!!!!

    if dsl.get_player_by_id(PlayerId::new(identity)).is_ok() {
        log::error!("Player Already Registered");
        return Err("Player Already Registered.".to_string());
    }

    if username.len() > 32 {
        log::error!("Username is toooooo long");
        return Err("Username is toooooo long.".to_string());
    }

    if dsl.get_player_by_username(&username).is_ok() {
        // Synchronous validation failure — reducer Err is enough. The client
        // surfaces this via the `_then` callback at registration time. We do
        // *not* persist a DM for it (per #101: inbox is for async events).
        log::error!(
            "Username '{}' is already taken. Please choose a different username.",
            username
        );
        return Err("Username already taken!".to_string());
    }

    // TODO: Re-enable faction validation once client bindings are updated
    // For now, just use the provided faction_id or default to factionless
    let final_faction = FactionId::new(if faction_id == 0 {
        FACTION_FACTIONLESS
    } else {
        faction_id
    });

    let player = dsl.create_player(CreatePlayer {
        id: identity,
        username: username.clone(),
        credits: 1000,
        logged_in: true,
        faction_id: final_faction.clone(),
        last_login: None, // Stamped by the welcome-back composer on first connect.
    })?;
    let _ = post_faction_channel(
        &dsl,
        final_faction,
        MessageSender::Player(identity),
        format!("{} has joined the faction!", player.get_username()),
    );

    Ok(())
}

//////////////////////////////////////////////////////////////
// Utils
//////////////////////////////////////////////////////////////
