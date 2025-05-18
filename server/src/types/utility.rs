use spacetimedb::{ Identity, ReducerContext };
use spacetimedsl::{dsl, Wrapper};

use super::{ players::{ CreatePlayerRow, GetPlayerRowOptionByUsername }, sector::SectorLocationId, stellarobjects::{create_sobj_internal, create_sobj_player_window_for, CreatePlayerControlledStellarObjectRow} };

/// For helper reducers that utilize several different tables
///

#[spacetimedb::reducer]
pub fn create_player_controlled_ship(ctx: &ReducerContext, identity: Identity, username: String) -> Result<(), String> {
    let dsl = dsl(ctx);

    if dsl.get_player_by_username(&username).is_some() {
        return Err("Username already taken!".to_string());
    }

    let player = dsl.create_player(identity, &username)?; // TODO: Bust this out into its own reducer that the player needs to set up before calling this reducer.
    
    if let Ok(ship) = create_sobj_internal(
        ctx,
        super::stellarobjects::StellarObjectKinds::Ship,
        SectorLocationId::new(0), // TODO: Make this the proper sector id!
        super::stellarobjects::StellarObjectTransformInternal { x: 64.0, y: 64.0, rotation_radians: 0.0, sobj_id: 0 }
    ) {
        let controlled = dsl.create_player_controlled_stellar_object(
            player.identity, &ship, ship.get_sector_id())?;
        let _ = create_sobj_player_window_for(ctx, controlled)?;
        Ok(())
    } else {
        Err("Failed to create ship!".to_string())
    }
}
