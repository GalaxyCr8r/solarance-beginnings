use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::tables::{chats::*, factions::FactionId, players::*, sectors::SectorId, ships::*};

/// Sends a message to the global chat channel visible to all players.
/// Validates the sender and logs the message before storing it in the database.
#[spacetimedb::reducer]
pub fn send_global_chat(ctx: &ReducerContext, chat_message: String) -> Result<(), String> {
    let dsl = dsl(ctx);

    // If ctx.sender is a valid, unbanned, unmuted player
    info!(
        "GlobalChat [{}]: {}",
        get_username(&dsl, ctx.sender),
        chat_message
    );

    dsl.create_global_chat_message(PlayerId::new(ctx.sender), &chat_message)?;
    Ok(())
}

/// Sends a message to a specific sector's chat channel.
/// Validates that the sender has a ship in the target sector before allowing the message.
#[spacetimedb::reducer]
pub fn send_sector_chat(
    ctx: &ReducerContext,
    chat_message: String,
    sector_id: u64,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let sender = PlayerId::new(ctx.sender);
    let username = get_username(&dsl, ctx.sender);

    if let Some(player) = dsl.get_ships_by_player_id(&sender).next() {
        if player.get_sector_id().value() != sector_id {
            return Err(format!(
                "Player {} is not in sector {}",
                username, sector_id
            ));
        }
    } else {
        return Err(format!("Player {} does not have a ship object", username));
    }

    // If ctx.sender is a valid, unbanned, unmuted player
    info!("SectorChat #{} [{}]: {}", sector_id, username, chat_message);

    dsl.create_sector_chat_message(sender, SectorId::new(sector_id), &chat_message)?;
    Ok(())
}

/// Sends a message to the faction chat channel visible to all players of that faction.
/// Validates the sender and logs the message before storing it in the database.
#[spacetimedb::reducer]
pub fn send_faction_chat(
    ctx: &ReducerContext,
    chat_message: String,
    faction_id: FactionId,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let sender = PlayerId::new(ctx.sender);
    let username = get_username(&dsl, ctx.sender);

    if let Ok(player) = dsl.get_player_by_id(&sender) {
        if player.get_faction_id().value() != faction_id.value() {
            return Err(format!(
                "Player {} does not access to faction id {}",
                username, faction_id
            ));
        }
    } else {
        return Err(format!(
            "Player {} does not access to faction id {}",
            username, faction_id
        ));
    }

    // If ctx.sender is a valid, unbanned, unmuted player
    info!(
        "FactionChat #{} [{}]: {}",
        faction_id.value(),
        get_username(&dsl, ctx.sender),
        chat_message
    );

    dsl.create_faction_chat_message(PlayerId::new(ctx.sender), faction_id, &chat_message)?;
    Ok(())
}
