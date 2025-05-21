use spacetimedb::{table, Identity, ReducerContext, SpacetimeType, Timestamp};
use spacetimedsl::dsl;

use crate::types::stellarobjects::StellarObjectId;

use super::{common::{EntityAIState, EquipmentSlotType}, sector::SectorId};

pub mod definitions;
pub mod rls;
pub mod utility;

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
    pub max_shield: u16,
    pub max_energy: u16,

    pub base_speed: f32,
    pub base_acceleration: f32,
    pub base_turn_rate: f32, // Radians per second

    pub cargo_capacity: u16, // Max cargo volume

    pub num_weapon_slots: u8,
    pub num_shield_slots: u8,
    pub num_engine_slots: u8,
    pub num_mining_laser_slots: u8,
    pub num_special_slots: u8,

    pub gfx_key: Option<String>, // Key for client to look up 2D sprite/model
}

#[dsl(plural_name = ship_instances)]
#[table(name = ship_instance, public)]
pub struct ShipInstance {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[wrapped(path = ShipTypeDefinitionId)]
    pub shiptype_id: u32,           // FK to ShipTypeDefinition.id

    pub owner_id: Option<Identity>,      // FK to player.id
    #[wrapped(path = crate::types::factions::FactionDefinitionId)]
    pub faction_id: Option<u32>,    // FK to faction.id
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: Option<u64>, // FK: StellarObject

    #[index(btree)] // To easily find ships in a given sector
    #[wrapped(path = SectorId)]
    pub current_sector_id: u64, // FK to Sector.id // Needs to be kept in sync with StellarObject.sector_id

    pub health: f32,
    pub shields: f32,
    pub energy: f32,

    pub cargo_capacity: u16, // Needs to be manually maintained via ShipCargoItem

    pub ai_state: Option<EntityAIState>, // Current high-level AI state or player command
    pub docked_at_station_id: Option<u64>, // FK to a potential Station table

    pub last_update_ts: Timestamp, // For server-side logic or client interpolation
}

#[dsl(plural_name = ship_objects)]
#[table(name = ship_object, public)]//, index(name = ship_and_sobj, btree(columns = [ship_id, sobj_id])))]
// This table duplicates PlayerControlledStellarObject, but because RLS doesn't allow NULLs we kind-of have to.
pub struct ShipObject { 
    #[primary_key]
    #[wrapped(path = ShipInstanceId)]
    pub ship_id: u64, // FK: Ship

    #[unique]
    #[wrapped(path = StellarObjectId)]
    pub sobj_id: u64, // FK: StellarObject

    #[index(btree)]
    #[wrapped(path = crate::types::sector::SectorId)]
    pub sector_id: u64, // FK to Sector ID - Only because actually referencing the player's stellar object would require three table hits.

    #[index(btree)]
    pub player_id: Identity,   // FK to player.id
}

#[dsl(plural_name = ship_cargo_items)]
#[table(name = ship_cargo_item, public)]
pub struct ShipCargoItem {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[index(btree)] // To query all cargo for a specific ship
    pub ship_id: u64, // FK to Ship
    pub item_id: u32, // FK to ItemDefinition
    pub quantity: u32,
}

#[dsl(plural_name = ship_equipment_slots)]
#[table(name = ship_equipment_slot, public)]
pub struct ShipEquipmentSlot {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[index(btree)] // To query all equipment for a specific ship
    pub ship_id: u64, // FK to Ship

    pub slot_type: EquipmentSlotType,
    pub slot_index: u8, // E.g., Weapon Slot 0, Weapon Slot 1 within its type
    pub equipped_item_id: Option<u32>, // FK to ItemDefinition (if an item is equipped)
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    definitions::init(ctx)?;
    
    Ok(())
}

