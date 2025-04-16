use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table};


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

#[spacetimedb::reducer(init)]
pub fn init(_ctx: &ReducerContext) {
    // Called when the module is initially published
}

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(_ctx: &ReducerContext) {
    // Called everytime a new client connects
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(_ctx: &ReducerContext) {
    // Called everytime a client disconnects
}

#[spacetimedb::reducer]
pub fn server_only(ctx: &ReducerContext){
    if ctx.sender != ctx.identity() {
        panic!("This reducer can only be called by SpacetimeDB!");
    }
    log::info!("I'm a server!");
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
