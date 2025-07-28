# Design Document

## Overview

The server-to-player messaging system extends the existing chat infrastructure to enable the SpacetimeDB server to send targeted messages to individual players or groups. This system provides a reliable mechanism for delivering action feedback, system notifications, and administrative communications while maintaining the existing chat interface patterns.

The design leverages SpacetimeDB's table and reducer architecture, integrating seamlessly with the current chat system (`GlobalChatMessage`, `SectorChatMessage`, etc.) and following established patterns for data persistence, client synchronization, and UI integration.

## Architecture

### Core Components

1. **ServerMessage Table**: Stores server-generated messages with recipient targeting
2. **ServerMessageRecipient Table**: Manages message delivery to individual players and groups
3. **Server Message Utility Functions**: Helper functions for sending messages from reducers
4. **Client Integration**: Extensions to the existing chat widget for server message display
5. **Message Delivery System**: Handles message routing and delivery status tracking

### Data Flow

```
Reducer Action Failure → Server Message Utility → ServerMessage Table → Client Subscription → Chat Widget Display
```

### Integration Points

- **Existing Reducers**: Enhanced with server message utility calls for error feedback
- **Chat System**: Extended to include server message channel
- **Client UI**: Modified chat widget with server message display
- **Database**: New tables following existing SpacetimeDB patterns

## Components and Interfaces

### Database Schema

#### ServerMessage Table

```rust
#[dsl(plural_name = server_messages)]
#[table(name = server_message, public)]
pub struct ServerMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub message: String,
    pub message_type: ServerMessageType, // Error, Info, Warning, Admin
    pub group_name: Option<String>, // For group messages
    pub sender_context: Option<String>, // Context about what action triggered this

    created_at: Timestamp,
}
```

#### ServerMessageRecipient Table

```rust
#[dsl(plural_name = server_message_recipients)]
#[table(name = server_message_recipient, public)]
pub struct ServerMessageRecipient {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(path = crate::types::server_messages::ServerMessageId)]
    pub server_message_id: u64,

    #[index(btree)]
    #[use_wrapper(path = crate::players::PlayerId)]
    pub player_id: Identity,

    pub read_at: Option<Timestamp>,
    pub delivered_at: Timestamp,
}
```

#### ServerMessageType Enum

```rust
#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum ServerMessageType {
    Error,      // Action failures, validation errors
    Info,       // General information
    Warning,    // Important notices
    Admin,      // Administrative messages
    System,     // System-generated notifications
}
```

### Utility Functions

#### Core Messaging Functions

```rust
// Send message to individual player
pub fn send_server_message_to_player(
    ctx: &ReducerContext,
    player_id: &PlayerId,
    message: String,
    message_type: ServerMessageType,
    sender_context: Option<String>,
) -> Result<(), String>

// Send message to multiple players with optional group name
pub fn send_server_message_to_group(
    ctx: &ReducerContext,
    player_ids: Vec<PlayerId>,
    message: String,
    message_type: ServerMessageType,
    group_name: Option<String>,
    sender_context: Option<String>,
) -> Result<(), String>

// Convenience function for error messages from reducers
pub fn send_error_message(
    ctx: &ReducerContext,
    player_id: &PlayerId,
    message: String,
    action_context: Option<&str>,
) -> Result<(), String>

// Convenience function for info messages from reducers
pub fn send_info_message(
    ctx: &ReducerContext,
    player_id: &PlayerId,
    message: String,
    action_context: Option<&str>,
) -> Result<(), String>
```

### Client-Side Integration

#### Chat Widget Extensions

- New `Server` channel in chat tabs, gains an asterisk when there's an unread message.
- Distinct visual styling for server messages
- Message type-based color coding (errors in red, info in blue, etc.)
- Group context display when applicable
- Unread message indicators

#### Message Display Format

```
[ERROR] Trading: Cannot buy 5x Food Rations from station: Not enough credits. You have 500c but it costs 750c.
[INFO] System: Welcome to Solarance: Beginnings!
[ADMIN] Maintenance: Server will restart in 10 minutes.
[WARNING] Security: Suspicious activity detected on your account.
```

## Data Models

### Message Storage Strategy

**ServerMessage**: Contains the core message content and metadata

- Immutable once created
- Indexed by creation timestamp for efficient retrieval
- Contains optional group context and sender information

**ServerMessageRecipient**: Tracks delivery and read status per player

- One record per message per recipient
- Enables efficient querying of unread messages
- Supports delivery confirmation and read receipts

### Indexing Strategy

- `server_message_recipient.player_id` (btree): Fast lookup of messages for specific players
- `server_message_recipient.server_message_id` (btree): Efficient message-to-recipients mapping
- `server_message.created_at` (btree): Chronological message ordering
- `server_message_recipient.read_at` (btree): Unread message queries

## Error Handling

### Message Delivery Failures

- Log delivery failures with detailed error information
- Implement retry mechanism for temporary failures
- Graceful degradation when messaging system is unavailable

### Client-Side Error Handling

- Handle missing or corrupted message data
- Fallback display for unknown message types
- Connection loss recovery for message synchronization

### Reducer Integration Error Handling

- Non-blocking message sending (failures don't interrupt main reducer logic)
- Logging of message sending failures
- Fallback to basic error returns when messaging fails

## Testing Strategy

Due to the current limitations of SpacetimeDB, only implement Unit Tests right now. Leave the other tests here for future reference.

### Unit Tests

- Message creation and delivery functions
- Recipient targeting logic
- Message type validation
- Database constraint enforcement

### Integration Tests

- End-to-end message flow from reducer to client display
- Multiple recipient message delivery
- Client chat widget integration
- Message persistence and retrieval

### Performance Tests

- Message delivery latency under load
- Database query performance with large message volumes
- Client UI responsiveness with many messages

### Security Tests

- Message recipient authorization
- Prevention of message spoofing
- Group membership validation
- Admin message authorization

## Implementation Phases

### Phase 1: Core Infrastructure

- Create database tables and basic utility functions
- Implement message creation and storage
- Basic recipient targeting

### Phase 2: Reducer Integration

- Add error message sending to existing reducers
- Implement convenience functions for common message types
- Update station trading, player creation, and other key reducers

### Phase 3: Client Integration

- Extend chat widget with server message display
- Implement message type styling and formatting
- Add unread message indicators

### Phase 4: Advanced Features

- Group messaging functionality
- Message read status tracking
- Administrative message tools
- Performance optimizations

## Security Considerations

### Message Authorization

- Only server-side code can create server messages
- Admin messages require appropriate permissions
- Player identity validation for all message operations

### Privacy Protection

- Group message recipients are not visible to other recipients
- Message content is only accessible to intended recipients
- Audit logging for administrative messages

### Data Integrity

- Immutable message content once created
- Referential integrity between messages and recipients
- Validation of message types and formatting

## Performance Considerations

### Database Optimization

- Efficient indexing for common query patterns
- Message archival strategy for long-term storage
- Batch operations for group message delivery

### Client Performance

- Lazy loading of message history
- Efficient UI updates for new messages
- Memory management for large message volumes

### Network Efficiency

- Minimal data transfer for message updates
- Efficient synchronization with SpacetimeDB subscriptions
- Compression for large message payloads
