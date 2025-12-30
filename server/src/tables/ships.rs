use log::info;
use spacetimedb::{table, Identity, SpacetimeType};
use spacetimedsl::*;

use crate::tables::{
    common_types::*, items::*, players::PlayerId, sectors::*, stations::*, stellarobjects::*,
};

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
    /// Interplanetary travel?
    System,
    /// Regular flying around
    Sector,
    /// Docked at a station
    Station,
    /// Docked at a ship
    Ship,
}

// Enum for different types of equipment slots on a ship
#[derive(SpacetimeType, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EquipmentSlotType {
    Weapon,
    Shield,
    Engine,
    MiningLaser,
    /// For things like cloaking devices, tractor beams etc.
    Special,
    CargoExpansion,
}

#[dsl(plural_name = ship_type_definitions, method(update = true))] // One could argue that this should be false. idk atm - Karl
#[table(name = ship_type_definition, public)]
pub struct ShipTypeDefinition {
    #[primary_key] // NOT Auto-inc so it can be reloaded as-is
    #[create_wrapper]
    #[referenced_by(path = crate::tables::ships, table = ship)]
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

    pub sprite_width: u16,  // Width of the ship sprite in pixels
    pub sprite_height: u16, // Height of the ship sprite in pixels

    pub gfx_key: Option<String>, // Key for client to look up 2D sprite/model
}

impl ShipTypeDefinition {
    pub fn get_world_corners_at_position(&self, position: &Vec2, angle: f32) -> [Vec2; 4] {
        let half_width = self.sprite_width as f32 / 2.0;
        let half_height = self.sprite_height as f32 / 2.0;

        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // Corners relative to ship's center, assuming 0 orientation
        let corners_local = [
            (half_width, half_height),   // top_right
            (-half_width, half_height),  // top_left
            (-half_width, -half_height), // bottom_left
            (half_width, -half_height),  // bottom_right
        ];

        // Rotate and translate to world space
        corners_local.map(|(x, y)| {
            let rotated_x = x * cos_angle - y * sin_angle;
            let rotated_y = x * sin_angle + y * cos_angle;
            Vec2::new(position.x + rotated_x, position.y + rotated_y)
        })
    }
}

#[dsl(plural_name = ship_statuses, method(update = true))]
#[table(name = ship_status, public)]
/// The status of a ship agnostic of where it is physically.
pub struct ShipStatus {
    #[primary_key]
    #[use_wrapper(ShipId)]
    id: u64,

    #[index(btree)] // To easily find ships in a given sector
    #[use_wrapper(SectorId)]
    /// FK to Sector.id // Needs to be kept in sync with StellarObject.sector_id
    pub sector_id: u64,

    #[index(btree)]
    #[use_wrapper(crate::players::PlayerId)]
    /// FK to player.id // You should only be able to see your ship, or other ships in your sector.
    pub player_id: Identity,

    pub health: f32,
    pub shields: f32,
    pub energy: f32,

    pub weapon_cooldown_ms: u32, // Milliseconds remaining until weapons can fire again
    pub missile_cooldown_ms: u32, // Milliseconds remaining until missiles can fire again

    pub used_cargo_capacity: u16, // Needs to be manually maintained via ShipCargoItem
    pub max_cargo_capacity: u16,  // Needs to be manually maintained via ShipCargoItem

    pub ai_state: Option<CurrentAction>, // Current high-level AI state or player command
}

impl ShipStatus {
    pub fn get_remaining_cargo_space(&self) -> u16 {
        self.get_max_cargo_capacity() - self.get_used_cargo_capacity()
    }

    pub fn calculate_used_cargo_space<T: spacetimedsl::WriteContext>(&self, dsl: &DSL<T>) -> u16 {
        let mut used_cargo_space: u16 = 0;

        info!(
            "Calculating cargo space usage for ship #{}. (Max cargo {}v)",
            self.id, self.max_cargo_capacity
        );

        // Collect all the ship items and calculate their volume usage
        for item in dsl.get_ship_cargo_items_by_ship_id(&self.get_id()) {
            if let Ok(item_def) = dsl.get_item_definition_by_id(&item.get_item_id()) {
                let volume_usage: u16 = item.quantity * item_def.get_volume_per_unit();
                info!(
                    "     Stack of {}x {}: {} volume used",
                    item.quantity,
                    item_def.get_name(),
                    volume_usage
                );
                used_cargo_space += volume_usage;
            }
        }
        info!(
            "Total cargo space usage for ship #{}: {}",
            self.id, used_cargo_space
        );

        used_cargo_space
    }
}

#[dsl(plural_name = ship_movement_controllers, method(update = true))]
#[table(name = ship_movement_controller, public)]
pub struct ShipMovementController {
    #[primary_key]
    #[use_wrapper(PlayerId)]
    #[foreign_key(path = crate::tables::players, table = player, column = id, on_delete = Delete)]
    id: Identity,

    #[index(btree)]
    #[use_wrapper(StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    stellar_object_id: u64,

    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
}

#[dsl(plural_name = ships, method(update = true))]
#[table(name = ship, public)]
pub struct Ship {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::tables::ships, table = ship_cargo_item)]
    #[referenced_by(path = crate::tables::ships, table = ship_equipment_slot)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(ShipTypeDefinitionId)]
    #[foreign_key(path = crate::tables::ships, table = ship_type_definition, column = id, on_delete = Error)]
    /// FK to ShipTypeDefinition.id
    pub shiptype_id: u32,

    /// Where is the ship currently located? Is it docked or currently flying?
    #[index(btree)]
    pub location: ShipLocation,

    #[index(btree)] // Can't be unique anymore because docked ships now have a sobj_id of 0
    #[use_wrapper(StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Ignore)]
    pub sobj_id: u64,

    #[index(btree)]
    #[use_wrapper(StationId)]
    #[foreign_key(path = crate::tables::stations, table = station, column = id, on_delete = SetZero)]
    /// TODO - STDSL doesn't allow this to be `pub station_id: Option<u64>,` due to STDB not allowing optional indexes.
    /// Therefore we'll use 0 as the sentinel value for None.
    pub station_id: u64,

    #[index(btree)]
    #[use_wrapper(crate::tables::sectors::SectorId)]
    #[foreign_key(path = crate::tables::sectors, table = sector, column = id, on_delete = Error)]
    /// Only because actually referencing the player's stellar object would require three table hits.
    pub sector_id: u64,

    #[index(btree)]
    #[use_wrapper(crate::players::PlayerId)]
    #[foreign_key(path = crate::players, table = player, column = id, on_delete = Error)]
    pub player_id: Identity,

    #[index(btree)]
    #[use_wrapper(crate::tables::factions::FactionId)]
    #[foreign_key(path = crate::tables::factions, table = faction, column = id, on_delete = Error)]
    pub faction_id: u32,
}

#[dsl(plural_name = ship_cargo_items, method(update = true))]
#[table(name = ship_cargo_item, public)]
pub struct ShipCargoItem {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)] // To query all cargo for a specific ship
    #[use_wrapper(ShipId)]
    #[foreign_key(path = crate::tables::ships, table = ship, column = id, on_delete = Delete)]
    /// FK to Ship
    pub ship_id: u64,

    #[index(btree)]
    #[use_wrapper(crate::tables::items::ItemDefinitionId)]
    #[foreign_key(path = crate::tables::items, table = item_definition, column = id, on_delete = Error)]
    /// FK to ItemDefinition
    pub item_id: u32,

    pub quantity: u16, // How many of this item are currently in this stack
                       //pub stack_size: u8, // TODO: Do we keep this value here to save query time?
}

#[dsl(plural_name = ship_equipment_slots, method(update = true))]
#[table(name = ship_equipment_slot, public)]
pub struct ShipEquipmentSlot {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)] // To query all equipment for a specific ship
    #[use_wrapper(ShipId)]
    #[foreign_key(path = crate::tables::ships, table = ship, column = id, on_delete = Delete)]
    /// FK to Ship
    pub ship_id: u64,

    pub slot_type: EquipmentSlotType,
    pub slot_index: u8, // E.g., Weapon Slot 0, Weapon Slot 1 within its type

    #[index(btree)]
    #[use_wrapper(ItemDefinitionId)]
    #[foreign_key(path = crate::tables::items, table = item_definition, column = id, on_delete = Error)]
    /// FK to ItemDefinition
    pub item_id: u32,
}
