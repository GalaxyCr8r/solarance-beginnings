use spacetimedb::ReducerContext;
use spacetimedsl::dsl;

use super::*;
use crate::types::players::PlayerId;

/// Send message to individual player (server-only)
pub fn send_server_message_to_player(
    ctx: &ReducerContext,
    player_id: &PlayerId,
    message: String,
    message_type: ServerMessageType,
    sender_context: Option<String>,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Create the server message
    let server_message = dsl.create_server_message(
        &message,
        message_type,
        None, // No group name for individual messages
        sender_context,
    )?;

    // Create recipient record
    dsl.create_server_message_recipient(
        ServerMessageId::new(server_message.id),
        player_id.clone(),
        None, // read_at starts as None
        spacetimedb::Timestamp::now(),
    )?;

    Ok(())
}

/// Send message to multiple players with optional group name (server-only)
pub fn send_server_message_to_group(
    ctx: &ReducerContext,
    player_ids: Vec<PlayerId>,
    message: String,
    message_type: ServerMessageType,
    group_name: Option<String>,
    sender_context: Option<String>,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if player_ids.is_empty() {
        return Err("Cannot send message to empty group".to_string());
    }

    // Create the server message
    let server_message =
        dsl.create_server_message(&message, message_type, group_name, sender_context)?;

    // Create recipient records for each player
    for player_id in player_ids {
        dsl.create_server_message_recipient(
            ServerMessageId::new(server_message.id),
            player_id,
            None, // read_at starts as None
            spacetimedb::Timestamp::now(),
        )?;
    }

    Ok(())
}

/// Convenience function for error messages from reducers (server-only)
pub fn send_error_message(
    ctx: &ReducerContext,
    player_id: &PlayerId,
    message: String,
    action_context: Option<&str>,
) -> Result<(), String> {
    send_server_message_to_player(
        ctx,
        player_id,
        message,
        ServerMessageType::Error,
        action_context.map(|s| s.to_string()),
    )
}

/// Convenience function for info messages from reducers (server-only)
pub fn send_info_message(
    ctx: &ReducerContext,
    player_id: &PlayerId,
    message: String,
    action_context: Option<&str>,
) -> Result<(), String> {
    send_server_message_to_player(
        ctx,
        player_id,
        message,
        ServerMessageType::Info,
        action_context.map(|s| s.to_string()),
    )
}

/// Convenience function for warning messages from reducers (server-only)
pub fn send_warning_message(
    ctx: &ReducerContext,
    player_id: &PlayerId,
    message: String,
    action_context: Option<&str>,
) -> Result<(), String> {
    send_server_message_to_player(
        ctx,
        player_id,
        message,
        ServerMessageType::Warning,
        action_context.map(|s| s.to_string()),
    )
}

/// Convenience function for admin messages from reducers (server-only)
pub fn send_admin_message(
    ctx: &ReducerContext,
    player_id: &PlayerId,
    message: String,
    action_context: Option<&str>,
) -> Result<(), String> {
    send_server_message_to_player(
        ctx,
        player_id,
        message,
        ServerMessageType::Admin,
        action_context.map(|s| s.to_string()),
    )
}

/// Get unread message count for a player
pub fn get_unread_message_count(ctx: &ReducerContext, player_id: &PlayerId) -> u64 {
    let dsl = dsl(ctx);

    dsl.get_server_message_recipients_by_player_id(player_id)
        .filter(|recipient| recipient.read_at.is_none())
        .count() as u64
}

/// Get all unread messages for a player
pub fn get_unread_messages_for_player(
    ctx: &ReducerContext,
    player_id: &PlayerId,
) -> Result<Vec<(ServerMessage, ServerMessageRecipient)>, String> {
    let dsl = dsl(ctx);

    let unread_recipients: Vec<ServerMessageRecipient> = dsl
        .get_server_message_recipients_by_player_id(player_id)
        .filter(|recipient| recipient.read_at.is_none())
        .collect();

    let mut messages_with_recipients = Vec::new();

    for recipient in unread_recipients {
        match dsl.get_server_message_by_id(ServerMessageId::new(recipient.server_message_id)) {
            Ok(message) => {
                messages_with_recipients.push((message, recipient));
            }
            Err(_) => {
                // Skip messages that can't be found (they may have been deleted)
                continue;
            }
        }
    }

    Ok(messages_with_recipients)
}

/// Mark a message as read for a specific player
pub fn mark_message_as_read(
    ctx: &ReducerContext,
    player_id: &PlayerId,
    server_message_id: u64,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Find the recipient record
    let recipient_opt = dsl
        .get_server_message_recipients_by_player_id(player_id)
        .find(|r| r.server_message_id == server_message_id);

    if let Some(mut recipient) = recipient_opt {
        recipient.read_at = Some(spacetimedb::Timestamp::now());
        dsl.update_server_message_recipient_by_id(recipient)?;
        Ok(())
    } else {
        Err("Message recipient not found".to_string())
    }
}
