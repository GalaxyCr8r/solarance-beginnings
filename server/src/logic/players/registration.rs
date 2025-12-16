use spacetimedb::{Identity, ReducerContext};
use spacetimedsl::*;

use crate::definitions::factions::FACTION_FACTIONLESS;
use crate::tables::{chats::*, factions::FactionId, server_messages::send_error_message};

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

    if dsl.get_player_by_id(PlayerId::new(identity)).is_err() {
        return Err("Player Already Registered.".to_string());
    }

    if username.len() > 32 {
        return Err("Username is toooooo long.".to_string());
    }

    if dsl.get_player_by_username(&username).is_ok() {
        let player_id = PlayerId::new(identity);
        let error_message = format!(
            "Username '{}' is already taken. Please choose a different username.",
            username
        );

        // Send server message for error feedback
        send_error_message(
            &dsl,
            &player_id,
            error_message.clone(),
            Some("Player Registration"),
        )?;

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
        username,
        credits: 1000,
        logged_in: true,
        faction_id: final_faction.clone(),
    })?;
    let _ = dsl.create_faction_chat_message(CreateFactionChatMessage {
        player_id: PlayerId::new(identity),
        faction_id: final_faction,
        message: "- has joined the faction!".to_string(),
    });

    Ok(())
}

//////////////////////////////////////////////////////////////
// Utils
//////////////////////////////////////////////////////////////
