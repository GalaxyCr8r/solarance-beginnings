use spacetimedb::ReducerContext;
use spacetimedsl::dsl;

use super::stellarobjects::player_controlled_stellar_object;


#[dsl(plural_name = global_configurations)]
#[spacetimedb::table(name = global_config)]
pub struct GlobalConfig {
    #[primary_key]
    #[wrap]
    pub id: u32,
    
    pub active_players: u32,
    pub old_gods_defeated: u8,
}

#[spacetimedb::reducer]
pub fn try_server_only(ctx: &ReducerContext) -> Result<(), String> {
    if ctx.sender != ctx.identity() {
        log::info!("I'm a server!");
        return Ok(());
    }
    if ctx.sender.to_string().contains("eyJhbGciOiJSUzI1NiJ9.eyJzdWIiO") {
        log::info!("I'm Karl's desktop!");
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
pub fn is_server_or_owner(ctx: &ReducerContext, sobj_id: u64) -> Result<(), String> {
    if ctx.sender == ctx.identity() {
        return Ok(());
    }
    let owner = ctx.db.player_controlled_stellar_object().identity().find(ctx.sender).ok_or(IS_SERVER_OR_OWNER_ERROR)?;
    if owner.controlled_sobj_id == sobj_id {
        return Ok(());
    }
    Err(IS_SERVER_OR_OWNER_ERROR.to_string())
}
