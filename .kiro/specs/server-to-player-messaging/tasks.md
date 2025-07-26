# Implementation Plan

- [x] 1. Create server message data structures and database tables

  - Implement ServerMessage and ServerMessageRecipient table definitions with proper SpacetimeDB attributes
  - Create ServerMessageType enum with appropriate derive macros
  - Add DSL wrapper configurations and primary key relationships
  - Write unit tests for data structure creation and validation
  - _Requirements: 1.3, 1.4, 4.3, 6.2_

- [x] 2. Implement core server messaging utility functions

  - Create send_server_message_to_player function with proper error handling
  - Implement send_server_message_to_group function for multiple recipients
  - Add send_error_message convenience function for reducer error feedback
  - Write comprehensive unit tests for all utility functions
  - _Requirements: 2.1, 2.2, 4.1, 4.2_

- [x] 3. Create server message module structure and initialization

  - Set up server/src/types/server_messages.rs module with standard submodules
  - Implement module initialization function following existing patterns
  - Create reducers.rs, utility.rs, and impls.rs files with basic structure
  - Add module registration to main types module
  - _Requirements: 4.3, 4.4_

- [ ] 4. Integrate server messaging into existing reducers for error feedback
- [ ] 4.1 Update player registration reducer with server message error feedback

  - Modify register_playername reducer to send server messages on username conflicts
  - Replace existing error returns with server message calls plus error returns
  - Test username conflict scenarios with message delivery verification
  - _Requirements: 1.1, 1.2, 4.1_

- [ ] 4.2 Update station trading reducers with server message feedback

  - Modify buy_item_from_station_module to send detailed error messages for failures
  - Update sell_item_to_station_module with specific failure reason messages
  - Include transaction context and specific failure reasons in messages
  - Test all trading failure scenarios with proper message delivery
  - _Requirements: 1.1, 1.2, 4.1_

- [ ] 4.3 Add server message integration to ship creation reducer

  - Update create_player_controlled_ship with server message error feedback
  - Send informative messages for ship creation failures and validation errors
  - Test ship creation failure scenarios with message verification
  - _Requirements: 1.2, 4.1_

- [ ] 5. Implement message delivery and tracking system

  - Create reducer functions for marking messages as read
  - Implement message history retrieval with pagination support
  - Add unread message count functionality for players
  - Write tests for message delivery status tracking
  - _Requirements: 1.4, 6.1, 6.3, 6.4_

- [ ] 6. Extend client chat widget for server message display
- [ ] 6.1 Add ServerMessage data structures to client module bindings

  - Update client-side data structures to match server message types
  - Implement proper serialization/deserialization for message data
  - Add client-side message type handling and validation
  - _Requirements: 1.3, 5.1_

- [ ] 6.2 Create server message channel in chat widget

  - Add ServerMessage tab to existing chat interface
  - Implement server message display with proper formatting
  - Add message type-based color coding and styling
  - Create separate message rendering function for server messages
  - _Requirements: 1.3, 5.1, 5.2, 5.3_

- [ ] 6.3 Implement server message visual styling and formatting

  - Create distinct visual styles for different message types (Error, Info, Warning, Admin)
  - Add group name display for group messages without showing other recipients
  - Implement message timestamp and context display
  - Add unread message indicators and highlighting
  - _Requirements: 2.3, 3.1, 3.3, 5.1, 5.2, 5.3, 5.4_

- [ ] 7. Add administrative server message functionality
- [ ] 7.1 Create admin reducer for sending targeted messages

  - Implement send_admin_message reducer with proper authorization checks
  - Add support for individual and group message targeting
  - Include message type selection and group naming functionality
  - Write authorization tests and message delivery verification
  - _Requirements: 2.1, 2.2, 2.4, 2.5_

- [ ] 7.2 Implement group messaging with privacy protection

  - Create group message delivery system that hides recipient lists
  - Add group name display functionality for recipients
  - Implement proper recipient privacy controls
  - Test group messaging scenarios with multiple players
  - _Requirements: 2.2, 2.3, 3.1, 3.2, 3.3_

- [ ] 8. Add message persistence and retrieval optimization

  - Implement database indexing for efficient message queries
  - Create message archival system for performance optimization
  - Add message cleanup functionality for old messages
  - Write performance tests for message retrieval under load
  - _Requirements: 1.4, 6.4_

- [ ] 9. Implement comprehensive error handling and logging

  - Add proper error handling for all message delivery failures
  - Implement logging for message delivery status and failures
  - Create fallback mechanisms for messaging system failures
  - Add client-side error handling for message display issues
  - _Requirements: 4.2, 6.2_

- [ ] 10. Create integration tests for end-to-end message flow
  - Write tests for complete message flow from reducer error to client display
  - Test multiple recipient message delivery scenarios
  - Verify message persistence and retrieval functionality
  - Create performance tests for high-volume message scenarios
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 6.1_
