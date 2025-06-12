
use spacetimedb::{table, Table, Identity, ReducerContext};
use crate::types::utility::{try_server_only};
use spacetimedsl::{dsl};

#[dsl(plural_name = privileged_clients)]
#[table(name = privileged_client)]
pub struct PrivilegedClient {
    #[primary_key]
    #[wrap]
    pub identity: Identity,
    pub description: String, // Store some info eg. Karls Client
    pub privilage: u32,
}

#[spacetimedb::reducer]
pub fn grant_privilage_to_client(ctx: &ReducerContext, identity: Identity, description: String, privilage: u32) -> Result<(), String> {
    if try_server_only(ctx).is_err() {
        panic!("This reducer can only be called by SpacetimeDB!");
    }

    let dsl = dsl(ctx);
    dsl.create_privileged_client(identity, &description, privilage)?;
    Ok(())
}

#[spacetimedb::reducer]
pub fn revoke_privilage_from_client(ctx: &ReducerContext, client_identity: PrivilegedClientIdentity) -> Result<(), String> {
    if try_server_only(ctx).is_err() {
        panic!("This reducer can only be called by SpacetimeDB!");
    }
    let dsl = dsl(ctx);
    dsl.delete_privileged_client_by_identity(client_identity);

    Ok(())
}