use log::info;
use spacetimedb::{table, Identity, ReducerContext, SpacetimeType};
use spacetimedsl::{dsl, Wrapper};

use crate::types::{
    common::*, items::utility::*, items::*, sectors::*, stations::*,
    stellarobjects::StellarObjectId,
};

pub mod definitions; // Definitions for initial ingested data.
pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum ShipClass {
    Miner,
    Shuttle,
    Freighter,
    Fighter,
    Scout,
    Cruiser,
    BattleCruiser,
    Carrier,
}

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum ShipLocation {
    System,  // Interplanetary travel?
    Sector,  // Regular flying around
    Station, // Docked at a station
    Ship,    // Docked at a ship
}

// Enum for different types of equipment slots on a ship
#[derive(SpacetimeType, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EquipmentSlotType {
    Weapon,
    Shield,
    Engine,
    MiningLaser,
    Special, // For things like cloaking devices, tractor beams etc.
    CargoExpansion,
}

#[dsl(plural_name = ship_type_definitions)]
#[table(name = ship_type_definition, public)]
pub struct ShipTypeDefinition {
    #[primary_key] // NOT Auto-inc so it can be reloaded as-is
    #[create_wrapper]
    #[referenced_by(path = crate::types::ships, table = ship)]
    id: u32,

    pub name: String, // E.g., "Fighter Mk1", "Heavy Hauler"
    pub description: Option<String>,

    #[index(btree)]
    pub class: ShipClass,

    pub max_health: u16,
    pub max_shields: u16,
    pub max_energy: u16,

    pub base_speed: f32,
    pub base_acceleration: f32,
    pub base_turn_rate: f32, // Radians per second

    pub cargo_capacity: u16, // Max cargo volume

    pub num_weapon_slots: u8,
    pub num_large_weapon_slots: u8,
    pub num_turret_slots: u8,
    pub num_large_turret_slots: u8,
    pub num_shield_slots: u8,
    pub num_engine_slots: u8,
    pub num_mining_laser_slots: u8,
    pub num_special_slots: u8,

    pub gfx_key: Option<String>, // Key for client to look up 2D sprite/model
}

#[dsl(plural_name = ship_statuses)]
#[table(name = ship_status, public)]
/// The status of a ship agnostic of where it is physically.
pub struct ShipStatus {
    #[primary_key]
    #[use_wrapper(path = ShipId)]
    id: u64,

    #[index(btree)] // To easily find ships in a given sector
    #[use_wrapper(path = SectorId)]
    /// FK to Sector.id // Needs to be kept in sync with StellarObject.sector_id
    pub sector_id: u64,

    #[index(btree)]
    #[use_wrapper(path = crate::players::PlayerId)]
    /// FK to player.id // You should only be able to see your ship, or other ships in your sector.
    pub player_id: Identity,

    pub health: f32,
    pub shields: f32,
    pub energy: f32,

    pub used_cargo_capacity: u16, // Needs to be manually maintained via ShipCargoItem
    pub max_cargo_capacity: u16,  // Needs to be manually maintained via ShipCargoItem

    pub ai_state: Option<CurrentAction>, // Current high-level AI state or player command
}

#[dsl(plural_name = ships)]
#[table(name = ship, public)]
pub struct Ship {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::types::ships, table = ship_cargo_item)]
    #[referenced_by(path = crate::types::ships, table = ship_equipment_slot)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(path = ShipTypeDefinitionId)]
    #[foreign_key(path = crate::types::ships, table = ship_type_definition, column = id, on_delete = Error)]
    /// FK to ShipTypeDefinition.id
    pub shiptype_id: u32,

    /// Where is the ship currently located? Is it docked or currently flying?
    pub location: ShipLocation,

    #[unique]
    #[use_wrapper(path = StellarObjectId)]
    #[foreign_key(path = crate::types::stellarobjects, table = stellar_object, column = id, on_delete = Ignore)]
    /// FK to StellarObject
    pub sobj_id: u64,

    #[index(btree)]
    #[use_wrapper(path = StationId)]
    #[foreign_key(path = crate::types::stations, table = station, column = id, on_delete = Error)]
    /// FK to Station
    pub station_id: u64, // TODO - STDSL doesn't allow this to be `pub station_id: Option<u64>,` for some reason.

    #[index(btree)]
    #[use_wrapper(path = crate::types::sectors::SectorId)]
    #[foreign_key(path = crate::types::sectors, table = sector, column = id, on_delete = Error)]
    /// FK to Sector ID - Only because actually referencing the player's stellar object would require three table hits.
    pub sector_id: u64,

    #[index(btree)]
    #[use_wrapper(path = crate::players::PlayerId)]
    #[foreign_key(path = crate::players, table = player, column = id, on_delete = Error)]
    /// FK to player.id
    pub player_id: Identity,

    #[index(btree)]
    #[use_wrapper(path = crate::types::factions::FactionId)]
    #[foreign_key(path = crate::types::factions, table = faction_definition, column = id, on_delete = Error)]
    /// FK to faction.id
    pub faction_id: u32,
}

#[dsl(plural_name = ship_cargo_items)]
#[table(name = ship_cargo_item, public)]
pub struct ShipCargoItem {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)] // To query all cargo for a specific ship
    #[use_wrapper(path = ShipId)]
    #[foreign_key(path = crate::types::ships, table = ship, column = id, on_delete = Delete)]
    /// FK to Ship
    pub ship_id: u64,

    #[index(btree)]
    #[use_wrapper(path = crate::types::items::ItemDefinitionId)]
    #[foreign_key(path = crate::types::items, table = item_definition, column = id, on_delete = Error)]
    /// FK to ItemDefinition
    pub item_id: u32,

    pub quantity: u16, // How many of this item are currently in this stack
                       //pub stack_size: u8, // TODO: Do we keep this value here to save query time?
}

#[dsl(plural_name = ship_equipment_slots)]
#[table(name = ship_equipment_slot, public)]
pub struct ShipEquipmentSlot {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)] // To query all equipment for a specific ship
    #[use_wrapper(path = ShipId)]
    #[foreign_key(path = crate::types::ships, table = ship, column = id, on_delete = Delete)]
    /// FK to Ship
    pub ship_id: u64,

    pub slot_type: EquipmentSlotType,
    pub slot_index: u8, // E.g., Weapon Slot 0, Weapon Slot 1 within its type

    #[index(btree)]
    #[use_wrapper(path = ItemDefinitionId)]
    #[foreign_key(path = crate::types::items, table = item_definition, column = id, on_delete = Error)]
    /// FK to ItemDefinition
    pub item_id: u32,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    definitions::init(ctx)?;
    timers::init(ctx)?;

    Ok(())
}
