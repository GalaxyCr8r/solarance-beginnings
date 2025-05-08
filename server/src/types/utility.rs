use spacetimedb::{ Identity, ReducerContext, Table };

use super::{ common::is_server_or_owner, players::{ player, Player }, stellarobjects::{create_stellar_object_internal, create_stellar_object_player_window_for, player_controlled_stellar_object, PlayerControlledStellarObject} };

/// For helper reducers that utilize several different tables
///

#[spacetimedb::reducer]
pub fn create_player_controlled_ship(ctx: &ReducerContext, identity: Identity, username: String) -> Result<(), String> {
    ctx.db.player().insert(Player {
        identity: identity,
        username: username, // TODO: Bust this out into its own reducer that the player needs to set up before calling this reducer.
    });
    
    if let Ok(ship) = create_stellar_object_internal(
        ctx,
        super::stellarobjects::StellarObjectKinds::Ship,
        0, // TODO: Make this the proper sector id!
        super::stellarobjects::StellarObjectTransform { x: 64.0, y: 64.0, rotation_radians: 0.0, sobj_id: 0 },
        0.0
    ) {
        ctx.db.player_controlled_stellar_object().insert(PlayerControlledStellarObject { identity, controlled_sobj_id: ship.id, sector_id: ship.sector_id });

        create_stellar_object_player_window_for(ctx, ship.id);
        Ok(())
    } else {
        Err("Failed to create ship!".to_string())
    }
}
