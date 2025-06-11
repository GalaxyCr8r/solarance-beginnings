
use log::info;
use spacetimedb::{ Identity, ReducerContext };
use spacetimedsl::{ dsl, Wrapper };

use crate::types::{
    chats::*,
    common::*,
    items::{ definitions::*, utility::*, * },
    players::timers::*,
    ships::{timers::*, utility::*},
};

use crate::{ ships::*, stellarobjects::{ *, utility::*, reducers::* } };
use crate::{ players::*, sectors::* };

//////////////////////////////////////////////////////////////
// Reducers ///
//////////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn register_playername(
    ctx: &ReducerContext,
    identity: Identity,
    username: String
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if dsl.get_player_by_username(&username).is_some() {
        return Err("Username already taken!".to_string());
    }

    let _player = dsl.create_player(identity, &username, 1000)?;

    // Select starting faction?

    Ok(())
}

#[spacetimedb::reducer]
pub fn create_player_controlled_ship(
    ctx: &ReducerContext,
    identity: Identity,
    username: String // TODO ReMOVE
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if dsl.get_player_by_identity(&identity).is_none() {
        return Err("Client hasn't created a username yet!".to_string());
    }

    if
        let Ok(sobj) = create_sobj_internal(
            ctx,
            StellarObjectKinds::Ship,
            &SectorId::new(0), // TODO: Make this the proper sector id!
            StellarObjectTransformInternal { x: 64.0, y: 64.0, rotation_radians: 0.0, sobj_id: 0 }
        )
    {
        let _ = create_sobj_player_window_for(ctx, identity, sobj.get_id())?;
        init_timers(ctx, identity, &sobj)?;

        let ship_type = dsl
            .get_ship_type_definition_by_id(ShipTypeDefinitionId::new(1001))
            .ok_or("Failed to get ship type")?;
        let mut ship = create_ship_instance(ctx, ship_type, identity, sobj.clone())?;
        let _shipobj = dsl.create_ship_object(&ship, &sobj, sobj.get_sector_id(), identity)?;

        {
            let item = get_item_definition(ctx, 1000).ok_or("Failed to get item definition")?;
            let _ = attempt_to_load_cargo_into_ship(ctx, &mut ship, &item, 5)?;
        }

        {
            let item = get_item_definition(ctx, 1003).ok_or("Failed to get item definition")?;
            let _ = attempt_to_load_cargo_into_ship(ctx, &mut ship, &item, 1)?;
        }

        dsl.create_ship_equipment_slot(
            &ship,
            EquipmentSlotType::MiningLaser,
            0,
            ItemDefinitionId::new(DEFAULT_MINING_LASER_ID)
        )?;

        info!("Successfully created ship!");
        send_global_chat(ctx, format!("{} has created a ship!", username))?;
        Ok(())
    } else {
        Err("Failed to create ship!".to_string())
    }
}

/// Called by players to update their own ship's controls.
#[spacetimedb::reducer]
pub fn update_player_controller(
    ctx: &ReducerContext,
    controller: PlayerController
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if ctx.sender != controller.identity {
        info!(
            "SECURITY ERROR: ID {} is trying to change player controller for ID {}!!! Username: {}",
            ctx.sender,
            controller.identity,
            get_username(ctx, controller.identity)
        );
        return Err("ID Mismatch. This was reported to the system admin.".to_string());
    }

    // Clean up player's mining timers.
    if let Some(previous_controller) = dsl.get_player_controller_by_identity(&controller.identity) {
        // Check if the player had been trying to mine, if so, remove the mining timers.
        if previous_controller.mining_laser_on && !controller.mining_laser_on {
            info!(
                "Player {} no longer mining, removing mining timers.",
                get_username(ctx, controller.identity)
            );
            for mining_timer in dsl.get_ship_mining_timers_by_ship_sobj_id(
                StellarObjectId::new(previous_controller.stellar_object_id.unwrap())
            ) {
                dsl.delete_ship_mining_timer_by_scheduled_id(&mining_timer);
            }
        }
    }

    ctx.db.player_controller().identity().update(controller.clone());

    Ok(())
}

//////////////////////////////////////////////////////////////
// Utils
//////////////////////////////////////////////////////////////
