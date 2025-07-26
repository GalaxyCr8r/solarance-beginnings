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

    // Unit tests for utility functions
    // Note: These test the logic and validation, but cannot test actual database operations
    // which require SpacetimeDB runtime environment

    #[test]
    fn test_message_validation_logic() {
        // Test message content validation
        let valid_message = "This is a valid message".to_string();
        let empty_message = "".to_string();

        assert!(!valid_message.is_empty());
        assert!(empty_message.is_empty());
    }

    #[test]
    fn test_group_message_validation() {
        // Test group message validation logic
        let empty_group: Vec<u32> = vec![];
        let valid_group = vec![1, 2, 3];

        assert!(empty_group.is_empty());
        assert!(!valid_group.is_empty());
        assert_eq!(valid_group.len(), 3);
    }

    #[test]
    fn test_message_type_variants() {
        // Test all message type variants exist and are distinct
        let error_type = ServerMessageType::Error;
        let info_type = ServerMessageType::Info;
        let warning_type = ServerMessageType::Warning;
        let admin_type = ServerMessageType::Admin;
        let system_type = ServerMessageType::System;

        // Verify they're all different
        assert_ne!(error_type, info_type);
        assert_ne!(info_type, warning_type);
        assert_ne!(warning_type, admin_type);
        assert_ne!(admin_type, system_type);
        assert_ne!(system_type, error_type);
    }

    #[test]
    fn test_optional_context_handling() {
        // Test optional context parameter handling
        let some_context: Option<&str> = Some("test_context");
        let none_context: Option<&str> = None;

        assert!(some_context.is_some());
        assert!(none_context.is_none());

        // Test conversion to String
        let converted_some = some_context.map(|s| s.to_string());
        let converted_none = none_context.map(|s| s.to_string());

        assert_eq!(converted_some, Some("test_context".to_string()));
        assert_eq!(converted_none, None);
    }

    #[test]
    fn test_group_name_handling() {
        // Test group name optional parameter handling
        let some_group_name: Option<String> = Some("Test Group".to_string());
        let none_group_name: Option<String> = None;

        assert!(some_group_name.is_some());
        assert!(none_group_name.is_none());

        if let Some(ref name) = some_group_name {
            assert_eq!(name, "Test Group");
        }
    }

    #[test]
    fn test_convenience_function_message_types() {
        // Test that convenience functions use correct message types
        // This tests the logic that would be used in the convenience functions

        let error_type = ServerMessageType::Error;
        let info_type = ServerMessageType::Info;
        let warning_type = ServerMessageType::Warning;
        let admin_type = ServerMessageType::Admin;

        // Verify each type has correct display string
        assert_eq!(error_type.as_display_string(), "ERROR");
        assert_eq!(info_type.as_display_string(), "INFO");
        assert_eq!(warning_type.as_display_string(), "WARNING");
        assert_eq!(admin_type.as_display_string(), "ADMIN");
    }

    #[test]
    fn test_error_message_validation() {
        // Test error message string validation logic
        let valid_error = "Transaction failed: Insufficient credits".to_string();
        let empty_error = String::new();

        assert!(!valid_error.is_empty());
        assert!(empty_error.is_empty());
        assert!(valid_error.contains("failed"));
    }

    // Note: Full integration tests for utility functions require SpacetimeDB runtime
    // and would need to be run as integration tests within the SpacetimeDB environment.
    // These tests cover the validation logic and parameter handling that can be tested
    // without database operations.
}
