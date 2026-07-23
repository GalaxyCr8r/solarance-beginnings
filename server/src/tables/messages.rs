//! Messaging — Channel Messages + Direct Server Messages (#101 redesign).
//!
//! Two systems, six private tables, exposed to clients only through STDB Views.
//! See `CONTEXT.md` §5 for the domain decisions; this file is the schema +
//! helpers + view bodies they reduce to.
//!
//! ## Channel Messages — one row, many readers, no read state
//! - `ServerChannelMessage`     — official MOTD/updates. **Public** (no view).
//! - `GalaxyChannelMessage`     — galaxy-wide player chat. Gated to logged-in.
//! - `StarSystemChannelMessage` — scoped to one StarSystem.
//! - `SectorChannelMessage`     — scoped to one Sector.
//! - `FactionChannelMessage`    — scoped to one Faction.
//!
//! ## Direct Server Messages — 1-to-1, server→player, async inbox
//! - `DirectServerMessage`      — server is always the sender (implicit, no enum).
//!
//! ## View legality (no `.iter()` full scans permitted)
//! Every View filters on an *indexed* column. Star-system/sector/faction/DM
//! pick the natural FK (`system_id`/`sector_id`/`faction_id`/`to`). Galaxy has
//! no natural key, so it carries a constant indexed `galaxy_id` (always `0`)
//! that the view filters on, plus an in-body "is the caller a player?" gate.

use crate::spacetimedsl::prelude::*;
use spacetimedb::{table, view, Identity, SpacetimeType, Timestamp, ViewContext};

use crate::tables::{
    factions::FactionId, players::PlayerId, sectors::SectorId, star_system::StarSystemId,
};

////////////////////////////////////////////////////////////////////////////////
// Enums
////////////////////////////////////////////////////////////////////////////////

/// Who composed a **Channel Message**. Lives only on channel tables — every
/// channel row may be from a Player or from the Server. `DirectServerMessage`
/// omits the sender field entirely (the Server is always the sender).
#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum MessageSender {
    Player(Identity),
    System,
}

/// Three-tier severity for `DirectServerMessage` only. Drives client filtering
/// + visual treatment; intentionally short so each tier remains meaningful.
///
/// **`Error` is deliberately omitted** — synchronous validation failures are
/// reducer `Err(String)` returns and surface through the `_then` callback, not
/// the inbox. The inbox is for *durable, async* events the player needs to
/// learn about (e.g. on return after several days away).
#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum MessageSeverity {
    /// Routine notices: welcome-back summary, "construction completed".
    Info,
    /// Important but non-blocking: "your station is low on fuel".
    Warning,
    /// Loss / urgent: "your ship was destroyed", "your station was captured".
    Critical,
}

////////////////////////////////////////////////////////////////////////////////
// Channel Messages
////////////////////////////////////////////////////////////////////////////////

/// Server-wide MOTD / official updates. Sender is implicitly the Server.
/// **Public** table — readable by anyone connected (including not-logged-in
/// and banned identities), and is the only message table safe to surface
/// outside the game client (e.g. a website status page).
#[spacetimedsl::dsl(plural_name = server_channel_messages, method(update = false))]
#[table(accessor = server_channel_message, public)]
pub struct ServerChannelMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    body: String,
    created_at: Timestamp,
}

/// Galaxy-wide player chat. Private table; view-gated to logged-in players.
/// `galaxy_id` is a constant (`0`) indexed column whose only job is to keep
/// the view legal under the "no full scan" rule — single-galaxy MVP, but the
/// column is real domain shape so it survives the eventual multi-galaxy add.
#[spacetimedsl::dsl(plural_name = galaxy_channel_messages, method(update = false))]
#[table(accessor = galaxy_channel_message)]
pub struct GalaxyChannelMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    galaxy_id: u32,

    sender: MessageSender,
    body: String,
    created_at: Timestamp,
}

/// Scoped to one StarSystem. Audience: players whose Ship is in a Sector of
/// that StarSystem. In MVP one StarSystem exists, so this is effectively
/// galaxy-wide today; correctly scopes when system #2 ships.
#[spacetimedsl::dsl(plural_name = star_system_channel_messages, method(update = false))]
#[table(accessor = star_system_channel_message)]
pub struct StarSystemChannelMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(crate::tables::star_system::StarSystemId)]
    #[foreign_key(path = crate::tables::star_system, table = star_system, column = id, on_delete = Delete)]
    system_id: u32,

    sender: MessageSender,
    body: String,
    created_at: Timestamp,
}

/// Scoped to one Sector. Audience: players whose Ship is in that Sector.
#[spacetimedsl::dsl(plural_name = sector_channel_messages, method(update = false))]
#[table(accessor = sector_channel_message)]
pub struct SectorChannelMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(crate::tables::sectors::SectorId)]
    #[foreign_key(path = crate::tables::sectors, table = sector, column = id, on_delete = Delete)]
    sector_id: u64,

    sender: MessageSender,
    body: String,
    created_at: Timestamp,
}

/// Scoped to one Faction. Audience: players of that Faction.
#[spacetimedsl::dsl(plural_name = faction_channel_messages, method(update = false))]
#[table(accessor = faction_channel_message)]
pub struct FactionChannelMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(crate::tables::factions::FactionId)]
    #[foreign_key(path = crate::tables::factions, table = faction, column = id, on_delete = Error)]
    faction_id: u32,

    sender: MessageSender,
    body: String,
    created_at: Timestamp,
}

////////////////////////////////////////////////////////////////////////////////
// Direct Server Messages — 1-to-1, server→player, async inbox
////////////////////////////////////////////////////////////////////////////////

/// 1-to-1 message FROM the Server TO a single Player. Sender is implicit —
/// there is no `sender` field. Replaces the old `ServerMessage` +
/// `ServerMessageRecipient` pair.
///
/// **Read state is *not* stored here.** A message is "unread" if its
/// `created_at` is later than the player's `last_login`; the client highlights
/// those and clears the highlight when the player next sends a chat message.
#[spacetimedsl::dsl(plural_name = direct_server_messages, method(update = false))]
#[table(accessor = direct_server_message)]
pub struct DirectServerMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    /// Recipient Player identity. Indexed — the view filters here.
    #[index(btree)]
    #[use_wrapper(crate::tables::players::PlayerId)]
    #[foreign_key(path = crate::tables::players, table = player, column = id, on_delete = Delete)]
    to: Identity,

    severity: MessageSeverity,
    body: String,
    created_at: Timestamp,
}

////////////////////////////////////////////////////////////////////////////////
// Views — every channel except Server (which is plain public) gets one.
//
// All five are per-caller (`ViewContext`) because the audience is identity-
// derived. Per the STDB docs this is O(subscribers); fine at <10 players.
////////////////////////////////////////////////////////////////////////////////

/// Direct Server Messages addressed to the caller.
#[view(accessor = my_direct_server_messages, public)]
pub fn my_direct_server_messages(ctx: &ViewContext) -> Vec<DirectServerMessage> {
    let sender = ctx.sender();
    ctx.db
        .direct_server_message()
        .to()
        .filter(&sender)
        .collect()
}

/// Galaxy chat — gated to identities that resolve to a Player row.
/// Non-players (not-logged-in / banned) get `vec![]`.
#[view(accessor = my_galaxy_chat, public)]
pub fn my_galaxy_chat(ctx: &ViewContext) -> Vec<GalaxyChannelMessage> {
    let dsl = read_only_dsl(ctx);
    if dsl.get_player_by_id(PlayerId::new(ctx.sender())).is_err() {
        return Vec::new();
    }
    ctx.db
        .galaxy_channel_message()
        .galaxy_id()
        .filter(&0u32)
        .collect()
}

/// StarSystem chat for the caller's *current* StarSystem (derived via their
/// ship's sector). No ship → no view contents.
#[view(accessor = my_star_system_chat, public)]
pub fn my_star_system_chat(ctx: &ViewContext) -> Vec<StarSystemChannelMessage> {
    let dsl = read_only_dsl(ctx);
    let ship = match dsl
        .get_ships_by_player_id(&PlayerId::new(ctx.sender()))
        .next()
    {
        Some(s) => s,
        None => return Vec::new(),
    };
    let sector = match dsl.get_sector_by_id(&ship.get_sector_id().clone()) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let system_id = sector.get_system_id().value();
    ctx.db
        .star_system_channel_message()
        .system_id()
        .filter(&system_id)
        .collect()
}

/// Sector chat for the caller's *current* Sector (derived via their ship).
#[view(accessor = my_sector_chat, public)]
pub fn my_sector_chat(ctx: &ViewContext) -> Vec<SectorChannelMessage> {
    let dsl = read_only_dsl(ctx);
    let ship = match dsl
        .get_ships_by_player_id(&PlayerId::new(ctx.sender()))
        .next()
    {
        Some(s) => s,
        None => return Vec::new(),
    };
    let sector_id = ship.get_sector_id().value();
    ctx.db
        .sector_channel_message()
        .sector_id()
        .filter(&sector_id)
        .collect()
}

/// Faction chat for the caller's Faction.
#[view(accessor = my_faction_chat, public)]
pub fn my_faction_chat(ctx: &ViewContext) -> Vec<FactionChannelMessage> {
    let dsl = read_only_dsl(ctx);
    let player = match dsl.get_player_by_id(PlayerId::new(ctx.sender())) {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };
    let faction_id = player.get_faction_id().value();
    ctx.db
        .faction_channel_message()
        .faction_id()
        .filter(&faction_id)
        .collect()
}

////////////////////////////////////////////////////////////////////////////////
// Send helpers — keep callers stating intent, not steps. Reducers reach for
// these instead of `dsl.create_*_channel_message(...)` directly.
////////////////////////////////////////////////////////////////////////////////

/// Send a Direct Server Message to one player. Server is implicit sender.
pub fn send_direct_server_message<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    to: &PlayerId,
    severity: MessageSeverity,
    body: String,
) -> Result<(), String> {
    dsl.create_direct_server_message(CreateDirectServerMessage {
        to: to.clone(),
        severity,
        body,
    })?;
    Ok(())
}

/// Convenience: Info-severity DSM.
pub fn send_direct_server_info<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    to: &PlayerId,
    body: String,
) -> Result<(), String> {
    send_direct_server_message(dsl, to, MessageSeverity::Info, body)
}

/// Convenience: Warning-severity DSM.
pub fn send_direct_server_warning<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    to: &PlayerId,
    body: String,
) -> Result<(), String> {
    send_direct_server_message(dsl, to, MessageSeverity::Warning, body)
}

/// Convenience: Critical-severity DSM.
pub fn send_direct_server_critical<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    to: &PlayerId,
    body: String,
) -> Result<(), String> {
    send_direct_server_message(dsl, to, MessageSeverity::Critical, body)
}

/// Post an official Server-channel message (MOTD / updates).
pub fn post_server_channel<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    body: String,
) -> Result<(), String> {
    dsl.create_server_channel_message(CreateServerChannelMessage { body })?;
    Ok(())
}

/// Post to the Galaxy channel.
pub fn post_galaxy_channel<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    sender: MessageSender,
    body: String,
) -> Result<(), String> {
    dsl.create_galaxy_channel_message(CreateGalaxyChannelMessage {
        galaxy_id: 0,
        sender,
        body,
    })?;
    Ok(())
}

/// Post to a StarSystem channel.
pub fn post_star_system_channel<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    system_id: StarSystemId,
    sender: MessageSender,
    body: String,
) -> Result<(), String> {
    dsl.create_star_system_channel_message(CreateStarSystemChannelMessage {
        system_id,
        sender,
        body,
    })?;
    Ok(())
}

/// Post to a Sector channel.
pub fn post_sector_channel<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    sector_id: SectorId,
    sender: MessageSender,
    body: String,
) -> Result<(), String> {
    dsl.create_sector_channel_message(CreateSectorChannelMessage {
        sector_id,
        sender,
        body,
    })?;
    Ok(())
}

/// Post to a Faction channel.
pub fn post_faction_channel<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_id: FactionId,
    sender: MessageSender,
    body: String,
) -> Result<(), String> {
    dsl.create_faction_channel_message(CreateFactionChannelMessage {
        faction_id,
        sender,
        body,
    })?;
    Ok(())
}
