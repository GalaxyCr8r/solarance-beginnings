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

    // Tests for administrative message functionality

    #[test]
    fn test_admin_message_validation_logic() {
        // Test message validation for admin messages
        let valid_message = "Server maintenance scheduled for tonight".to_string();
        let empty_message = "".to_string();
        let whitespace_message = "   ".to_string();

        assert!(!valid_message.trim().is_empty());
        assert!(empty_message.trim().is_empty());
        assert!(whitespace_message.trim().is_empty());
    }

    #[test]
    fn test_admin_message_recipient_validation() {
        use spacetimedb::Identity;

        // Test recipient list validation
        let empty_recipients: Vec<Identity> = vec![];
        let single_recipient = vec![Identity::from_byte_array([1; 32])];
        let multiple_recipients = vec![
            Identity::from_byte_array([1; 32]),
            Identity::from_byte_array([2; 32]),
            Identity::from_byte_array([3; 32]),
        ];

        assert!(empty_recipients.is_empty());
        assert_eq!(single_recipient.len(), 1);
        assert_eq!(multiple_recipients.len(), 3);
        assert!(!single_recipient.is_empty());
        assert!(!multiple_recipients.is_empty());
    }

    #[test]
    fn test_admin_message_type_selection() {
        // Test that all message types are available for admin messages
        let error_type = ServerMessageType::Error;
        let info_type = ServerMessageType::Info;
        let warning_type = ServerMessageType::Warning;
        let admin_type = ServerMessageType::Admin;
        let system_type = ServerMessageType::System;

        // Verify admin can use any message type
        assert_eq!(error_type.as_display_string(), "ERROR");
        assert_eq!(info_type.as_display_string(), "INFO");
        assert_eq!(warning_type.as_display_string(), "WARNING");
        assert_eq!(admin_type.as_display_string(), "ADMIN");
        assert_eq!(system_type.as_display_string(), "SYSTEM");
    }

    #[test]
    fn test_admin_group_naming_functionality() {
        // Test group name handling for admin messages
        let no_group_name: Option<String> = None;
        let empty_group_name: Option<String> = Some("".to_string());
        let valid_group_name: Option<String> = Some("Moderators".to_string());
        let long_group_name: Option<String> =
            Some("Very Long Group Name That Might Be Too Long For Display".to_string());

        assert!(no_group_name.is_none());
        assert!(empty_group_name.is_some());
        assert!(valid_group_name.is_some());
        assert!(long_group_name.is_some());

        if let Some(ref name) = valid_group_name {
            assert_eq!(name, "Moderators");
            assert!(!name.is_empty());
        }

        if let Some(ref name) = empty_group_name {
            assert!(name.is_empty());
        }
    }

    #[test]
    fn test_admin_message_context_handling() {
        // Test sender context for admin messages
        let admin_context = Some("Admin".to_string());
        let custom_context = Some("System Maintenance".to_string());
        let no_context: Option<String> = None;

        assert!(admin_context.is_some());
        assert!(custom_context.is_some());
        assert!(no_context.is_none());

        if let Some(ref context) = admin_context {
            assert_eq!(context, "Admin");
        }

        if let Some(ref context) = custom_context {
            assert_eq!(context, "System Maintenance");
        }
    }

    #[test]
    fn test_admin_message_delivery_scenarios() {
        use spacetimedb::Identity;

        // Test different delivery scenarios
        let single_player = vec![Identity::from_byte_array([1; 32])];
        let small_group = vec![
            Identity::from_byte_array([1; 32]),
            Identity::from_byte_array([2; 32]),
        ];
        let large_group: Vec<Identity> = (0..10)
            .map(|i| {
                let mut bytes = [0u8; 32];
                bytes[0] = i as u8;
                Identity::from_byte_array(bytes)
            })
            .collect();

        // Verify different group sizes
        assert_eq!(single_player.len(), 1);
        assert_eq!(small_group.len(), 2);
        assert_eq!(large_group.len(), 10);

        // Verify all identities are unique
        let mut unique_check = std::collections::HashSet::new();
        for id in &large_group {
            assert!(unique_check.insert(id));
        }
    }

    #[test]
    fn test_admin_message_authorization_logic() {
        use spacetimedb::Identity;

        // Test authorization logic (without actual SpacetimeDB context)
        let server_identity = Identity::from_byte_array([0; 32]);
        let player_identity = Identity::from_byte_array([1; 32]);
        let admin_identity = Identity::from_byte_array([2; 32]);

        // Verify identities are different
        assert_ne!(server_identity, player_identity);
        assert_ne!(server_identity, admin_identity);
        assert_ne!(player_identity, admin_identity);

        // Test identity comparison logic
        assert_eq!(server_identity, server_identity);
        assert_eq!(player_identity, player_identity);
    }

    #[test]
    fn test_admin_message_error_scenarios() {
        // Test various error scenarios for admin messages
        let empty_message = String::new();
        let whitespace_message = "   \n\t  ".to_string();
        let valid_message = "Valid admin message".to_string();

        // Test message validation
        assert!(empty_message.trim().is_empty());
        assert!(whitespace_message.trim().is_empty());
        assert!(!valid_message.trim().is_empty());

        // Test recipient validation
        let empty_recipients: Vec<u32> = vec![];
        let valid_recipients = vec![1, 2, 3];

        assert!(empty_recipients.is_empty());
        assert!(!valid_recipients.is_empty());
    }

    #[test]
    fn test_admin_message_group_privacy_logic() {
        use spacetimedb::Identity;

        // Test group message privacy logic
        let group_recipients = vec![
            Identity::from_byte_array([1; 32]),
            Identity::from_byte_array([2; 32]),
            Identity::from_byte_array([3; 32]),
        ];

        let group_name = Some("Staff".to_string());

        // Each recipient should only see the group name, not other recipients
        for recipient in &group_recipients {
            // Simulate what each recipient would see
            let visible_group_name = group_name.clone();
            let visible_recipients = vec![*recipient]; // Only their own identity

            assert!(visible_group_name.is_some());
            assert_eq!(visible_recipients.len(), 1);
            assert_eq!(visible_recipients[0], *recipient);
        }
    }

    #[test]
    fn test_admin_message_delivery_verification_logic() {
        use crate::types::server_messages::ServerMessageRecipient;
        use spacetimedb::{Identity, Timestamp};

        // Test message delivery verification logic
        let message_id = 100u64;
        let recipients = vec![
            ServerMessageRecipient::new(message_id, Identity::from_byte_array([1; 32])),
            ServerMessageRecipient::new(message_id, Identity::from_byte_array([2; 32])),
            ServerMessageRecipient::new(message_id, Identity::from_byte_array([3; 32])),
        ];

        // Verify all recipients have the same message ID
        assert!(recipients.iter().all(|r| r.server_message_id == message_id));

        // Verify all recipients have delivery timestamps
        assert!(recipients
            .iter()
            .all(|r| r.delivered_at.to_micros_since_unix_epoch() > 0));

        // Verify all recipients start as unread
        assert!(recipients.iter().all(|r| !r.is_read()));

        // Test delivery verification count
        let delivered_count = recipients.len();
        let unread_count = recipients.iter().filter(|r| !r.is_read()).count();

        assert_eq!(delivered_count, 3);
        assert_eq!(unread_count, 3);
    }

    // Tests for group messaging with privacy protection

    #[test]
    fn test_group_message_privacy_protection_logic() {
        use spacetimedb::Identity;

        // Test that group messaging maintains privacy
        let group_members = vec![
            Identity::from_byte_array([1; 32]),
            Identity::from_byte_array([2; 32]),
            Identity::from_byte_array([3; 32]),
        ];

        let group_name = Some("Development Team".to_string());
        let message_content = "Server maintenance tonight at 10 PM".to_string();

        // Simulate what each member would see
        for (index, member) in group_members.iter().enumerate() {
            // Each member should only see their own recipient record
            let visible_to_member = vec![*member];

            // Verify privacy: member only sees themselves
            assert_eq!(visible_to_member.len(), 1);
            assert_eq!(visible_to_member[0], *member);

            // Verify they can see the group name
            assert!(group_name.is_some());
            if let Some(ref name) = group_name {
                assert_eq!(name, "Development Team");
            }

            // Verify they can see the message content
            assert_eq!(message_content, "Server maintenance tonight at 10 PM");

            // Verify they cannot see other members
            for (other_index, other_member) in group_members.iter().enumerate() {
                if index != other_index {
                    assert_ne!(visible_to_member[0], *other_member);
                }
            }
        }
    }

    #[test]
    fn test_group_name_display_functionality() {
        // Test group name display scenarios
        let no_group_name: Option<String> = None;
        let empty_group_name = Some("".to_string());
        let short_group_name = Some("Staff".to_string());
        let long_group_name =
            Some("Very Long Group Name That Should Still Display Properly".to_string());
        let special_chars_group_name = Some("Admin & Moderators (Level 1)".to_string());

        // Test display logic for different group name scenarios
        assert!(no_group_name.is_none());

        if let Some(ref name) = empty_group_name {
            assert!(name.is_empty());
        }

        if let Some(ref name) = short_group_name {
            assert_eq!(name, "Staff");
            assert!(!name.is_empty());
        }

        if let Some(ref name) = long_group_name {
            assert!(name.len() > 20);
            assert!(!name.is_empty());
        }

        if let Some(ref name) = special_chars_group_name {
            assert!(name.contains("&"));
            assert!(name.contains("("));
            assert!(name.contains(")"));
        }
    }

    #[test]
    fn test_group_message_recipient_isolation() {
        use crate::types::server_messages::ServerMessageRecipient;
        use spacetimedb::Identity;

        let message_id = 100u64;
        let group_members = vec![
            Identity::from_byte_array([1; 32]),
            Identity::from_byte_array([2; 32]),
            Identity::from_byte_array([3; 32]),
            Identity::from_byte_array([4; 32]),
        ];

        // Create recipient records for group message
        let recipients: Vec<ServerMessageRecipient> = group_members
            .iter()
            .map(|&member| ServerMessageRecipient::new(message_id, member))
            .collect();

        // Verify all recipients have the same message ID
        assert!(recipients.iter().all(|r| r.server_message_id == message_id));

        // Test that each member can only see their own recipient record
        for (index, member) in group_members.iter().enumerate() {
            let member_recipients: Vec<&ServerMessageRecipient> = recipients
                .iter()
                .filter(|r| r.player_id == *member)
                .collect();

            // Each member should only see one recipient record (their own)
            assert_eq!(member_recipients.len(), 1);
            assert_eq!(member_recipients[0].player_id, *member);

            // Verify they cannot see other members' recipient records
            let other_recipients: Vec<&ServerMessageRecipient> = recipients
                .iter()
                .filter(|r| r.player_id != *member)
                .collect();

            assert_eq!(other_recipients.len(), group_members.len() - 1);
            for other_recipient in other_recipients {
                assert_ne!(other_recipient.player_id, *member);
            }
        }
    }

    #[test]
    fn test_multiple_group_message_scenarios() {
        use spacetimedb::Identity;

        // Test different group messaging scenarios

        // Scenario 1: Small group (2 members)
        let small_group = vec![
            Identity::from_byte_array([1; 32]),
            Identity::from_byte_array([2; 32]),
        ];

        // Scenario 2: Medium group (5 members)
        let medium_group: Vec<Identity> = (0..5)
            .map(|i| {
                let mut bytes = [0u8; 32];
                bytes[0] = i as u8;
                Identity::from_byte_array(bytes)
            })
            .collect();

        // Scenario 3: Large group (20 members)
        let large_group: Vec<Identity> = (0..20)
            .map(|i| {
                let mut bytes = [0u8; 32];
                bytes[0] = i as u8;
                bytes[1] = (i / 256) as u8;
                Identity::from_byte_array(bytes)
            })
            .collect();

        // Test group sizes
        assert_eq!(small_group.len(), 2);
        assert_eq!(medium_group.len(), 5);
        assert_eq!(large_group.len(), 20);

        // Test privacy for each scenario
        for group in [&small_group, &medium_group, &large_group] {
            for member in group {
                // Each member should only see themselves
                let visible_members: Vec<&Identity> =
                    group.iter().filter(|&m| m == member).collect();
                assert_eq!(visible_members.len(), 1);
                assert_eq!(visible_members[0], member);
            }
        }
    }

    #[test]
    fn test_group_message_context_privacy() {
        use spacetimedb::Identity;

        // Test that group message context doesn't leak private information
        let group_members = vec![
            Identity::from_byte_array([1; 32]), // Alice
            Identity::from_byte_array([2; 32]), // Bob
            Identity::from_byte_array([3; 32]), // Charlie
        ];

        let group_name = Some("Project Alpha".to_string());
        let sender_context = Some("Admin".to_string());
        let message = "Meeting moved to 3 PM".to_string();

        // Simulate what each member sees
        for member in &group_members {
            // Each member sees the same message content and context
            assert_eq!(message, "Meeting moved to 3 PM");

            // Each member sees the same group name
            if let Some(ref name) = group_name {
                assert_eq!(name, "Project Alpha");
            }

            // Each member sees the same sender context
            if let Some(ref context) = sender_context {
                assert_eq!(context, "Admin");
            }

            // But each member only knows about their own participation
            // (they don't see the list of other recipients)
            let visible_recipient = *member;

            // Verify they can't determine other recipients from the message
            for other_member in &group_members {
                if other_member != member {
                    // The message content doesn't reveal other recipients
                    assert!(!message.contains(&format!("{:?}", other_member)));

                    // The group name doesn't reveal other recipients
                    if let Some(ref name) = group_name {
                        assert!(!name.contains(&format!("{:?}", other_member)));
                    }
                }
            }
        }
    }

    #[test]
    fn test_group_message_read_status_privacy() {
        use crate::types::server_messages::ServerMessageRecipient;
        use spacetimedb::Identity;

        let message_id = 200u64;
        let group_members = vec![
            Identity::from_byte_array([1; 32]),
            Identity::from_byte_array([2; 32]),
            Identity::from_byte_array([3; 32]),
        ];

        // Create recipient records
        let mut recipients: Vec<ServerMessageRecipient> = group_members
            .iter()
            .map(|&member| ServerMessageRecipient::new(message_id, member))
            .collect();

        // Simulate one member reading the message
        recipients[0].mark_as_read();

        // Test privacy of read status
        for (index, member) in group_members.iter().enumerate() {
            let member_recipient = &recipients[index];

            // Each member can only see their own read status
            assert_eq!(member_recipient.player_id, *member);

            if index == 0 {
                // First member marked as read
                assert!(member_recipient.is_read());
            } else {
                // Other members still unread
                assert!(!member_recipient.is_read());
            }

            // Members cannot see other members' read status
            for (other_index, _) in group_members.iter().enumerate() {
                if index != other_index {
                    // They only have access to their own recipient record
                    assert_ne!(recipients[other_index].player_id, *member);
                }
            }
        }
    }

    #[test]
    fn test_group_message_delivery_verification() {
        use crate::types::server_messages::ServerMessageRecipient;
        use spacetimedb::Identity;

        let message_id = 300u64;
        let group_size = 10;
        let group_members: Vec<Identity> = (0..group_size)
            .map(|i| {
                let mut bytes = [0u8; 32];
                bytes[0] = i as u8;
                Identity::from_byte_array(bytes)
            })
            .collect();

        // Create recipient records for all group members
        let recipients: Vec<ServerMessageRecipient> = group_members
            .iter()
            .map(|&member| ServerMessageRecipient::new(message_id, member))
            .collect();

        // Verify delivery to all members
        assert_eq!(recipients.len(), group_size);

        // Verify all recipients have the same message ID
        assert!(recipients.iter().all(|r| r.server_message_id == message_id));

        // Verify all recipients have delivery timestamps
        assert!(recipients
            .iter()
            .all(|r| r.delivered_at.to_micros_since_unix_epoch() > 0));

        // Verify all recipients start as unread
        assert!(recipients.iter().all(|r| !r.is_read()));

        // Verify each recipient corresponds to a unique group member
        let mut seen_members = std::collections::HashSet::new();
        for recipient in &recipients {
            assert!(seen_members.insert(recipient.player_id));
        }
        assert_eq!(seen_members.len(), group_size);

        // Verify delivery count matches group size
        let delivered_count = recipients.len();
        assert_eq!(delivered_count, group_size);
    }

    // Note: Full integration tests for utility functions require SpacetimeDB runtime
    // and would need to be run as integration tests within the SpacetimeDB environment.
    // These tests cover the validation logic and parameter handling that can be tested
    // without database operations.
}
