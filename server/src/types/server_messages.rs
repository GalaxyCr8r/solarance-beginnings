use spacetimedb::{table, Identity, ReducerContext, SpacetimeType, Timestamp};
use spacetimedsl::*;

pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
pub mod rls; // Row-level-security rules for this file's structs.
pub mod tests; // Unit tests for this module
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum ServerMessageType {
    Error,   // Action failures, validation errors
    Info,    // General information
    Warning, // Important notices
    Admin,   // Administrative messages
    System,  // System-generated notifications
}

#[dsl(plural_name = server_messages)]
#[table(name = server_message, public)]
pub struct ServerMessage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub message: String,
    pub message_type: ServerMessageType,
    pub group_name: Option<String>,     // For group messages
    pub sender_context: Option<String>, // Context about what action triggered this

    created_at: Timestamp,
}

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
    #[use_wrapper(path = crate::types::players::PlayerId)]
    pub player_id: Identity,

    pub read_at: Option<Timestamp>,
    pub delivered_at: Timestamp,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {
    Ok(())
}
