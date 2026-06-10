//! Player-sent Channel Message reducers (#101 redesign).
//!
//! One reducer per channel a player can post into. Each derives the audience
//! key (sector / system / faction) from server-side state — the client never
//! tells us "I'm in sector N", because the server already knows.

use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::*;

use crate::tables::{
    messages::{
        post_faction_channel, post_galaxy_channel, post_sector_channel,
        post_star_system_channel, MessageSender,
    },
    players::*,
    sectors::{SectorId, *},
    ships::*,
};

/// Send a message to the **Galaxy** channel — visible to every logged-in player.
#[spacetimedb::reducer]
pub fn send_galaxy_chat(ctx: &ReducerContext, message: String) -> Result<(), String> {
    let dsl = dsl(ctx);
    let sender = ctx.sender();

    // Resolve the player row both to enforce "logged-in to post" and to log a username.
    let username = get_username(&dsl, sender);

    info!("GalaxyChat [{}]: {}", username, message);

    post_galaxy_channel(&dsl, MessageSender::Player(sender), message)
}

/// Send a message to the caller's current **StarSystem** channel.
///
/// Derived: the system the player's ship is currently in. No ship → reject.
#[spacetimedb::reducer]
pub fn send_star_system_chat(ctx: &ReducerContext, message: String) -> Result<(), String> {
    let dsl = dsl(ctx);
    let sender = ctx.sender();
    let player_id = PlayerId::new(sender);
    let username = get_username(&dsl, sender);

    let ship = dsl
        .get_ships_by_player_id(&player_id)
        .next()
        .ok_or_else(|| format!("Player {} has no ship — cannot post to star system chat", username))?;
    let sector = dsl.get_sector_by_id(&ship.get_sector_id().clone())?;
    // `sector.get_system_id()` already returns the typed wrapper.
    let system_id = sector.get_system_id().clone();

    info!(
        "StarSystemChat #{} [{}]: {}",
        system_id.value(),
        username,
        message
    );

    post_star_system_channel(&dsl, system_id, MessageSender::Player(sender), message)
}

/// Send a message to the caller's current **Sector** channel.
///
/// Derived: the sector the player's ship is currently in. No ship → reject.
#[spacetimedb::reducer]
pub fn send_sector_chat(ctx: &ReducerContext, message: String) -> Result<(), String> {
    let dsl = dsl(ctx);
    let sender = ctx.sender();
    let player_id = PlayerId::new(sender);
    let username = get_username(&dsl, sender);

    let ship = dsl
        .get_ships_by_player_id(&player_id)
        .next()
        .ok_or_else(|| format!("Player {} has no ship — cannot post to sector chat", username))?;
    let sector_id = SectorId::new(ship.get_sector_id().value());

    info!(
        "SectorChat #{} [{}]: {}",
        sector_id.value(),
        username,
        message
    );

    post_sector_channel(&dsl, sector_id, MessageSender::Player(sender), message)
}

/// Send a message to the caller's **Faction** channel.
///
/// Derived: the player's own faction. No need for a client-supplied faction id —
/// a player may only post to their own faction's channel.
#[spacetimedb::reducer]
pub fn send_faction_chat(ctx: &ReducerContext, message: String) -> Result<(), String> {
    let dsl = dsl(ctx);
    let sender = ctx.sender();
    let username = get_username(&dsl, sender);

    let player = dsl
        .get_player_by_id(PlayerId::new(sender))
        .map_err(|_| format!("Player {} is not registered — cannot post to faction chat", username))?;
    let faction_id = player.get_faction_id().clone();

    info!(
        "FactionChat #{} [{}]: {}",
        faction_id,
        username,
        message
    );

    post_faction_channel(&dsl, faction_id, MessageSender::Player(sender), message)
}
