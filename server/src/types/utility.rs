use spacetimedb::{ Identity, ReducerContext, Table };

use super::{ common::is_server_or_owner, players::{ player, Player }, stellarobjects::create_stellar_object_internal };

/// For helper functions that utilize several different tables
///

#[spacetimedb::reducer]
pub fn create_player_controlled_ship(ctx: &ReducerContext, identity: Identity) -> Result<(), String> {
    let mut player = Player {
        identity: identity,
        username: "GalaxyCr8r".to_string(),
        controlled_entity_id: None,
        current_sector: 0,
    };
    let ship = create_stellar_object_internal(
        ctx,
        super::stellarobjects::StellarObjectKinds::Ship,
        0, // TODO: Make this the proper sector id!
        super::stellarobjects::StellarObjectTransform { x: 64.0, y: 64.0, rotation_radians: 0.0, sobj_id: 0 },
        0.0
    );
    if ship.is_ok() {
        player.controlled_entity_id = Some(ship.unwrap().id);
        ctx.db.player().insert(player);
        Ok(())
    } else {
        Err("Failed to create ship!".to_string())
    }
}
