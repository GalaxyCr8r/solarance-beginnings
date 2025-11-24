use spacetimedb::{Identity, ReducerContext};
use spacetimedsl::*;

use crate::{
    tables::{players::PlayerId, server_messages::*},
    utility::try_server_only,
};

/// Mark a server message as read by the calling player
#[spacetimedb::reducer]
pub fn mark_server_message_as_read(
    ctx: &ReducerContext,
    server_message_id: u64,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    let player_id = PlayerId::new(dsl.ctx().sender);

    // Find the recipient record
    let recipient_opt = dsl
        .get_server_message_recipients_by_player_id(player_id)
        .find(|r| r.get_server_message_id().value() == server_message_id);

    if let Some(mut recipient) = recipient_opt {
        recipient.set_read_at(Some(dsl.ctx().timestamp));
        dsl.update_server_message_recipient_by_id(recipient)?;
        Ok(())
    } else {
        Err("Message recipient not found".to_string())
    }
}

/// Administrative reducer for sending targeted messages (server-only)
/// Supports both individual and group message targeting with proper authorization
#[spacetimedb::reducer]
pub fn send_admin_message(
    ctx: &ReducerContext,
    target_player_ids: Vec<Identity>,
    message: String,
    message_type: ServerMessageType,
    group_name: Option<String>,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    // Authorization check - only server can send admin messages
    try_server_only(&dsl)?;

    // Validate input parameters
    if target_player_ids.is_empty() {
        return Err("Cannot send admin message to empty recipient list".to_string());
    }

    if message.trim().is_empty() {
        return Err("Cannot send empty admin message".to_string());
    }

    // Convert Identity to PlayerId
    let player_ids: Vec<PlayerId> = target_player_ids.into_iter().map(PlayerId::new).collect();

    // Send message using utility function
    send_server_message_to_group(
        &dsl,
        player_ids,
        message,
        message_type,
        group_name,
        Some("Admin".to_string()),
    )
}

/// Administrative reducer for sending a message to a single player (server-only)
/// Convenience function for individual player targeting
#[spacetimedb::reducer]
pub fn send_admin_message_to_player(
    ctx: &ReducerContext,
    target_player_id: Identity,
    message: String,
    message_type: ServerMessageType,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    // Authorization check - only server can send admin messages
    try_server_only(&dsl)?;

    // Validate input parameters
    if message.trim().is_empty() {
        return Err("Cannot send empty admin message".to_string());
    }

    let player_id = PlayerId::new(target_player_id);

    // Send message using utility function
    send_server_message_to_player(
        &dsl,
        &player_id,
        message,
        message_type,
        Some("Admin".to_string()),
    )
}
