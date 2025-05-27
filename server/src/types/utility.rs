use log::info;
use spacetimedb::{ Identity, ReducerContext };
use spacetimedsl::{dsl, Wrapper};

use crate::types::{chats::send_global_chat, items::utility::get_item_definition, ships::utility::*};

use super::{ships::*, stellarobjects::{*, utility::*, reducers::*}};
use super::{players::*, sectors::* };

/// For helper reducers that utilize several different tables
///

#[spacetimedb::reducer]
pub fn try_server_only(ctx: &ReducerContext) -> Result<(), String> {
    if ctx.sender == ctx.identity() {
        //log::info!("I'm a server!");
        return Ok(());
    }
    if ctx.sender.to_string().contains("eyJhbGciOiJSUzI1NiJ9.eyJzdWIiO") {
        //log::info!("I'm Karl's desktop!");
        return Ok(());
    }
    
    Err("This reducer can only be called by SpacetimeDB!".to_string())
}

#[spacetimedb::reducer]
pub fn server_only(ctx: &ReducerContext){
    if try_server_only(ctx).is_err() {
        panic!("This reducer can only be called by SpacetimeDB!");
    }
}

const IS_SERVER_OR_OWNER_ERROR: &str = "This reducer can only be called by SpacetimeDB or the owner!";

#[spacetimedb::reducer]
pub fn is_server_or_owner(ctx: &ReducerContext, sobj_id: StellarObjectId) -> Result<(), String> {
    let dsl = dsl(ctx);

    if ctx.sender == ctx.identity() {
        return Ok(());
    }
    
    let owner = dsl.get_ship_objects_by_player_id(&ctx.sender).last().ok_or(IS_SERVER_OR_OWNER_ERROR)?;
    if StellarObjectId::new(owner.sobj_id) == sobj_id {
        return Ok(());
    }
    Err(IS_SERVER_OR_OWNER_ERROR.to_string())
}

#[spacetimedb::reducer]
pub fn create_player_controlled_ship(ctx: &ReducerContext, identity: Identity, username: String) -> Result<(), String> {
    let dsl = dsl(ctx);

    if dsl.get_player_by_username(&username).is_some() {
        return Err("Username already taken!".to_string());
    }

    let player = dsl.create_player(identity, &username, 0)?; // TODO: Bust this out into its own reducer that the player needs to set up before calling this reducer.
    
    if let Ok(sobj) = create_sobj_internal(
        ctx,
        super::stellarobjects::StellarObjectKinds::Ship,
        &SectorId::new(0), // TODO: Make this the proper sector id!
        super::stellarobjects::StellarObjectTransformInternal { x: 64.0, y: 64.0, rotation_radians: 0.0, sobj_id: 0 }
    ) {
        let _ = create_sobj_player_window_for(ctx, player.identity, sobj.get_id())?;

        let ship_type = dsl.get_ship_type_definition_by_id(ShipTypeDefinitionId::new(1001)).ok_or("Failed to get ship type")?;
        let mut ship = create_ship_instance(
            ctx,
            ship_type,
            player.identity,
            sobj.clone()
        )?;
        let _shipobj = dsl.create_ship_object(&ship, &sobj, sobj.get_sector_id(), identity)?;

        {
            let item = get_item_definition(ctx, 1000).ok_or("Failed to get item definition")?;
            let _ = load_cargo_into_ship(ctx, &mut ship, &item, 5)?;
        }

        {
            let item = get_item_definition(ctx, 1003).ok_or("Failed to get item definition")?;
            let _ = load_cargo_into_ship(ctx, &mut ship, &item, 1)?;
        }

        info!("Successfully created ship!");
        send_global_chat(ctx, format!("{} has created a ship!", username))?;
        Ok(())
    } else {
        Err("Failed to create ship!".to_string())
    }
}
