// Row-level security rules for server messages
// Currently commented out as per project patterns

/*
use spacetimedb::Identity;
use super::*;

// Players can only see server messages addressed to them
impl ServerMessageRecipient {
    pub fn can_read(&self, identity: &Identity) -> bool {
        self.player_id == *identity
    }
}

// Server messages themselves are readable by anyone who has a recipient record
impl ServerMessage {
    pub fn can_read(&self, identity: &Identity) -> bool {
        // This would need to check if the identity has a corresponding recipient record
        // Implementation would depend on how RLS queries work in SpacetimeDB
        true // Placeholder
    }
}
*/
