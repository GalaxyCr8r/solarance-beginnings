use spacetimedb::{ table, ReducerContext, SpacetimeType, Timestamp };
use spacetimedsl::dsl;

use crate::types::ships::*;

pub mod definitions; // Definitions for initial ingested data.
pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[derive(SpacetimeType, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResourceCategory {
    RawOre,
    RefinedIngot,
    StoredEnergy,
    ManufacturedComponentBasic,
    ManufacturedComponentAdvanced,
    BiomatterRaw,
    BiomatterProcessedFood, // Basic food
    BiomatterProcessedLuxury, // Luxury food
    ConsumableShipAmmo,
    ConsumableShipFuel,
    ExoticMatter, // For high-tier research/construction
    ResearchDataFragments, // Gathered from anomalies/ruins
    FinishedGoods, // For trade, NPC requests
}

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq, Hash)]
pub enum OreType {
    NickelIron,
    Silicon,
    Ice,
    Platinum,
    Tungsten,
    Carbon,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShipModuleType {
    Engine,
    ShieldGenerator,
    WeaponKinetic,
    WeaponEnergy,
    WeaponMissile,
    MiningLaserBasic,
    MiningLaserAdvanced,
    CargoExpander,
    ScannerBasic,
    ScannerAdvanced,
    TractorBeam,
    CloakingDevice,
    RepairSystem,
    WarpDrive,
    JumpDrive, // For inter-system travel
}

// Enum for different categories of items
#[derive(SpacetimeType, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ItemCategory {
    ShipModule(ShipModuleType),
    Resource(ResourceCategory),
}

/// Enum for different effects for items/modules
#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum ItemMetadata {
    /// Effects damage output
    DamageBoost(f32),
    /// Effects shield output
    ShieldBoost(f32),
    /// Adds additional cargo capacity
    CargoCapacityBoost(u16),
    /// From 0.001 to 10.0
    MiningSpeedMultiplier(f32),
    /// How much energy this item consumes per second.
    EnergyConsumption(f32),
    /// Some other special effect
    SpecialEffect(String),

    /// How many of this item can exist in a single stack
    Stacks(u8),
    /// This item cannot be stacked in ship cargo
    NoStacking,
    /// This item cannot be traded
    NoTrade,
    /// This item cannot be sold
    NoSell,
    /// Cannot be dropped from inventory
    NoDrop,
    /// Quality of the item, 0-100
    Quality(u8),
}

#[dsl(plural_name = item_definitions)]
#[table(name = item_definition, public)]
pub struct ItemDefinition {
    #[primary_key]
    #[create_wrapper]
    id: u32,

    pub name: String, // E.g., "Iron Ore", "Laser Cannon Mk2", "Energy Cells"
    pub description: Option<String>,

    pub category: ItemCategory,

    pub base_value: u32, // Base monetary value
    pub volume_per_unit: u16, // How much cargo space one unit takes
    pub units_per_stack: u8, // How units can be stacked in cargo slot
    // For equipment, additional stats might be here or in a linked table:
    // E.g., damage: Option<u32>, shield_boost: Option<u32>, etc.
    pub metadata: Vec<ItemMetadata>,

    pub gfx_key: Option<String>, // For items that have a visual representation
}

#[dsl(plural_name = cargo_crates)]
#[table(name = cargo_crate, public)]
pub struct CargoCrate {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[use_wrapper(path = crate::types::sectors::SectorId)]
    #[index(btree)] // To find crates in a specific sector
    /// FK to Sector.id
    pub current_sector_id: u64,

    #[use_wrapper(path = crate::types::stellarobjects::StellarObjectId)]
    #[unique]
    /// FK to StellarObject
    pub sobj_id: u64,

    #[use_wrapper(path = ItemDefinitionId)]
    /// FK to ItemDefinition
    pub item_id: u32,
    pub quantity: u16,

    pub despawn_ts: Option<Timestamp>, // When the crate should disappear if not collected

    pub gfx_key: Option<String>,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    definitions::init(ctx)?;

    Ok(())
}
