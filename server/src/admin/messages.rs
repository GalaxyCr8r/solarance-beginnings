//! Admin-only reducers for sending Direct Server Messages.
//!
//! Authorization: server-only via `try_server_only` — only the module itself can
//! call these (the typical use case is a debug pipeline or scripted admin tool
//! that schedules a reducer call with the module identity).
//!
//! Per #101 redesign:
//! - `DirectServerMessage` is 1-to-1; "send to many" loops over recipients.
//! - There is no per-message read state — the old `mark_*_as_read` reducer is gone.
//! - Server is the implicit sender on every DSM, so no sender field is passed.

use spacetimedb::{Identity, ReducerContext};
use spacetimedsl::*;

use crate::{
    tables::{
        messages::{send_direct_server_message, MessageSeverity},
        players::PlayerId,
    },
    utility::try_server_only,
};

/// Send a Direct Server Message to one player (server-only).
#[spacetimedb::reducer]
pub fn admin_send_direct_server_message(
    ctx: &ReducerContext,
    target_player_id: Identity,
    severity: MessageSeverity,
    body: String,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    if body.trim().is_empty() {
        return Err("Cannot send empty admin message".to_string());
    }

    send_direct_server_message(&dsl, &PlayerId::new(target_player_id), severity, body)
}

/// Send the same Direct Server Message to a list of players (server-only).
/// Each recipient gets its own row — DSM is 1-to-1 by design.
#[spacetimedb::reducer]
pub fn admin_send_direct_server_message_to_group(
    ctx: &ReducerContext,
    target_player_ids: Vec<Identity>,
    severity: MessageSeverity,
    body: String,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    if target_player_ids.is_empty() {
        return Err("Cannot send admin message to empty recipient list".to_string());
    }

    if body.trim().is_empty() {
        return Err("Cannot send empty admin message".to_string());
    }

    for target in target_player_ids {
        send_direct_server_message(
            &dsl,
            &PlayerId::new(target),
            severity.clone(),
            body.clone(),
        )?;
    }
    Ok(())
}
