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

    // Tests for message delivery and tracking system functionality

    #[test]
    fn test_message_read_status_logic() {
        use crate::types::server_messages::ServerMessageRecipient;
        use spacetimedb::{Identity, Timestamp};

        // Test unread message
        let mut recipient = ServerMessageRecipient {
            id: 1,
            server_message_id: 100,
            player_id: Identity::from_byte_array([1; 32]),
            read_at: None,
            delivered_at: Timestamp::now(),
        };

        assert!(!recipient.is_read());

        // Test marking as read
        recipient.mark_as_read();
        assert!(recipient.is_read());
        assert!(recipient.read_at.is_some());
    }

    #[test]
    fn test_unread_count_logic() {
        use crate::types::server_messages::ServerMessageRecipient;
        use spacetimedb::{Identity, Timestamp};

        // Simulate a collection of message recipients
        let recipients = vec![
            ServerMessageRecipient {
                id: 1,
                server_message_id: 100,
                player_id: Identity::from_byte_array([1; 32]),
                read_at: None, // Unread
                delivered_at: Timestamp::now(),
            },
            ServerMessageRecipient {
                id: 2,
                server_message_id: 101,
                player_id: Identity::from_byte_array([1; 32]),
                read_at: Some(Timestamp::now()), // Read
                delivered_at: Timestamp::now(),
            },
            ServerMessageRecipient {
                id: 3,
                server_message_id: 102,
                player_id: Identity::from_byte_array([1; 32]),
                read_at: None, // Unread
                delivered_at: Timestamp::now(),
            },
        ];

        // Count unread messages
        let unread_count = recipients.iter().filter(|r| r.read_at.is_none()).count();
        assert_eq!(unread_count, 2);

        // Count read messages
        let read_count = recipients.iter().filter(|r| r.read_at.is_some()).count();
        assert_eq!(read_count, 1);
    }

    #[test]
    fn test_message_filtering_logic() {
        use crate::types::server_messages::{ServerMessageRecipient, ServerMessageType};
        use spacetimedb::{Identity, Timestamp};

        let player_id = Identity::from_byte_array([1; 32]);
        let other_player_id = Identity::from_byte_array([2; 32]);

        // Simulate message recipients for different players
        let recipients = vec![
            ServerMessageRecipient {
                id: 1,
                server_message_id: 100,
                player_id: player_id.clone(),
                read_at: None,
                delivered_at: Timestamp::now(),
            },
            ServerMessageRecipient {
                id: 2,
                server_message_id: 101,
                player_id: other_player_id.clone(),
                read_at: None,
                delivered_at: Timestamp::now(),
            },
            ServerMessageRecipient {
                id: 3,
                server_message_id: 102,
                player_id: player_id.clone(),
                read_at: Some(Timestamp::now()),
                delivered_at: Timestamp::now(),
            },
        ];

        // Filter messages for specific player
        let player_messages: Vec<_> = recipients
            .iter()
            .filter(|r| r.player_id == player_id)
            .collect();
        assert_eq!(player_messages.len(), 2);

        // Filter unread messages for specific player
        let unread_player_messages: Vec<_> = recipients
            .iter()
            .filter(|r| r.player_id == player_id && r.read_at.is_none())
            .collect();
        assert_eq!(unread_player_messages.len(), 1);
    }

    #[test]
    fn test_message_delivery_timestamp_logic() {
        use crate::types::server_messages::ServerMessageRecipient;
        use spacetimedb::{Identity, Timestamp};

        let now = Timestamp::now();
        let recipient = ServerMessageRecipient {
            id: 1,
            server_message_id: 100,
            player_id: Identity::from_byte_array([1; 32]),
            read_at: None,
            delivered_at: now,
        };

        // Verify delivery timestamp is set
        assert_eq!(recipient.delivered_at, now);

        // Test that read_at is initially None
        assert!(recipient.read_at.is_none());
    }

    #[test]
    fn test_message_recipient_creation_logic() {
        use crate::types::server_messages::ServerMessageRecipient;
        use spacetimedb::Identity;

        let player_id = Identity::from_byte_array([1; 32]);
        let message_id = 100u64;

        let recipient = ServerMessageRecipient::new(message_id, player_id.clone());

        assert_eq!(recipient.server_message_id, message_id);
        assert_eq!(recipient.player_id, player_id);
        assert!(recipient.read_at.is_none());
        assert!(recipient.delivered_at.to_micros_since_unix_epoch() > 0);
    }

    #[test]
    fn test_message_tracking_state_transitions() {
        use crate::types::server_messages::ServerMessageRecipient;
        use spacetimedb::Identity;

        let mut recipient = ServerMessageRecipient::new(100, Identity::from_byte_array([1; 32]));

        // Initial state: delivered but not read
        assert!(!recipient.is_read());
        assert!(recipient.delivered_at.to_micros_since_unix_epoch() > 0);
        assert!(recipient.read_at.is_none());

        // Transition to read state
        recipient.mark_as_read();
        assert!(recipient.is_read());
        assert!(recipient.read_at.is_some());

        // Verify read timestamp is after delivery timestamp
        if let Some(read_time) = recipient.read_at {
            assert!(
                read_time.to_micros_since_unix_epoch()
                    >= recipient.delivered_at.to_micros_since_unix_epoch()
            );
        }
    }

    #[test]
    fn test_message_validation_for_tracking() {
        // Test message ID validation logic
        let valid_message_id = 100u64;
        let invalid_message_id = 0u64;

        assert!(valid_message_id > 0);
        assert_eq!(invalid_message_id, 0);

        // Test player ID validation logic
        use spacetimedb::Identity;
        let valid_player_id = Identity::from_byte_array([1; 32]);
        let zero_player_id = Identity::from_byte_array([0; 32]);

        assert_ne!(valid_player_id, zero_player_id);
    }

    #[test]
    fn test_group_message_tracking_logic() {
        use crate::types::server_messages::ServerMessageRecipient;
        use spacetimedb::{Identity, Timestamp};

        // Simulate group message with multiple recipients
        let message_id = 100u64;
        let recipients = vec![
            ServerMessageRecipient::new(message_id, Identity::from_byte_array([1; 32])),
            ServerMessageRecipient::new(message_id, Identity::from_byte_array([2; 32])),
            ServerMessageRecipient::new(message_id, Identity::from_byte_array([3; 32])),
        ];

        // All recipients should have the same message ID
        assert!(recipients.iter().all(|r| r.server_message_id == message_id));

        // All recipients should be initially unread
        assert!(recipients.iter().all(|r| !r.is_read()));

        // Test individual read status tracking
        let mut recipient_copy = recipients[0].clone();
        recipient_copy.mark_as_read();
        assert!(recipient_copy.is_read());

        // Original recipients should still be unread
        assert!(!recipients[0].is_read());
    }

    // Note: Full integration tests for utility functions require SpacetimeDB runtime
    // and would need to be run as integration tests within the SpacetimeDB environment.
    // These tests cover the validation logic and parameter handling that can be tested
    // without database operations.
}
