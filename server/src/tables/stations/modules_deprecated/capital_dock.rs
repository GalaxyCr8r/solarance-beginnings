use super::*;
use crate::tables::ships;

#[dsl(plural_name = capital_dock_modules, method(update = true))]
#[table(name = capital_dock_module, public)]
pub struct CapitalDock {
    #[primary_key]
    #[use_wrapper(StationModuleId)]
    /// FK to StationModule
    id: u64,

    pub max_capital_ship_capacity: u8, // e.g., 10
}

/// Tracks capital ships currently docked at a specific Capital Dock module.
#[dsl(plural_name = docked_capital_ship_at_modules, method(update = true))]
#[table(name = docked_capital_ship_at_module, public)]
pub struct DockedCapitalShipAt {
    #[primary_key]
    #[use_wrapper(ships::ShipId)]
    id: u64, // FK to Ship (must be a capital ship)

    #[index(btree)]
    #[use_wrapper(StationModuleId)]
    pub capital_dock_module_id: u64, // FK to StationModuleInstance (a CapitalDock)

    pub docked_at_timestamp: Timestamp,
}
