use spacetimedb_sdk::*;

use crate::module_bindings::*;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
//// Subscriptions
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

/// Register subscriptions for all rows of both tables.
pub(super) fn subscribe_to_tables(ctx: &DbConnection) {
    let stellar_object = format!(
        "SELECT o.* 
        FROM stellar_object o
        JOIN ship s ON s.sector_id = o.sector_id
        WHERE s.player_id = '{}'",
        ctx.identity());
    let sobj_player_window = format!(
        "SELECT w.* 
        FROM sobj_player_window w
        WHERE w.player_id = '{}'",
        ctx.identity());
    let sector_chat_message = format!(
        "SELECT c.* 
        FROM sector_chat_message c
        JOIN ship o ON o.sector_id = c.sector_id
        WHERE o.player_id = '{}'",
        ctx.identity());
    let player_ship_controller = format!(
        "SELECT c.* 
        FROM player_ship_controller c
        WHERE c.player_id = '{}'",
        ctx.identity());
    let player_ship = format!(
        "SELECT s.* 
        FROM ship s
        WHERE s.player_id = '{}'",
        ctx.identity());
    let ship_cargo_item = format!(
        "SELECT i.* 
        FROM ship_cargo_item i
        JOIN ship s ON i.ship_id = s.id
        WHERE s.player_id = '{}'",
        ctx.identity());
    let jump_gate = format!(
        "SELECT j.*
        FROM ship s
        JOIN jump_gate j ON s.sector_id = j.current_sector_id
        WHERE s.player_id = '{}'",
        ctx.identity());
    let ship = format!(
        "SELECT o.* 
        FROM ship o
        JOIN ship s ON s.sector_id = o.sector_id
        WHERE s.player_id = '{}'",
        ctx.identity());
    let asteroid = format!(
        "SELECT a.* 
        FROM asteroid a
        JOIN ship s ON s.sector_id = a.current_sector_id
        WHERE s.player_id = '{}'",
        ctx.identity());
    let cargo_crate = format!(
        "SELECT c.* 
        FROM cargo_crate c
        JOIN ship s ON s.sector_id = c.current_sector_id
        WHERE s.player_id = '{}'",
        ctx.identity());
    let sobj_hi_res_transform = "SELECT o.* FROM sobj_hi_res_transform o";
    let sobj_low_res_transform = "SELECT o.* FROM sobj_low_res_transform o";
    // let sobj_hi_res_transform = format!( // TODO: Once we can do two joins enable these after we have a way to specify via sector
    //     "SELECT o.* 
    //     FROM sobj_player_window w
    //     JOIN sobj_hi_res_transform o ON o.sobj_id = w.sobj_id
    //     WHERE (o.x > w.tl_x AND 
    //         o.y > w.tl_y AND 
    //         o.x < w.br_x AND 
    //         o.y < w.br_y) AND w.player_id = '{}'",
    //     ctx.identity());
    // let sobj_low_res_transform = format!( // TODO: Once we can do two joins enable these after we have a way to specify via sector
    //     "SELECT o.* 
    //     FROM sobj_low_res_transform o
    //     JOIN sobj_player_window w ON o.sobj_id = w.sobj_id
    //     WHERE (o.x <= w.tl_x OR 
    //         o.y <= w.tl_y OR 
    //         o.x >= w.br_x OR 
    //         o.y >= w.br_y) AND w.player_id = '{}'",
    //     ctx.identity());

    ctx.subscription_builder()
        .on_applied(on_sub_applied)
        .on_error(on_sub_error)
        .subscribe(vec![
            asteroid.as_str(),
            "SELECT * FROM global_chat_message",
            sector_chat_message.as_str(),
            "SELECT * FROM faction_chat_message",
            "SELECT * FROM faction_definition",
            "SELECT * FROM faction_standing",
            "SELECT * FROM player_faction_standing",
            "SELECT * FROM item_definition",
            cargo_crate.as_str(),
            jump_gate.as_str(),
            "SELECT * FROM player",
            player_ship_controller.as_str(),
            "SELECT * FROM star_system",
            "SELECT * FROM star_system_object",
            "SELECT * FROM sector",
            "SELECT * FROM asteroid_sector",
            "SELECT * FROM ship_type_definition",
            "SELECT * FROM ship_status",
            "SELECT * FROM ship_global",
            player_ship.as_str(),
            ship.as_str(),
            "SELECT * FROM docked_ship",
            ship_cargo_item.as_str(),
            "SELECT * FROM ship_equipment_slot",
            "SELECT * FROM trading_port_module",
            "SELECT * FROM trading_port_listing",
            "SELECT * FROM storage_depot_module",
            "SELECT * FROM embassy_presence",
            "SELECT * FROM embassy_module",
            "SELECT * FROM farm_module",
            "SELECT * FROM observatory_module",
            "SELECT * FROM refinery_module",
            "SELECT * FROM solar_array_module",
            "SELECT * FROM synthesizer_module",
            "SELECT * FROM production_recipe_definition",
            "SELECT * FROM manufacturing_module",
            "SELECT * FROM laboratory_module",
            "SELECT * FROM capital_dock_module",
            "SELECT * FROM docked_capital_ship_at_module",
            "SELECT * FROM anti_capital_turret_module",
            "SELECT * FROM residential_module",
            "SELECT * FROM hospital_module",
            "SELECT * FROM station_module_blueprint",
            "SELECT * FROM station_module",
            "SELECT * FROM station_module_inventory_item",
            "SELECT * FROM station",
            "SELECT * FROM station_status",
            stellar_object.as_str(),
            "SELECT * FROM sobj_velocity",
            sobj_hi_res_transform.as_str(),
            sobj_low_res_transform.as_str(),
            sobj_player_window.as_str(),
    ]);
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

