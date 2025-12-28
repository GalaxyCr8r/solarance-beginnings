use super::*;
use crate::tables::ships;

#[dsl(plural_name = anti_capital_turret_modules, method(update = true))]
#[table(name = anti_capital_turret_module, public)]
pub struct AntiCapitalTurret {
    #[primary_key]
    #[use_wrapper(StationModuleId)]
    /// FK to StationModule
    id: u64,

    /// FK to a ShipModuleBlueprint that defines the weapon's stats (damage, range, fire rate)
    #[use_wrapper(crate::tables::items::ItemDefinitionId)]
    pub weapon_core_blueprint_id: u32,

    #[use_wrapper(ships::ShipId)]
    pub current_target_ship_id: Option<u64>, // FK to ShipInstance

    pub can_launch_fighters: bool,
    pub fighter_capacity: Option<u8>,
    // Fighters stored here would be ShipInstances linked to this module, perhaps in a `ShipAtModule` table.
    // Ammo and fuel are in StationModuleInventoryItem.
}
