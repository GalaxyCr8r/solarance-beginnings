use spacetimedb::{table, Identity, SpacetimeType, Timestamp};
use spacetimedsl::*;

use crate::tables::players::PlayerId;

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum ServerMessageType {
    Error,   // Action failures, validation errors
    Info,    // General information
    Warning, // Important notices
    Admin,   // Administrative messages
    System,  // System-generated notifications
}

impl ServerMessageType {
    /// Returns a string representation of the message type for display
    pub fn as_display_string(&self) -> &'static str {
        match self {
            ServerMessageType::Error => "ERROR",
            ServerMessageType::Info => "INFO",
            ServerMessageType::Warning => "WARNING",
            ServerMessageType::Admin => "ADMIN",
            ServerMessageType::System => "SYSTEM",
        }
    }
}

#[dsl(plural_name = server_messages, method(update = true))]
#[table(name = server_message, public)]
pub struct ServerMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::tables::server_messages, table = server_message_recipient)]
    id: u64,

    pub message: String,
    pub message_type: ServerMessageType,
    pub group_name: Option<String>,     // For group messages
    pub sender_context: Option<String>, // Context about what action triggered this

    created_at: Timestamp,
}

impl ServerMessage {
    /// Creates a new server message with the current timestamp
    pub fn new(
        message: String,
        message_type: ServerMessageType,
        group_name: Option<String>,
        sender_context: Option<String>,
        now: &Timestamp,
    ) -> Self {
        Self {
            id: 0, // Will be auto-incremented by SpacetimeDB
            message,
            message_type,
            group_name,
            sender_context,
            created_at: now.clone(),
        }
    }
}

#[dsl(plural_name = server_message_recipients, method(update = true))]
#[table(name = server_message_recipient, public)]
pub struct ServerMessageRecipient {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(crate::tables::server_messages::ServerMessageId)]
    #[foreign_key(path = crate::tables::server_messages, table = server_message, column = id, on_delete = Delete)]
    pub server_message_id: u64,

    #[index(btree)]
    #[use_wrapper(crate::tables::players::PlayerId)]
    #[foreign_key(path = crate::tables::players, table = player, column = id, on_delete = Delete)]
    pub player_id: Identity,

    pub read_at: Option<Timestamp>,
    pub delivered_at: Timestamp,
}

impl ServerMessageRecipient {
    /// Creates a new message recipient record
    pub fn new(server_message_id: u64, player_id: Identity, now: &Timestamp) -> Self {
        Self {
            id: 0, // Will be auto-incremented by SpacetimeDB
            server_message_id,
            player_id,
            read_at: None,
            delivered_at: now.clone(),
        }
    }

    /// Marks the message as read
    pub fn mark_as_read(&mut self, now: &Timestamp) {
        self.read_at = Some(now.clone());
    }

    /// Checks if the message has been read
    pub fn is_read(&self) -> bool {
        self.read_at.is_some()
    }
}

////////////////////////////////////////////////////
/// Utility
///

/// Send message to individual player (server-only)
pub fn send_server_message_to_player<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player_id: &PlayerId,
    message: String,
    message_type: ServerMessageType,
    sender_context: Option<String>,
) -> Result<(), String> {
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
        dsl.ctx().timestamp(),
    )?;

    Ok(())
}

/// Send message to multiple players with optional group name (server-only)
pub fn send_server_message_to_group<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player_ids: Vec<PlayerId>,
    message: String,
    message_type: ServerMessageType,
    group_name: Option<String>,
    sender_context: Option<String>,
) -> Result<(), String> {
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
            dsl.ctx().timestamp(),
        )?;
    }

    Ok(())
}

/// Convenience function for error messages from reducers (server-only)
pub fn send_error_message<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player_id: &PlayerId,
    message: String,
    action_context: Option<&str>,
) -> Result<(), String> {
    send_server_message_to_player(
        dsl,
        player_id,
        message,
        ServerMessageType::Error,
        action_context.map(|s| s.to_string()),
    )
}

/// Convenience function for info messages from reducers (server-only)
pub fn send_info_message<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player_id: &PlayerId,
    message: String,
    action_context: Option<&str>,
) -> Result<(), String> {
    send_server_message_to_player(
        dsl,
        player_id,
        message,
        ServerMessageType::Info,
        action_context.map(|s| s.to_string()),
    )
}

/// Convenience function for warning messages from reducers (server-only)
pub fn send_warning_message<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player_id: &PlayerId,
    message: String,
    action_context: Option<&str>,
) -> Result<(), String> {
    send_server_message_to_player(
        dsl,
        player_id,
        message,
        ServerMessageType::Warning,
        action_context.map(|s| s.to_string()),
    )
}

/// Convenience function for admin messages from reducers (server-only)
pub fn send_admin_message<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player_id: &PlayerId,
    message: String,
    action_context: Option<&str>,
) -> Result<(), String> {
    send_server_message_to_player(
        dsl,
        player_id,
        message,
        ServerMessageType::Admin,
        action_context.map(|s| s.to_string()),
    )
}

/// Get unread message count for a player
pub fn get_unread_message_count<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player_id: &PlayerId,
) -> u64 {
    dsl.get_server_message_recipients_by_player_id(player_id)
        .filter(|recipient| recipient.read_at.is_none())
        .count() as u64
}

/// Get all unread messages for a player
pub fn get_unread_messages_for_player<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player_id: &PlayerId,
) -> Result<Vec<(ServerMessage, ServerMessageRecipient)>, String> {
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
pub fn mark_message_as_read<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player_id: &PlayerId,
    server_message_id: u64,
) -> Result<(), String> {
    // Find the recipient record
    let recipient_opt = dsl
        .get_server_message_recipients_by_player_id(player_id)
        .find(|r| r.server_message_id == server_message_id);

    if let Some(mut recipient) = recipient_opt {
        recipient.read_at = Some(dsl.ctx().timestamp());
        dsl.update_server_message_recipient_by_id(recipient)?;
        Ok(())
    } else {
        Err("Message recipient not found".to_string())
    }
}
