use spacetimedb_sdk::*;

use crate::module_bindings::*;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
//// Subscriptions
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

/// Register subscriptions for all rows of both tables.
pub(super) fn subscribe_to_tables(ctx: &DbConnection) {
    ctx.subscription_builder()
        .on_applied(on_sub_applied)
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM global_chat_message"]);
    ctx.subscription_builder()
        .on_applied(on_sub_applied)
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM stellar_object"]);
    ctx.subscription_builder()
        .on_applied(on_sub_applied)
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM sobj_hi_res_transform"]);
    ctx.subscription_builder()
        .on_applied(on_sub_applied)
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM sobj_low_res_transform"]);
    ctx.subscription_builder()
        .on_applied(on_sub_applied)
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM sobj_velocity"]);
    ctx.subscription_builder()
        .on_applied(on_sub_applied)
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM player"]);
    ctx.subscription_builder()
        .on_applied(on_sub_applied)
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM player_controlled_stellar_object"]);
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Subscription Callbacks
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

/// Our `on_subscription_applied` callback:
/// sort all past messages and print them in timestamp order.
fn on_sub_applied(ctx: &SubscriptionEventContext) {
    println!("Subscription Successfully Applied for {}", ctx.identity().to_hex());

    // let persons = ctx.db.person().iter().collect::<Vec<_>>();
    // let mut local_person: Option<Person> = None;
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
    // TODO Make a message here suggesting you might be on the wrong version.
    std::process::exit(1);
}

