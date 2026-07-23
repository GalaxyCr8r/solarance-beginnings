use spacetimedb::{Identity, ReducerContext, log};
use crate::spacetimedsl::prelude::*;

use crate::definitions::factions::FACTION_FACTIONLESS;
use crate::tables::{
    factions::*,
    messages::{post_faction_channel, MessageSender},
};

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

    // Faction validation (#93): the chosen faction must exist, be joinable,
    // and have a Capital station — new players spawn at their faction's
    // Capital (#105), so a faction without one has nowhere to put them. In
    // MVP this admits exactly Lrak Combine and Rediar Federation; the picker
    // greys everything else out, and this is the server-side backstop.
    let final_faction = FactionId::new(if faction_id == 0 {
        FACTION_FACTIONLESS
    } else {
        faction_id
    });
    let faction = dsl.get_faction_by_id(&final_faction).map_err(|_| {
        format!(
            "Registration rejected: faction id {} does not exist (player identity {})",
            final_faction.value(),
            identity
        )
    })?;
    if !faction.get_joinable() {
        return Err(format!(
            "Registration rejected: faction '{}' (id {}) is not joinable",
            faction.get_name(),
            final_faction.value()
        ));
    }
    if faction.get_capital_station_id().is_none() {
        return Err(format!(
            "Registration rejected: faction '{}' (id {}) has no Capital station to spawn at — only capital-holding factions are joinable in MVP (#93)",
            faction.get_name(),
            final_faction.value()
        ));
    }

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
