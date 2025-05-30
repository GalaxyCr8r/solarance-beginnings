use spacetimedb::{table, ReducerContext, SpacetimeType, Timestamp};
use spacetimedsl::dsl;

pub mod definitions;
pub mod utility;

// Enum for different categories of items
#[derive(SpacetimeType, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ItemCategory {
    ShipEquipment,
    Commodity, // Tradable goods like ore, food
    ManufacturedGood, // Components, advanced materials
    Ammunition,
    Special, // Quest items, blueprints, etc.
}

#[dsl(plural_name = item_definitions)]
#[table(name = item_definition, public)]
pub struct ItemDefinition {
    #[primary_key]
    #[wrap]
    pub id: u32,

    pub name: String, // E.g., "Iron Ore", "Laser Cannon Mk2", "Energy Cells"
    pub description: Option<String>,

    pub category: ItemCategory,

    pub base_value: u32, // Base monetary value
    pub volume_per_unit: u16, // How much cargo space one unit takes
    // For equipment, additional stats might be here or in a linked table:
    // E.g., damage: Option<u32>, shield_boost: Option<u32>, etc.

    pub gfx_key: Option<String>, // For items that have a visual representation
}

#[dsl(plural_name = cargo_crates)]
#[table(name = cargo_crate, public)]
pub struct CargoCrate {
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    #[wrapped(path = crate::types::sectors::SectorId)]
    #[index(btree)] // To find crates in a specific sector
    pub current_sector_id: u64, // FK: Sector.id

    #[wrapped(path = crate::types::stellarobjects::StellarObjectId)]
    #[unique]
    pub sobj_id: u64, // FK: StellarObject

    #[wrapped(path = ItemDefinitionId)]
    pub item_id: u32, // FK: ItemDefinition
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