use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table};

use super::{players::player, stellarobjects::player_controlled_stellar_object};



#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum MapView {
    LocalSpace,
    LocalSystem,
    SolarSystem,
    GalacticSystem
}

#[spacetimedb::table(name = person, public)]
pub struct Person {
    #[primary_key]
    identity: Identity,
    name: String,
    last_view: MapView
}

//// TODO: Deprecate the below

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

#[spacetimedb::reducer]
pub fn add_person(ctx: &ReducerContext, name: String) {
    ctx.db.person().insert(Person { name, identity: ctx.sender, last_view: MapView::GalacticSystem });
}

#[spacetimedb::reducer]
pub fn say_hello(ctx: &ReducerContext) {
    for person in ctx.db.person().iter() {
        log::info!("Hello, {}!", person.name);
    }
    log::info!("Hello, World!");
}

#[spacetimedb::reducer]
pub fn set_map_view(ctx: &ReducerContext, new_view: MapView) -> Result<(), String> {
    
    if let Some(user) = ctx.db.person().identity().find(ctx.sender) {
        ctx.db.person().identity().update(Person { last_view: new_view, ..user });
        log::info!("New view set!");
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}
