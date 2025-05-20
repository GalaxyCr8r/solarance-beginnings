use spacetimedb::{ Identity, ReducerContext, Timestamp };
use spacetimedsl::{dsl, Wrapper};

use super::{ships::{CreateShipInstanceRow, CreateShipObjectRow, GetShipTypeDefinitionRowOptionById, ShipTypeDefinitionId}, stellarobjects::{GetPlayerControlledStellarObjectRowOptionByIdentity, StellarObjectId}};
use super::{players::{ CreatePlayerRow, GetPlayerRowOptionByUsername }, sector::SectorId, stellarobjects::{create_sobj_internal, create_sobj_player_window_for, CreatePlayerControlledStellarObjectRow} };

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
    
    let owner = dsl.get_player_controlled_stellar_object_by_identity(&ctx.sender).ok_or(IS_SERVER_OR_OWNER_ERROR)?;
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
        SectorId::new(0), // TODO: Make this the proper sector id!
        super::stellarobjects::StellarObjectTransformInternal { x: 64.0, y: 64.0, rotation_radians: 0.0, sobj_id: 0 }
    ) {
        let controlled = dsl.create_player_controlled_stellar_object(
            player.identity, &sobj, sobj.get_sector_id())?;
        let _ = create_sobj_player_window_for(ctx, controlled)?;

        let ship_type = dsl.get_ship_type_definition_by_id(ShipTypeDefinitionId::new(1001)).ok_or("Blah")?;
        let ship = dsl.create_ship_instance(
            Some(identity), None, 
            ship_type.get_id(), 
            SectorId::new(0), 
            ship_type.max_health.into(), ship_type.max_shield.into(), ship_type.max_energy.into(), ship_type.cargo_capacity, 
            None, None, ctx.timestamp)?;
        let _shipobj = dsl.create_ship_object(ship.get_id(), sobj.get_id(), identity)?;

        Ok(())
    } else {
        Err("Failed to create ship!".to_string())
    }
}
