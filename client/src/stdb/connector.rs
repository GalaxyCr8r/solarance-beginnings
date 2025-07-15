use macroquad::prelude::info;
use spacetimedb_sdk::{ credentials, Error, Identity };
use std::env;

use crate::module_bindings::*;

mod subscriptions;

/// Connection ///

/// The URI of the SpacetimeDB instance hosting our chat module.
const LOCAL_HOST: &str = "http://localhost:3000";

/// The database name we chose when we published our module.
const DB_NAME: &str = "solarance-beginnings";

pub fn connect_to_spacetime(jwt_token: Option<String>) -> Option<DbConnection> {
    info!(" Connecting to SpacetimeDB ...");

    // Connect to the database
    let host = {
        let result = env::var("DATABASE_HOST").unwrap_or(LOCAL_HOST.to_string());
        if result.is_empty() {
            LOCAL_HOST.to_string()
        } else {
            result
        }
    };

    let mut tried_loaded_token = false;
    let mut current_token = jwt_token.clone();

    // let mut current_token  = token.clone();

    loop {
        let result = connect_to_db(host.clone(), current_token.clone());
        if let Err(e) = result {
            info!("CONNECTION ERROR : {}", e);
            if !tried_loaded_token {
                tried_loaded_token = true;
                current_token = match creds_store().load() {
                    Ok(token) => {
                        info!("Loading token from creds store.");
                        token
                    }
                    Err(e) => {
                        info!("Failed to load token from creds_store! {:?}", e);
                        return None;
                    }
                };
                info!("Failed to connect, retrying...");
            } else {
                return None;
            }

            continue;
        }

        if current_token.is_some() {
            info!("Token connected successfully. Storing for future use.");
            let _ = creds_store().save(current_token.unwrap());
        }

        let ctx = result.unwrap();

        // Subscribe to SQL queries in order to construct a local partial replica of the database.
        //subscriptions::subscribe_to_tables(&ctx);

        // Spawn a thread, where the connection will process messages and invoke callbacks.
        ctx.run_threaded();

        return Some(ctx);
    }
}

/// Load credentials from a file and connect to the database.
fn connect_to_db(host: String, jwt_token: Option<String>) -> Result<DbConnection, String> {
    match
        DbConnection::builder()
            // Register our `on_connect` callback, which will save our auth token.
            .on_connect(on_connected)
            // Register our `on_connect_error` callback, which will print a message, then exit the process.
            .on_connect_error(on_connect_error)
            // Our `on_disconnect` callback, which will print a message, then exit the process.
            .on_disconnect(on_disconnected)
            // If the user has previously connected, we'll have saved a token in the `on_connect` callback.
            // In that case, we'll load it and pass it to `with_token`,
            // so we can re-authenticate as the same `Identity`.
            .with_token(jwt_token)
            // Set the database name we chose when we called `spacetime publish`.
            .with_module_name(DB_NAME)
            // Set the URI of the SpacetimeDB host that's running our database.
            .with_uri(host)
            // Finalize configuration and connect!
            .build()
    {
        Ok(connection) => Ok(connection),
        Err(e) => Err(e.to_string()),
    }
}

fn creds_store() -> credentials::File {
    credentials::File::new("solarance-beginnings-test")
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
// Connection Callbacks
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

/// Our `on_connect` callback: save our credentials to a file.
fn on_connected(ctx: &DbConnection, identity: Identity, token: &str) {
    info!("Successfully connected with idenitifer: {}", identity.to_abbreviated_hex());
    if let Err(e) = creds_store().save(token) {
        eprintln!("Failed to save credentials: {:?}", e);
    }
    subscriptions::subscribe_to_tables(&ctx);
}

/// Our `on_connect_error` callback: print the error, then exit the process.
fn on_connect_error(_ctx: &ErrorContext, err: Error) {
    eprintln!("Connection error: {:?}", err);
    std::process::exit(1);
}

/// Our `on_disconnect` callback: print a note, then exit the process.
fn on_disconnected(_ctx: &ErrorContext, err: Option<Error>) {
    if let Some(err) = err {
        eprintln!("Disconnected: {}", err);
        std::process::exit(1);
    } else {
        println!("Disconnected.");
        std::process::exit(0);
    }
}
