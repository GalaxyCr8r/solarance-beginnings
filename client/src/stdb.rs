use spacetimedb_sdk::{credentials, DbContext, Error, Event, Identity, Status, Table, TableWithPrimaryKey};

use crate::module_bindings::*;

/// The URI of the SpacetimeDB instance hosting our chat module.
const LOCAL_HOST: &str = "http://localhost:3000";

/// The database name we chose when we published our module.
const DB_NAME: &str = "solarance-beginnings";

pub(crate) fn connect_to_spacetime(jwt_token:Option<String>) -> DbConnection {

    // Connect to the database
    let ctx = connect_to_db(LOCAL_HOST, match jwt_token {
        Some(_) => jwt_token,
        None => creds_store().load().expect("Error loading credentials")
    });

    // Register callbacks to run in response to database events.
    register_callbacks(&ctx);

    // Subscribe to SQL queries in order to construct a local partial replica of the database.
    subscribe_to_tables(&ctx);

    // Spawn a thread, where the connection will process messages and invoke callbacks.
    ctx.run_threaded();

    ctx
}

/// Load credentials from a file and connect to the database.
fn connect_to_db(host: &str, jwt_token:Option<String>) -> DbConnection {
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
        .expect("Failed to connect")
}

fn creds_store() -> credentials::File {
    credentials::File::new("solarance-beginnings-test")
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Connection Callbacks
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

/// Our `on_connect` callback: save our credentials to a file.
fn on_connected(_ctx: &DbConnection, _identity: Identity, token: &str) {
    if let Err(e) = creds_store().save(token) {
        eprintln!("Failed to save credentials: {:?}", e);
    }
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

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Reducer Callbacks
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

/// Register all the callbacks our app will use to respond to database events.
fn register_callbacks(_ctx: &DbConnection) {
    // When a new user joins, print a notification.
    // ctx.db.user().on_insert(on_user_inserted);

    // // When a user's status changes, print a notification.
    // ctx.db.user().on_update(on_user_updated);

    // // When a new message is received, print it.
    // ctx.db.message().on_insert(on_message_inserted);

    // // When we fail to set our name, print a warning.
    // ctx.reducers.on_set_name(on_name_set);

    // // When we fail to send a message, print a warning.
    // ctx.reducers.on_send_message(on_message_sent);
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Subscriptions
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

/// Register subscriptions for all rows of both tables.
fn subscribe_to_tables(ctx: &DbConnection) {
    ctx.subscription_builder()
        .on_applied(on_sub_applied)
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM stellar_object"]);
    ctx.subscription_builder()
        .on_applied(on_sub_applied)
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM stellar_object_hi_res"]);
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Subscription Callbacks
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

/// Our `on_subscription_applied` callback:
/// sort all past messages and print them in timestamp order.
fn on_sub_applied(ctx: &SubscriptionEventContext) {
    // let persons = ctx.db.person().iter().collect::<Vec<_>>();
    // let mut local_person: Option<Person> = None;
    println!("Sub Applied for {}", ctx.identity().to_hex());
    // for person in persons {
    //     //print_message(ctx, &message);
    //     println!("Found person {} ({}) with map set to: {:?}", person.name, person.identity.to_hex(), person.last_view);

    //     // Did we find 'em??
    //     if person.identity == ctx.identity() {
    //         local_person = Some(person);
    //     }
    // }
    println!("Fully connected and all subscriptions applied.");
    //println!("Use /name to set your name, or type a message!");

    // match ctx.db.person().identity().find(&ctx.identity()) {
    //     Some(person) => println!("We last used the {:?} map view!", person.last_view),
    //     None => {
    //         println!("Could not find your person. Creating them now.");
    //         let _ = ctx.reducers.add_person("Henlo I'm name".into());
    //     }
    // }

    // match ctx.db.person().identity().find(&ctx.identity()) {
    //     person => println!("Found our old player instance! {:?}", person.unwrap().last_view),
    //     None => {
    //         eprintln!("Could not find player. Maybe we aren't created yet?");
    //         let _ = ctx.reducers.add_person("Henlo I'm name".into());
    //     }
    // }
}

/// Or `on_error` callback:
/// print the error, then exit the process.
fn on_sub_error(_ctx: &ErrorContext, err: Error) {
    eprintln!("Subscription failed: {}", err);
    std::process::exit(1);
}

