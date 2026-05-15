use spacetimedb::{table, Identity, Timestamp};
use spacetimedsl::*;

use crate::tables::{factions::FactionId, ships::*};

use super::stellarobjects::*;

#[dsl(plural_name = players, method(update = true))]
#[table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    #[create_wrapper]
    #[referenced_by(path = crate::tables::ships, table = ship_movement_controller)]
    #[referenced_by(path = crate::tables::ships, table = ship)]
    #[referenced_by(path = crate::tables::chats, table = global_chat_message)]
    #[referenced_by(path = crate::tables::chats, table = sector_chat_message)]
    #[referenced_by(path = crate::tables::chats, table = faction_chat_message)]
    #[referenced_by(path = crate::tables::server_messages, table = server_message_recipient)]
    #[referenced_by(path = crate::tables::stations, table = construction_contribution_log)]
    id: Identity,

    #[unique]
    pub username: String,
    pub credits: u64,

    pub logged_in: bool,
    pub faction_id: FactionId,

    created_at: Timestamp,
    modified_at: Timestamp,
}

impl Player {
    /// Sobj id of the player's currently-controlled (in-sector) ship, if any.
    pub fn get_ship_id<T: spacetimedsl::WriteContext>(&self, dsl: &DSL<T>) -> Option<u64> {
        dsl.get_ships_by_player_id(&self.get_id())
            .find(|s| *s.get_location() == crate::tables::ships::ShipLocation::Sector)
            .map(|s| s.get_sobj_id().value())
    }

    pub fn get_player_objects<T: spacetimedsl::WriteContext>(
        &self,
        dsl: &DSL<T>,
    ) -> Result<(Ship, StellarObject), String> {
        get_player_ship_and_sobj(dsl, &self.get_id())
    }
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init<T: spacetimedsl::WriteContext>(_dsl: &DSL<T>) -> Result<(), String> {
    Ok(())
}

pub fn get_username<T: spacetimedsl::WriteContext>(dsl: &DSL<T>, id: Identity) -> String {
    if let Some(player) = dsl.get_player_by_id(&PlayerId::new(id)).ok() {
        player.username
    } else {
        // TODO: Check if sender is server
        id.to_abbreviated_hex().to_string()
    }
}
pub fn get_player_ship_and_sobj<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player_id: &PlayerId,
) -> Result<(Ship, StellarObject), String> {
    // Pick the player's first in-sector ship. Phase 4 retired the controller-
    // based lookup; the controller is now purely an input-state mirror.
    let ship_object = dsl
        .get_ships_by_player_id(player_id)
        .find(|ship| *ship.get_location() == crate::tables::ships::ShipLocation::Sector)
        .ok_or_else(|| {
            format!(
                "Player {} has no in-sector ship",
                player_id.value().to_abbreviated_hex()
            )
        })?;
    let ship_sobj = dsl.get_stellar_object_by_id(&ship_object.get_sobj_id())?;
    Ok((ship_object, ship_sobj))
}
