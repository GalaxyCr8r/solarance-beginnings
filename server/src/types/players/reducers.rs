use log::info;
use spacetimedb::{Identity, ReducerContext};
use spacetimedsl::{dsl, Wrapper};

use crate::types::{
    chats::*,
    factions::{definitions::FACTION_FACTIONLESS, FactionId},
    items::{definitions::*, utility::*, *},
    players::{timers::*, utility::get_username},
    server_messages::utility::send_error_message,
    ships::{timers::*, utility::*},
};

use crate::stellarobjects::{reducers::*, utility::*};
use crate::{players::*, sectors::*};

//////////////////////////////////////////////////////////////
// Reducers ///
//////////////////////////////////////////////////////////////

/// Registers a new player with a unique username and creates their player account.
/// Validates username uniqueness and initializes the player with starting credits.
#[spacetimedb::reducer]
pub fn register_playername(
    ctx: &ReducerContext,
    identity: Identity,
    username: String,
    faction_id: u32,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if username.len() > 32 {
        return Err("Username is toooooo long.".to_string());
    }

    if dsl.get_player_by_username(&username).is_ok() {
        let player_id = PlayerId::new(identity);
        let error_message = format!(
            "Username '{}' is already taken. Please choose a different username.",
            username
        );

        // Send server message for error feedback
        send_error_message(
            ctx,
            &player_id,
            error_message.clone(),
            Some("Player Registration"),
        )?;

        return Err("Username already taken!".to_string());
    }

    // TODO: Re-enable faction validation once client bindings are updated
    // For now, just use the provided faction_id or default to factionless
    let final_faction = FactionId::new(if faction_id == 0 { 
        FACTION_FACTIONLESS 
    } else { 
        faction_id 
    });

    let player = dsl.create_player(identity, &username, 1000, true, final_faction.clone())?;
    let _ = dsl.create_faction_chat_message(&player, final_faction, "- has joined the faction!");

    Ok(())
}

/// Creates a new ship for a registered player with starting equipment and cargo.
/// Sets up the ship's stellar object, player window, controller, and initial inventory.
#[spacetimedb::reducer]
pub fn create_player_controlled_ship(
    ctx: &ReducerContext,
    identity: Identity,
    username: String, // TODO ReMOVE
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let player_id = PlayerId::new(identity);
    let player = match dsl.get_player_by_id(&player_id) {
        Ok(p) => p,
        Err(_) => {
            let error_message = 
                "You must register a username before creating a ship. Please use the registration system first.".to_string();
    
            // Send server message for error feedback
            send_error_message(ctx, &player_id, error_message.clone(), Some("Ship Creation"))?;
    
            return Err("Client hasn't created a username yet!".to_string());
        },
    };

    if let Ok(sobj) = create_sobj_internal(
        ctx,
        StellarObjectKinds::Ship,
        &SectorId::new(0), // TODO: Make this the proper sector id!
        StellarObjectTransformInternal::default().from_xy(64.0, 64.0),
    ) {
        let _ = create_sobj_player_window_for(ctx, identity, sobj.get_id())?;
        initialize_player_controller(ctx, &player_id, &sobj)?;

        let ship_type = dsl.get_ship_type_definition_by_id(ShipTypeDefinitionId::new(1001))?;
        let faction_id = player.get_faction_id().clone();
        let (ship, mut status) = create_ship_from_sobj(ctx, &ship_type, &player_id, &faction_id, &sobj)?;

        let _ = attempt_to_load_cargo_into_ship(
            ctx,
            &mut status,
            &ship.get_id(),
            &get_item_definition(ctx, ITEM_FOOD_RATIONS)?,
            3,
            false,
        )?;
        let _ = attempt_to_load_cargo_into_ship(
            ctx,
            &mut status,
            &ship.get_id(),
            &get_item_definition(ctx, ITEM_ENERGY_CELL)?,
            5,
            false,
        )?;

        dsl.create_ship_equipment_slot(
            &ship.get_id(),
            EquipmentSlotType::MiningLaser,
            0,
            ItemDefinitionId::new(SMOD_BASIC_MINING_LASER),
        )?;

        info!("Successfully created ship!");
        send_global_chat(ctx, format!("{} has created a ship!", username))?;
        Ok(())
    } else {
        let error_message = "Failed to create ship due to a system error. Please try again or contact support if the problem persists.".to_string();

        // Send server message for error feedback
        send_error_message(ctx, &player_id, error_message.clone(), Some("Ship Creation"))?;

        Err("Failed to create ship!".to_string())
    }
}

/// Called by players to update their own ship's controls.
#[spacetimedb::reducer]
pub fn update_player_controller(
    ctx: &ReducerContext,
    controller: PlayerShipController,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if ctx.sender != controller.id {
        info!(
            "SECURITY ERROR: ID {} is trying to change player controller for ID {}!!! Username: {}",
            ctx.sender,
            controller.id,
            get_username(ctx, controller.id)
        );
        return Err("ID Mismatch. This was reported to the system admin.".to_string());
    }

    // Clean up player's mining timers.
    let previous_controller = dsl.get_player_ship_controller_by_id(&controller.get_id())?;

    // Check if the player had been trying to mine, if so, remove the mining timers.
    if previous_controller.mining_laser_on && !controller.mining_laser_on {
        info!(
            "Player {} no longer mining, removing mining timers.",
            get_username(ctx, controller.id)
        );
        for mining_timer in
            dsl.get_ship_mining_timers_by_ship_sobj_id(previous_controller.get_stellar_object_id())
        {
            dsl.delete_ship_mining_timer_by_id(&mining_timer)?;
        }
    }

    dsl.update_player_ship_controller_by_id(controller)?;

    Ok(())
}

//////////////////////////////////////////////////////////////
// Utils
//////////////////////////////////////////////////////////////
