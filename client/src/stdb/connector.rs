use macroquad::prelude::info;
use spacetimedb_sdk::{credentials, Error, Identity};
use std::env;

use crate::server::bindings::*;

mod subscriptions;

/// Connection ///

/// The URI of the SpacetimeDB instance hosting our chat module.
const LOCAL_HOST: &str = "http://localhost:3000";

/// The database name we chose when we published our module.
const DB_NAME: &str = "solarance-beginnings";

/// How the player asked to get in, which is not the same question as "do we
/// have a token lying around" (#216).
///
/// The old signature was `Option<String>`, and `None` meant both "the player
/// chose Guest" and "we have no token, use whatever is in the creds store".
/// Those want opposite behavior, so an Auth0 handshake followed by *Play as
/// Guest* connected with the Auth0 token — and a later *Play via Auth0*
/// correctly resumed that "guest" ship, because it had been the Auth0 identity
/// all along.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Session {
    /// Mint a fresh identity. Never reuse a stored token, however convenient.
    Guest,
    /// Connect as exactly this token — an Auth0 id_token, or the one the
    /// *Continue* button loaded out of the creds store.
    Token(String),
}

pub fn connect_to_spacetime(session: Session) -> Option<DbConnection> {
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

    // (#216) Guest connects with no token so SpacetimeDB mints a new identity.
    // There is deliberately no fallback to the creds store on either branch:
    // substituting a stored token for one the player didn't offer is the whole
    // bug, and on the `Token` branch it also silently masked auth failures by
    // logging the player in as whoever last used this machine.
    let token = match &session {
        Session::Guest => None,
        Session::Token(token) => Some(token.clone()),
    };

    let ctx = match connect_to_db(host, token) {
        Ok(ctx) => ctx,
        Err(e) => {
            info!("CONNECTION ERROR : {}", e);
            return None;
        }
    };

    // Whatever identity we end up as — guest included — `on_connected` saves
    // its token, so the *Continue* button can resume this session next launch.

    // Spawn a thread, where the connection will process messages and invoke
    // callbacks.
    ctx.run_threaded();

    Some(ctx)
}

/// Load credentials from a file and connect to the database.
fn connect_to_db(host: String, jwt_token: Option<String>) -> Result<DbConnection, String> {
    match DbConnection::builder()
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
        .with_database_name(DB_NAME)
        // Set the URI of the SpacetimeDB host that's running our database.
        .with_uri(host)
        // Finalize configuration and connect!
        .build()
    {
        Ok(connection) => Ok(connection),
        Err(e) => Err(e.to_string()),
    }
}

pub fn creds_store() -> credentials::File {
    credentials::File::new("solarance-beginnings-test")
}

/// The name of the pilot we last played as (#171), so the login screen can say
/// "Continue as {name}" instead of a bare "Continue".
///
/// ponytail: this is a `credentials::File` holding a username rather than a
/// token — same API, same directory, a sibling of the cred file. A pilot name
/// is not a secret, so nothing is lost by storing it this way, and it saves
/// inventing a second persistence path for one string.
///
/// **Stopgap — belongs in the client config from #29** ("Add proper
/// configuration to store OIDC info and other things", which calls out the
/// refresh token and last player name by name). The SDK writes under the OS
/// user's home dir, so this is already per-OS-user, but `~/.spacetime_data/`
/// is the SDK's private location, not a config path any OS would point you
/// at — macOS wants `~/Library/Application Support/`, Windows `%APPDATA%`,
/// Linux `$XDG_CONFIG_HOME`. When #29 lands, the last-played pilot name and
/// the session token should move into that config alongside the Auth0
/// settings and this function should go away.
pub fn pilot_name_store() -> credentials::File {
    credentials::File::new("solarance-beginnings-test-pilot")
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
// Connection Callbacks
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

/// Our `on_connect` callback: save our credentials to a file.
fn on_connected(ctx: &DbConnection, identity: Identity, token: &str) {
    info!(
        "Successfully connected with idenitifer: {}",
        identity.to_abbreviated_hex()
    );
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
