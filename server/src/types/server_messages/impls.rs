use super::*;

impl ServerMessage {
    /// Creates a new server message with the current timestamp
    pub fn new(
        message: String,
        message_type: ServerMessageType,
        group_name: Option<String>,
        sender_context: Option<String>,
    ) -> Self {
        Self {
            id: 0, // Will be auto-incremented by SpacetimeDB
            message,
            message_type,
            group_name,
            sender_context,
            created_at: Timestamp::now(),
        }
    }
}

impl ServerMessageRecipient {
    /// Creates a new message recipient record
    pub fn new(server_message_id: u64, player_id: Identity) -> Self {
        Self {
            id: 0, // Will be auto-incremented by SpacetimeDB
            server_message_id,
            player_id,
            read_at: None,
            delivered_at: Timestamp::now(),
        }
    }

    /// Marks the message as read
    pub fn mark_as_read(&mut self) {
        self.read_at = Some(Timestamp::now());
    }

    /// Checks if the message has been read
    pub fn is_read(&self) -> bool {
        self.read_at.is_some()
    }
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
