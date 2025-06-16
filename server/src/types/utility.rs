
use spacetimedb::ReducerContext;
use spacetimedsl::{dsl, Wrapper};


use super::{ships::*, stellarobjects::*};

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
pub fn is_server_or_sobj_owner(ctx: &ReducerContext, sobj_id: StellarObjectId) -> Result<(), String> {
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
pub fn is_server_or_ship_owner(ctx: &ReducerContext, ship_id: ShipInstanceId) -> Result<(), String> {
    let dsl = dsl(ctx);

    if ctx.sender == ctx.identity() {
        return Ok(());
    }
    
    let owner = dsl.get_ship_instance_by_id(ship_id).ok_or(IS_SERVER_OR_OWNER_ERROR)?;
    if owner.owner_id.ok_or("Ship Instance doesn't exist!")? == ctx.sender {
        return Ok(());
    }
    Err(IS_SERVER_OR_OWNER_ERROR.to_string())
}
