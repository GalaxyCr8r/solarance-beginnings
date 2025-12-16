use spacetimedsl::*;

use crate::{
    logic::ships::add_cargo_timer::*,
    tables::{items::*, ships::*, stellarobjects::*},
};

pub fn attempt_to_pickup_cargo_crate(
    dsl: &DSL<T>,
    player_ship_obj: &Ship,
    crate_sobj: &StellarObject,
) -> Result<(), String> {
    let cargo_crate = dsl.get_cargo_crate_by_sobj_id(crate_sobj)?;
    let item_def = dsl.get_item_definition_by_id(cargo_crate.get_item_id())?;
    let ship = dsl.get_ship_status_by_id(player_ship_obj.get_id())?;

    if item_def.can_any_of_this_fit_inside_this_ship(&ship) {
        match create_timer_to_add_cargo_to_ship(
            // Do the actual thing
            dsl,
            ship.get_id(),
            item_def.get_id(),
            *cargo_crate.get_quantity(),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "ERROR {} : Ship {:?} could not fit {}x #{:?} items",
                e,
                ship.get_id(),
                *cargo_crate.get_quantity(),
                item_def.get_id()
            )),
        }
    } else {
        Err(format!(
            "Ship {:?} could not fit {}x #{:?} items",
            ship.get_id(),
            *cargo_crate.get_quantity(),
            item_def.get_id()
        ))
    }
}
