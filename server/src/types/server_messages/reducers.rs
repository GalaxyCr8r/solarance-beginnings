use spacetimedb::{Identity, ReducerContext};

use super::*;
use crate::types::{common::utility::try_server_only, players::PlayerId};

/// Mark a server message as read by the calling player
#[spacetimedb::reducer]
pub fn mark_server_message_as_read(
    ctx: &ReducerContext,
    server_message_id: u64,
) -> Result<(), String> {
    let player_id = PlayerId::new(ctx.sender);

    super::utility::mark_message_as_read(ctx, &player_id, server_message_id)
}

/// Get unread message count for a player (utility function, not a reducer)
/// Note: SpacetimeDB reducers cannot return values, so this is implemented as a utility function
/// that can be called from other reducers or accessed via table queries
pub fn get_unread_message_count_for_player(ctx: &ReducerContext, player_id: &PlayerId) -> u64 {
    super::utility::get_unread_message_count(ctx, player_id)
}

/// Get all unread messages for a player (utility function, not a reducer)
/// Note: SpacetimeDB reducers cannot return values, so this is implemented as a utility function
/// that can be called from other reducers or accessed via table queries
pub fn get_unread_messages_for_player_reducer(
    ctx: &ReducerContext,
    player_id: &PlayerId,
) -> Result<Vec<(ServerMessage, ServerMessageRecipient)>, String> {
    super::utility::get_unread_messages_for_player(ctx, player_id)
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
    // Authorization check - only server can send admin messages
    try_server_only(ctx)?;

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
    super::utility::send_server_message_to_group(
        ctx,
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
    // Authorization check - only server can send admin messages
    try_server_only(ctx)?;

    // Validate input parameters
    if message.trim().is_empty() {
        return Err("Cannot send empty admin message".to_string());
    }

    let player_id = PlayerId::new(target_player_id);

    // Send message using utility function
    super::utility::send_server_message_to_player(
        ctx,
        &player_id,
        message,
        message_type,
        Some("Admin".to_string()),
    )
}
