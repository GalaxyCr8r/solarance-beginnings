use log::info;
use spacetimedb::{table, Identity, ReducerContext, SpacetimeType};
use spacetimedsl::{dsl, Wrapper};

use crate::types::{items::utility::*, stellarobjects::StellarObjectId, common::*, items::*, sectors::*, stations::*};

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
    #[wrap]
    pub id: u32,

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
pub struct ShipStatus {
    #[primary_key]
    #[wrapped(path = ShipGlobalId)]
    pub id: u64,

    #[index(btree)] // To easily find ships in a given sector
    #[wrapped(path = SectorId)]
    pub sector_id: u64, // FK to Sector.id // Needs to be kept in sync with StellarObject.sector_id

    #[index(btree)]
    pub player_id: Identity,      // FK to player.id // You should only be able to see your ship, or other ships in your sector.

    pub health: f32,
    pub shields: f32,
    pub energy: f32,

    pub used_cargo_capacity: u16, // Needs to be manually maintained via ShipCargoItem
    pub max_cargo_capacity: u16, // Needs to be manually maintained via ShipCargoItem

    pub ai_state: Option<CurrentAction>, // Current high-level AI state or player command
}

/// Because we can have ships created both docked and not docked - we need a central table to create the IDs in a controlled way.
#[dsl(plural_name = ship_globals)]
#[table(name = ship_global, public)]
pub struct ShipGlobal { 
    #[primary_key] // Due to Ship/DockedShip tables both wanting the use the same ID, we can no longer use AutoInc for them
    #[auto_inc]
    #[wrap]
    pub id: u64,
    
    //pub custom_name: Option<String>,
}

#[dsl(plural_name = ships)]
#[table(name = ship, public)]
pub struct Ship { 
    #[primary_key]
    #[wrapped(path = ShipGlobalId)]
    pub id: u64,

    #[index(btree)]
    #[wrapped(path = ShipTypeDefinitionId)]
    pub shiptype_id: u32, // FK to ShipTypeDefinition.id

    #[unique]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    #[index(btree)]
    #[wrapped(path = crate::types::sectors::SectorId)]
    pub sector_id: u64, // FK to Sector ID - Only because actually referencing the player's stellar object would require three table hits.

    #[index(btree)]
    pub player_id: Identity, // FK to player.id
    
    #[wrapped(path = crate::types::factions::FactionId)]
    pub faction_id: u32, // FK to faction.id
}

#[dsl(plural_name = docked_ships)]
#[table(name = docked_ship, public)] // TODO Create utility functions to switch ships between docked and non-docked
pub struct DockedShip { 
    #[primary_key]
    #[wrapped(path = ShipGlobalId)]
    pub id: u64,

    #[index(btree)]
    #[wrapped(path = ShipTypeDefinitionId)]
    pub shiptype_id: u32, // FK to ShipTypeDefinition.id

    #[index(btree)]
    #[wrapped(path = StationId)]
    pub station_id: u64, // FK: Station

    #[index(btree)]
    #[wrapped(path = SectorId)]
    pub sector_id: u64, // FK to Sector ID - Only because actually referencing the player's stellar object would require three table hits.

    #[index(btree)]
    pub player_id: Identity, // FK to player.id
    
    #[wrapped(path = crate::types::factions::FactionId)]
    pub faction_id: u32, // FK to faction.id
}

#[dsl(plural_name = ship_cargo_items)]
#[table(name = ship_cargo_item, public)]
pub struct ShipCargoItem {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[index(btree)] // To query all cargo for a specific ship
    #[wrapped(path = ShipGlobalId)]
    pub ship_id: u64, // FK to ShipGlobal

    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    pub item_id: u32, // FK to ItemDefinition

    pub quantity: u16, // How many of this item are currently in this stack
    //pub stack_size: u8, // TODO: Do we keep this value here to save query time?
}

#[dsl(plural_name = ship_equipment_slots)]
#[table(name = ship_equipment_slot, public)]
pub struct ShipEquipmentSlot {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[index(btree)] // To query all equipment for a specific ship
    #[wrapped(path = ShipGlobalId)]
    pub ship_id: u64, // FK to ShipGlobal

    pub slot_type: EquipmentSlotType,
    pub slot_index: u8, // E.g., Weapon Slot 0, Weapon Slot 1 within its type

    #[wrapped(path = ItemDefinitionId)]
    pub item_id: u32, // FK to ItemDefinition
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    definitions::init(ctx)?;
    timers::init(ctx)?;
    
    Ok(())
}

