#[cfg(test)]
mod tests {
    use crate::types::server_messages::ServerMessageType;

    #[test]
    fn test_server_message_type_display_string() {
        assert_eq!(ServerMessageType::Error.as_display_string(), "ERROR");
        assert_eq!(ServerMessageType::Info.as_display_string(), "INFO");
        assert_eq!(ServerMessageType::Warning.as_display_string(), "WARNING");
        assert_eq!(ServerMessageType::Admin.as_display_string(), "ADMIN");
        assert_eq!(ServerMessageType::System.as_display_string(), "SYSTEM");
    }

    #[test]
    fn test_server_message_type_equality() {
        assert_eq!(ServerMessageType::Error, ServerMessageType::Error);
        assert_ne!(ServerMessageType::Error, ServerMessageType::Info);
        assert_ne!(ServerMessageType::Warning, ServerMessageType::Admin);
    }

    #[test]
    fn test_server_message_type_clone() {
        let msg_type = ServerMessageType::Warning;
        let cloned = msg_type.clone();
        assert_eq!(msg_type, cloned);
    }

    // Note: Tests for ServerMessage and ServerMessageRecipient structs require SpacetimeDB runtime
    // and cannot be run as standard unit tests. These would need to be integration tests
    // run within the SpacetimeDB environment.
}
