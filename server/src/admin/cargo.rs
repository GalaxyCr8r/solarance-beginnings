use std::str::FromStr;

use log::info;
use spacetimedb::{Identity, ReducerContext};
use crate::spacetimedsl::prelude::*;

use crate::logic::ships::cargo::attempt_to_load_cargo_into_ship;
use crate::tables::items::*;
use crate::tables::players::{get_player_ship_and_sobj, PlayerId};
use crate::tables::ships::*;
use crate::utility::try_server_only;

/// Admin/seed reducer for M1 spike testing: spawn `quantity` units of
/// item `item_id` into the in-sector ship belonging to `target_player_id`.
///
/// Loads through `attempt_to_load_cargo_into_ship` so capacity and stacking
/// rules match the real game; over-capacity contributions fail loudly
/// (no crate spawning) so the designer notices and adjusts the test.
#[spacetimedb::reducer]
pub fn admin_spawn_cargo_in_player_ship(
    ctx: &ReducerContext,
    target_player_id_str: &str,
    item_id: u32,
    quantity: u16,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;
    let target_player_id = Identity::from_str(target_player_id_str).map_err(|e| {
        format!(
            "admin_spawn_cargo_in_player_ship: invalid target_player_id_str {:?}: {}",
            target_player_id_str, e
        )
    })?;

    if quantity == 0 {
        return Err("admin_spawn_cargo_in_player_ship: quantity must be > 0".to_string());
    }

    let target = PlayerId::new(target_player_id);
    let (ship, _sobj) = get_player_ship_and_sobj(&dsl, &target)?;
    let item_def = dsl.get_item_definition_by_id(&ItemDefinitionId::new(item_id))?;
    let mut ship_status = dsl.get_ship_status_by_id(&ship.get_id())?;

    info!(
        "admin_spawn_cargo_in_player_ship: caller={} target_player={} item_id={} ({}) quantity={} ship_id={}",
        ctx.sender().to_abbreviated_hex(),
        target_player_id.to_abbreviated_hex(),
        item_id,
        item_def.get_name(),
        quantity,
        ship.get_id().value(),
    );

    attempt_to_load_cargo_into_ship(
        ctx,
        &dsl,
        &mut ship_status,
        &ship.get_id(),
        &item_def,
        quantity,
        false,
    )
}
