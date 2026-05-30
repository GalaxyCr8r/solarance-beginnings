use crate::server::bindings::*;
use spacetimedb_sdk::{DbContext, Identity, Table};

/// Discriminator the server stamps on the welcome-back message's
/// `sender_context` (see `server/src/logic/players/welcome_back.rs`,
/// `WELCOME_BACK_CONTEXT`). Keep in sync with the server-side constant — it is
/// how the client tells a welcome-back apart from a generic notification.
pub const WELCOME_BACK_CONTEXT: &str = "welcome_back";

/// Client-side utilities for handling server messages
pub struct ServerMessageUtils;

impl ServerMessageUtils {
    /// Get all server messages for the current player
    pub fn get_messages_for_player(
        ctx: &DbConnection,
        player_id: &Identity,
    ) -> Vec<(ServerMessage, ServerMessageRecipient)> {
        let mut messages = Vec::new();

        // Get all message recipients for this player
        for recipient in ctx.db().server_message_recipient().iter() {
            if recipient.player_id == *player_id {
                // Find the corresponding server message
                if let Some(message) = ctx
                    .db()
                    .server_message()
                    .id()
                    .find(&recipient.server_message_id)
                {
                    messages.push((message, recipient));
                }
            }
        }

        // Sort by creation time (newest first)
        messages.sort_by(|a, b| b.0.created_at.cmp(&a.0.created_at));

        messages
    }

    /// Get unread server messages for the current player
    pub fn get_unread_messages_for_player(
        ctx: &DbConnection,
        player_id: &Identity,
    ) -> Vec<(ServerMessage, ServerMessageRecipient)> {
        Self::get_messages_for_player(ctx, player_id)
            .into_iter()
            .filter(|(_, recipient)| recipient.read_at.is_none())
            .collect()
    }

    /// Get count of unread server messages for the current player
    pub fn get_unread_count(ctx: &DbConnection, player_id: &Identity) -> usize {
        Self::get_unread_messages_for_player(ctx, player_id).len()
    }

    /// Find the most recent welcome-back message for the current player, if any.
    ///
    /// The welcome-back panel (#100) renders this; it is the server-composed
    /// `ServerMessage` tagged with [`WELCOME_BACK_CONTEXT`]. Returns the newest
    /// one so a reconnect supersedes any stale welcome-back left unread from a
    /// previous session.
    pub fn get_latest_welcome_back(
        ctx: &DbConnection,
        player_id: &Identity,
    ) -> Option<(ServerMessage, ServerMessageRecipient)> {
        // `get_messages_for_player` already returns newest-first.
        Self::get_messages_for_player(ctx, player_id)
            .into_iter()
            .find(|(message, _)| {
                message.sender_context.as_deref() == Some(WELCOME_BACK_CONTEXT)
            })
    }

    /// Mark a server message as read
    pub fn mark_message_as_read(ctx: &DbConnection, server_message_id: u64) -> Result<(), String> {
        ctx.reducers
            .mark_server_message_as_read(server_message_id)
            .map_err(|e| format!("Failed to mark message as read: {}", e))
    }

    /// Format timestamp for display
    pub fn format_timestamp(timestamp: &spacetimedb_sdk::Timestamp) -> String {
        // For now, just show a simple time format
        // In a real implementation, you might want to format this more nicely
        format!("{:?}", timestamp)
    }

    /// Get a shorter timestamp format for inline display
    pub fn format_timestamp_short(timestamp: &spacetimedb_sdk::Timestamp) -> String {
        // // Extract just the time portion for a cleaner display
        // let full_time = format!("{:?}", timestamp);

        // // This is a simplified approach - in production you'd want proper time formatting
        // if full_time.len() > 10 {
        //     full_time[11..19].to_string() // Extract HH:MM:SS portion
        // } else {
        //     full_time
        // }
        match timestamp.to_rfc3339() {
            Ok(datetime) => datetime.as_str()[..16].to_string(), // 2025-05-05T11:33
            Err(_) => "n/a".to_string(),
        }
    }

    /// Format a server message for display with enhanced formatting
    pub fn format_message_for_display(message: &ServerMessage) -> String {
        let type_prefix = match message.message_type {
            ServerMessageType::Error => "[ERROR]",
            ServerMessageType::Info => "[INFO]",
            ServerMessageType::Warning => "[WARNING]",
            ServerMessageType::Admin => "[ADMIN]",
            ServerMessageType::System => "[SYSTEM]",
        };

        let context_part = if let Some(context) = &message.sender_context {
            format!(" {}: ", context)
        } else {
            " ".to_string()
        };

        let group_part = if let Some(group_name) = &message.group_name {
            format!(" (Group: {})", group_name)
        } else {
            "".to_string()
        };

        format!(
            "{}{}{}{}",
            type_prefix, context_part, message.message, group_part
        )
    }

    /// Get color for message type
    pub fn get_message_color(message_type: &ServerMessageType) -> egui::Color32 {
        match message_type {
            ServerMessageType::Error => egui::Color32::from_rgb(220, 50, 50), // Red
            ServerMessageType::Info => egui::Color32::from_rgb(50, 150, 220), // Blue
            ServerMessageType::Warning => egui::Color32::from_rgb(255, 165, 0), // Orange
            ServerMessageType::Admin => egui::Color32::from_rgb(128, 0, 128), // Purple
            ServerMessageType::System => egui::Color32::from_rgb(100, 100, 100), // Gray
        }
    }

    /// Check if message is urgent (should be highlighted)
    pub fn is_urgent_message(message_type: &ServerMessageType) -> bool {
        matches!(
            message_type,
            ServerMessageType::Error | ServerMessageType::Warning | ServerMessageType::Admin
        )
    }
}
