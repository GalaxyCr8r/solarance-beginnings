use spacetimedsl::*;

use crate::{
    logic::ships::add_cargo_timer::*,
    tables::{items::*, ships::*},
};

// Creates a timer to try to add the cargo to the ship if there looks like there will be enough space for it.
pub fn attempt_to_pickup_cargo_crate<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    cargo_crate: &CargoCrate,
    item_def: &ItemDefinition,
    ship_status: &ShipStatus,
) -> Result<(), String> {
    if ship_status.get_sector_id() != cargo_crate.get_current_sector_id() {
        return Err(format!(
            "Ship {} isn't in the same sector as cargo crate {}!",
            ship_status.get_id(),
            cargo_crate.get_id()
        ));
    }

    if item_def.can_any_of_this_fit_inside_this_ship(&ship_status) {
        match create_timer_to_add_cargo_to_ship(
            // Do the actual thing
            dsl,
            ship_status.get_id(),
            item_def.get_id(),
            *cargo_crate.get_quantity(),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "ERROR {} : Ship {:?} could not fit {}x #{:?} items",
                e,
                ship_status.get_id(),
                *cargo_crate.get_quantity(),
                item_def.get_id()
            )),
        }
    } else {
        Err(format!(
            "Ship {:?} could not fit {}x #{:?} items",
            ship_status.get_id(),
            *cargo_crate.get_quantity(),
            item_def.get_id()
        ))
    }
}
