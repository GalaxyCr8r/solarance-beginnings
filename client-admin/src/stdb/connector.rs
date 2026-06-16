//! SpacetimeDB connection plumbing for the Galaxy Creator.
//!
//! Unlike the player client (Auth0 / guest tokens), the admin tool connects
//! with the **module owner's secret token** so that `try_server_only` on the
//! server passes. The token, host, and database name all come from the startup
//! connection dialog (see `app.rs`).
//!
//! Async SDK callbacks run on the connection's background thread; they cannot
//! return values to the UI, so they funnel status into two module-level
//! channels that the UI thread drains each frame: `CONNECT_ERROR` (last fatal
//! connection error) and `ACTIVITY_LOG` (human-readable scrollback).

use std::sync::Mutex;

use macroquad::prelude::info;
use spacetimedb_sdk::{DbContext, Error, Identity};

use crate::server::bindings::*;

/// Well-known SpacetimeDB hosts offered in the connection dialog. Either can be
/// overridden with a free-text host in the UI.
pub const LOCAL_HOST: &str = "http://localhost:3000";
pub const MAINCLOUD_HOST: &str = "https://maincloud.spacetimedb.com";

/// Default database (module) name — matches `task server:publish`.
pub const DEFAULT_DB_NAME: &str = "solarance-beginnings";

/// Last fatal connection error reported by the async SDK callback. The UI
/// thread takes it to bounce the user back to the connection dialog.
static CONNECT_ERROR: Mutex<Option<String>> = Mutex::new(None);

/// Append-only activity scrollback shown in the UI. Reducer result callbacks
/// and connection callbacks push here; the UI thread snapshots it each frame.
static ACTIVITY_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Push a line onto the activity scrollback (bounded to the last 200 lines).
pub fn log_activity(line: impl Into<String>) {
    if let Ok(mut log) = ACTIVITY_LOG.lock() {
        log.push(line.into());
        let len = log.len();
        if len > 200 {
            log.drain(0..len - 200);
        }
    }
}

/// Snapshot the activity scrollback for rendering.
pub fn activity_log_snapshot() -> Vec<String> {
    ACTIVITY_LOG.lock().map(|log| log.clone()).unwrap_or_default()
}

/// Take (and clear) the last fatal connection error, if any.
pub fn take_connect_error() -> Option<String> {
    CONNECT_ERROR.lock().ok().and_then(|mut slot| slot.take())
}

/// Build a connection to `host`/`db_name` authenticating with `token`, and
/// start processing messages on a background thread. Returns immediately; the
/// caller polls [`DbContext::try_identity`] to know when the handshake landed.
pub fn connect(
    host: &str,
    db_name: &str,
    token: Option<String>,
) -> Result<DbConnection, String> {
    if let Ok(mut slot) = CONNECT_ERROR.lock() {
        *slot = None;
    }
    info!("Galaxy Creator connecting to {host} / {db_name}");
    log_activity(format!("Connecting to {host} / {db_name} …"));

    let connection = DbConnection::builder()
        .on_connect(on_connected)
        .on_connect_error(on_connect_error)
        .on_disconnect(on_disconnected)
        .with_token(token)
        .with_database_name(db_name)
        .with_uri(host)
        .build()
        .map_err(|e| e.to_string())?;

    connection.run_threaded();
    Ok(connection)
}

/// `on_connect`: log our identity and open subscriptions for the tables the
/// Galaxy Creator reads.
fn on_connected(ctx: &DbConnection, identity: Identity, _token: &str) {
    log_activity(format!("Connected as {}", identity.to_abbreviated_hex()));
    subscribe_to_tables(ctx);
}

/// `on_connect_error`: record the error so the UI can recover. Unlike the player
/// client, we deliberately do NOT `process::exit` — the admin should be able to
/// fix the host/token and retry without relaunching.
fn on_connect_error(_ctx: &ErrorContext, err: Error) {
    let msg = format!("Connection error: {err}");
    log_activity(msg.clone());
    if let Ok(mut slot) = CONNECT_ERROR.lock() {
        *slot = Some(msg);
    }
}

/// `on_disconnect`: note it in the scrollback. The UI notices the dropped
/// identity and returns to the connection dialog.
fn on_disconnected(_ctx: &ErrorContext, err: Option<Error>) {
    let msg = match err {
        Some(err) => format!("Disconnected: {err}"),
        None => "Disconnected.".to_string(),
    };
    log_activity(msg);
}

/// Subscribe to the public tables backing the dropdowns and listings. All are
/// small reference tables, so a full `SELECT *` is fine.
fn subscribe_to_tables(ctx: &DbConnection) {
    ctx.subscription_builder()
        .on_applied(|_ctx| log_activity("Subscription applied — galaxy data loaded."))
        .on_error(|_ctx, err| log_activity(format!("Subscription error: {err}")))
        .subscribe(vec![
            "SELECT * FROM star_system",
            "SELECT * FROM sector",
            "SELECT * FROM faction",
            "SELECT * FROM station",
            "SELECT * FROM jump_gate",
            "SELECT * FROM item_definition",
        ]);
}
