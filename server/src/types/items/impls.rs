use super::*;

//////////////////////////////////////////////////////////////
// Impls
//////////////////////////////////////////////////////////////

impl ItemDefinition {
    pub fn can_any_of_this_fit_inside_this_ship(&self, ship_status: &ShipStatus) -> bool {
        (ship_status.get_remaining_cargo_space() / self.volume_per_unit) > 0
    }
}