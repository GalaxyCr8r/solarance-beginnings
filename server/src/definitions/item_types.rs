use std::f32::consts::PI;

use log::info;
use spacetimedsl::*;

use crate::tables::{combat::*, items::*};

// # Ship modules
// ## Ship Engines

// ## Ship Shields

// ## Mining Lasers
/// Basic cheap mining laser produced by everyone.
pub const SMOD_BASIC_MINING_LASER: u32 = 17_000;

// ## Ship Weapons
/// Basic kinetic weapon firing solid projectiles at high velocity.
pub const SMOD_AUTOCANNON: u32 = 18_000;
/// Energy weapon that fires charged ionic particles to disrupt shields.
pub const SMOD_IONIC_BLASTER: u32 = 18_010;
/// Advanced energy weapon firing superheated plasma bolts.
pub const SMOD_PLASMA_CANNON: u32 = 18_020;

// # Technical items
/// A piece of research data usually collected from a laboratory station module.
pub const ITEM_RESEARCH_FRAGMENT: u32 = 10_000;
/// A piece of research data rarely collected - usually from a laboratory module near a space anonomly.
pub const ITEM_RESEARCH_FRAGMENT_RARE: u32 = 10_001;
/// A piece of exotic research data collected from an alien race or itself is an active anonomly.
pub const ITEM_RESEARCH_FRAGMENT_EXOTIC: u32 = 10_002;
// # Research Components
/// Needed to build lab station modules, it's built out of computers and viveium crystals.
pub const ITEM_RESEARCH_DEVICE: u32 = 10_100;
/// A specialized research device made to process exotic research fragments and produce rare fragements.
/// Much more expensive/resource-intesive than a standard research device.
pub const ITEM_RESEARCH_DEVICE_RARE: u32 = 10_101;

// # Fuel/Energy Items
/// A cheap battery for holding energy. Can be stored for hundreds of years.
pub const ITEM_ENERGY_CELL: u32 = 0_000;
pub const ITEM_COMPRESSED_HYDROGEN: u32 = 0_010;
pub const ITEM_JUMPDRIVE_FUEL: u32 = 0_050;

// # Raw Materials
// ## Ore
pub const ITEM_ICE_ORE: u32 = 2_000;
pub const ITEM_CARBON_ORE: u32 = 2_001;
pub const ITEM_IRON_ORE: u32 = 2_002;
pub const ITEM_SILICON_ORE: u32 = 2_003;
pub const ITEM_URANIUM_ORE: u32 = 2_004;
/// Made up mineral for this game. Rare. When processed produces an ingot and sometimes an even rarer "viveium crystal" used for advanced parts/research devices.
pub const ITEM_VIVEIUM_ORE: u32 = 2_005;
pub const ITEM_TITANIUM_ORE: u32 = 2_006;
pub const ITEM_GOLD_ORE: u32 = 2_007;

// ## Processed Ore
pub const ITEM_CARBON_RAW: u32 = 2_101;
pub const ITEM_IRON_INGOT: u32 = 2_102;
pub const ITEM_SILICON_RAW: u32 = 2_103;
pub const ITEM_URANIUM_INGOT: u32 = 2_104;
pub const ITEM_URANIUM_ENRICHED: u32 = 2_114;
pub const ITEM_VIVEIUM_INGOT: u32 = 2_105;
pub const ITEM_TITANIUM_INGOT: u32 = 2_106;
pub const ITEM_GOLD_INGOT: u32 = 2_107;
// ### Speciality Processed
pub const ITEM_VIVEIUM_CRYSTAL: u32 = 2_206; // A secondary "waste" byproduct of Viveium ingot refinement. Rare.

// ## Gas
pub const ITEM_HELIUM_GAS: u32 = 2_503;
pub const ITEM_HYDROGEN_GAS: u32 = 2_504;

// ## Liquids
pub const ITEM_WATER: u32 = 2_200;

// # Biomatter
// ## Raw/Waste Biomatter
/// Hazardous/useless biomatter.
pub const ITEM_BIOMATTER_RAW_HAZARD: u32 = 3_000;
/// Discarded plant matter.
pub const ITEM_BIOMATTER_RAW_PLANT: u32 = 3_001;
/// Animal refuse that could be processed further.
pub const ITEM_BIOMATTER_RAW_ANIMAL: u32 = 3_002;
/// Biologically-available minerals/vitamins.
pub const ITEM_BIOMATTER_RAW_MINERALS: u32 = 3_003;
/// Item dropped by biologic alien spacecraft. Needs to be processed into
/// ITEM_BIOMATTER_ANIMAL/ITEM_BIOMATTER_RAW_MINERALS while generating plenty of ITEM_BIOMATTER_RAW_HAZARD waste
/// (Special modules might take it and rarely produce ITEM_RESEARCH_FRAGMENT_EXOTIC/ITEM_BIOMATTER_RAW_HAZARD from it)
pub const ITEM_BIOMATTER_MALIGNANT: u32 = 3_004;

// ## Processed Biomatter
/// Created via raw plant matter and raw minerals. Used to grow more food.
pub const ITEM_BIOMATTER_COMPOST: u32 = 3_100;
// ### Generic foodstuffs
pub const ITEM_FOOD_RATIONS: u32 = 3_201;
pub const ITEM_FOOD_AVERAGE: u32 = 3_202;
pub const ITEM_FOOD_LUXURY: u32 = 3_203;
// ### Specific Foodstuffs (Potentially for luxury npc needs?)
pub const ITEM_ALCOHOL: u32 = 3_502;

// # Component Construction
// ## Computer-related
/// Processed silicon into wafers
pub const ITEM_COMPUTER_WAFERS: u32 = 4_000;
/// Wafers processed into functional chips.
pub const ITEM_COMPUTER_CHIPS: u32 = 4_001;
/// Wafer processed into CPU cores.
pub const ITEM_COMPUTER_CORE: u32 = 4_002;
/// Chips and cores processed into a general purpose computer board.
pub const ITEM_COMPUTER_BOARD: u32 = 4_003;

// ## Armor/Structural
/// Metal plates for normal components. Flavor-wise: used for floorboards, light armor, component casings, etc.
pub const ITEM_METAL_PLATES: u32 = 4_300;
/// Metal connecting devices. Only a few are needed, but almost every module needs at least one.
pub const ITEM_METAL_LINKAGES: u32 = 4_301;
/// Metal plates made specifically for armoring ships and the outside of stations.
pub const ITEM_METAL_ARMOR_PLATES: u32 = 4_302;
/// Metal plates made for combat spacecraft.
pub const ITEM_METAL_HARDENED_ARMOR_PLATES: u32 = 4_303;
/// Metal plates and beams welded together to make a section of structure or a capital-classed spacecraft or station.
pub const ITEM_METAL_HULL_STRUCTURE: u32 = 4_310;

// ## Ship/Station module component parts
/// A generic component part. Needed by almost any ship or station module. Requires a few computer boards, hull structure, plates, etc.
pub const ITEM_MODULE_COMPONENTS: u32 = 4_100;
/// A advanced component required for more complex components. Requires a few viveium crystals plus more computer boards than normal components.
pub const ITEM_MODULE_COMPONENTS_ADVANCED: u32 = 4_101;
/// A generic small spacecraft-sized component
pub const ITEM_MODULE_COMPONENTS_SHIP: u32 = 4_102;
/// A generic large spacecraft-sized component
pub const ITEM_MODULE_COMPONENTS_CAPITAL: u32 = 4_103;
/// A generic station-sized component
pub const ITEM_MODULE_COMPONENTS_STATION: u32 = 4_104;
// ### Specialized components
pub const ITEM_MODULE_COMPONENTS_WEAPON: u32 = 4_200;
pub const ITEM_MODULE_COMPONENTS_ENGINE: u32 = 4_201;
pub const ITEM_MODULE_COMPONENTS_SHIELD: u32 = 4_202;
pub const ITEM_MODULE_COMPONENTS_MISSILE: u32 = 4_203;

// Stack Sizes
/// Small items like food rations, handheld devices, research fragments, etc.
pub const COMPACT_STACK_SIZE: u8 = 64;
/// Medium sized items like ingots, large components/computers, etc.
pub const LOOSE_STACK_SIZE: u8 = 32;
/// Large sized items that are bulky or need special containment. Ore, ship plating, exotic materials, etc.
pub const LARGE_STACK_SIZE: u8 = 16;
/// Massive items like finished ship/station components, hull sections, research devices, etc.
pub const MASSIVE_STACK_SIZE: u8 = 4;

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    commodity_definitions(dsl)?;

    info!("Item Defs Loaded: {}", dsl.count_of_all_item_definitions());
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn commodity_definitions<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    energy_definitions(dsl)?;
    ore_definitions(dsl)?;
    ingot_definitions(dsl)?;
    biomatter_definitions(dsl)?;
    food_definitions(dsl)?;
    ship_module_definitions(dsl)?;
    Ok(())
}

fn energy_definitions<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    let current_category = ItemCategory::Resource(ResourceCategory::StoredEnergy);

    // Energy Cells
    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_ENERGY_CELL,
        name: "Energy Cell".to_string(),
        description: Some(
            "Energy Cells are the unified energy storage used throughout the whole known universe. 
            The unification of the energy storage specifications allows for interstellar trading with this most basic of all products.".into()
        ),
        category: current_category.clone(),
        base_value: 20,
        margin_percentage: 52,
        volume_per_unit: 1,
        units_per_stack: COMPACT_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    // Compressed Hydrogen
    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_COMPRESSED_HYDROGEN,
        name: "Compressed Hydrogen".to_string(),
        description: Some(
            "Compressed hydrogen fuel used for basic ship propulsion systems.".into(),
        ),
        category: ItemCategory::Resource(ResourceCategory::ConsumableShipFuel),
        base_value: 15,
        margin_percentage: 75,
        volume_per_unit: 2,
        units_per_stack: COMPACT_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    // Jump Drive Fuel
    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_JUMPDRIVE_FUEL,
        name: "Jump Drive Fuel".to_string(),
        description: Some(
            "Exotic fuel required for faster-than-light travel between star systems.".into(),
        ),
        category: ItemCategory::Resource(ResourceCategory::ConsumableShipFuel),
        base_value: 500,
        margin_percentage: 33,
        volume_per_unit: 4,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    Ok(())
}

fn ore_definitions<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    let current_category = ItemCategory::Resource(ResourceCategory::RawOre);

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_ICE_ORE,
        name: "Ice Ore".to_string(),
        description: Some(
            "Raw ice mined from an asteroid. Needs to be melted and filtered.".into(),
        ),
        category: current_category.clone(),
        base_value: 50,
        margin_percentage: 15,
        volume_per_unit: 8,
        units_per_stack: LARGE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_IRON_ORE,
        name: "Iron Ore".to_string(),
        description: Some(
            "Raw ore from a variety of sources. It's a common mineral that all metal components need.".into()
        ),
        category: current_category.clone(),
        base_value: 100,
        margin_percentage: 22,
        volume_per_unit: 8,
        units_per_stack: LARGE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_SILICON_ORE,
        name: "Silicon Ore".to_string(),
        description: Some(
            "Silicon ore to be processed. Used to create microchips and other advanced goods."
                .into(),
        ),
        category: current_category.clone(),
        base_value: 100,
        margin_percentage: 33,
        volume_per_unit: 8,
        units_per_stack: LARGE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_CARBON_ORE,
        name: "Carbon Ore".to_string(),
        description: Some(
            "Raw carbon ore from asteroid mining. Essential for many industrial processes.".into(),
        ),
        category: current_category.clone(),
        base_value: 80,
        margin_percentage: 25,
        volume_per_unit: 8,
        units_per_stack: LARGE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_URANIUM_ORE,
        name: "Uranium Ore".to_string(),
        description: Some(
            "Radioactive uranium ore. Handle with care. Used for energy production and weapons."
                .into(),
        ),
        category: current_category.clone(),
        base_value: 350,
        margin_percentage: 16,
        volume_per_unit: 8,
        units_per_stack: LARGE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_VIVEIUM_ORE,
        name: "Viveium Ore".to_string(),
        description: Some(
            "Rare viveium ore with unique properties. Essential for advanced research and technology.".into()
        ),
        category: current_category.clone(),
        base_value: 1000,
        margin_percentage: 5,
        volume_per_unit: 8,
        units_per_stack: LARGE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_TITANIUM_ORE,
        name: "Titanium Ore".to_string(),
        description: Some(
            "Strong and lightweight titanium ore. Preferred for high-performance ship components."
                .into(),
        ),
        category: current_category.clone(),
        base_value: 250,
        margin_percentage: 42,
        volume_per_unit: 8,
        units_per_stack: LARGE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_GOLD_ORE,
        name: "Gold Ore".to_string(),
        description: Some(
            "Precious gold ore. Valuable for trade and specialized electronics.".into(),
        ),
        category: current_category.clone(),
        base_value: 400,
        margin_percentage: 25,
        volume_per_unit: 8,
        units_per_stack: LARGE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    Ok(())
}

fn ingot_definitions<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    let current_category = ItemCategory::Resource(ResourceCategory::RefinedIngot);

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_IRON_INGOT,
        name: "Iron Ingot".to_string(),
        description: Some("Refined iron ingot. Used in many ship components.".into()),
        category: current_category.clone(),
        base_value: 150,
        margin_percentage: 33,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_SILICON_RAW,
        name: "Silicon Ingot".to_string(),
        description: Some("Refined silicon ingot. Used in many computer components.".into()),
        category: current_category.clone(),
        base_value: 150,
        margin_percentage: 33,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_URANIUM_INGOT,
        name: "Uranium Ingot".to_string(),
        description: Some(
            "Refined uranium ingot. Used in many reactor and weapon components.".into(),
        ),
        category: current_category.clone(),
        base_value: 200,
        margin_percentage: 22,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_VIVEIUM_INGOT,
        name: "Viveium Ingot".to_string(),
        description: Some("Refined viveium ingot. Used in many research components.".into()),
        category: current_category.clone(),
        base_value: 1250,
        margin_percentage: 33,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_TITANIUM_INGOT,
        name: "Titanium Ingot".to_string(),
        description: Some("Refined titanium ingot. Used in many advanced ship components.".into()),
        category: current_category.clone(),
        base_value: 300,
        margin_percentage: 15,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_GOLD_INGOT,
        name: "Gold Ingot".to_string(),
        description: Some(
            "Refined gold ingot. Used in luxury goods and specialized electronics.".into(),
        ),
        category: current_category.clone(),
        base_value: 500,
        margin_percentage: 25,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_CARBON_RAW,
        name: "Carbon Ingot".to_string(),
        description: Some(
            "Refined carbon ingot. Essential for advanced manufacturing processes.".into(),
        ),
        category: current_category.clone(),
        base_value: 120,
        margin_percentage: 12,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_URANIUM_ENRICHED,
        name: "Enriched Uranium".to_string(),
        description: Some(
            "Highly enriched uranium for advanced reactor cores and weapons systems.".into(),
        ),
        category: current_category.clone(),
        base_value: 800,
        margin_percentage: 29,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_VIVEIUM_CRYSTAL,
        name: "Viveium Crystal".to_string(),
        description: Some(
            "Rare crystal byproduct of viveium refinement. Used in advanced research devices."
                .into(),
        ),
        category: ItemCategory::Resource(ResourceCategory::ExoticMatter),
        base_value: 2500,
        margin_percentage: 52,
        volume_per_unit: 4,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    Ok(())
}

fn biomatter_definitions<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    let current_category = ItemCategory::Resource(ResourceCategory::BiomatterRaw);

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_WATER,
        name: "Water Barrel".to_string(),
        description: Some("A barrel of clean drinkable water.".into()),
        category: current_category.clone(),
        base_value: 75,
        margin_percentage: 60,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_BIOMATTER_RAW_ANIMAL,
        name: "Biomatter Waste".to_string(),
        description: Some(
            "Decomposing plant/food waste, recycling organic materials, or manure.".into(),
        ),
        category: current_category.clone(),
        base_value: 105,
        margin_percentage: 80,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_BIOMATTER_COMPOST,
        name: "Compost".to_string(),
        description: Some(
            "Compost is a mixture of ingredients used as plant fertilizer and to improve soil's physical, chemical, and biological properties. It is commonly prepared by decomposing plant and food waste, recycling organic materials, and manure.".into()
        ),
        category: current_category.clone(),
        base_value: 275,
        margin_percentage: 52,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_BIOMATTER_RAW_HAZARD,
        name: "Hazardous Biomatter".to_string(),
        description: Some(
            "Dangerous biological waste that requires special handling and disposal.".into(),
        ),
        category: current_category.clone(),
        base_value: 25,
        margin_percentage: 90,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_BIOMATTER_RAW_PLANT,
        name: "Plant Biomatter".to_string(),
        description: Some(
            "Discarded plant matter suitable for processing into useful materials.".into(),
        ),
        category: current_category.clone(),
        base_value: 50,
        margin_percentage: 52,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_BIOMATTER_RAW_MINERALS,
        name: "Mineral Biomatter".to_string(),
        description: Some(
            "Biologically-available minerals and vitamins extracted from organic sources.".into(),
        ),
        category: current_category.clone(),
        base_value: 120,
        margin_percentage: 52,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_BIOMATTER_MALIGNANT,
        name: "Malignant Biomatter".to_string(),
        description: Some(
            "Alien biological matter of unknown origin. Requires extreme caution when handling."
                .into(),
        ),
        category: current_category.clone(),
        base_value: 200,
        margin_percentage: 75,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    // Gas items
    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_HELIUM_GAS,
        name: "Helium Gas".to_string(),
        description: Some("Compressed helium gas used in various industrial applications.".into()),
        category: current_category.clone(),
        base_value: 40,
        margin_percentage: 75,
        volume_per_unit: 4,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_HYDROGEN_GAS,
        name: "Hydrogen Gas".to_string(),
        description: Some("Compressed hydrogen gas used for fuel and industrial processes.".into()),
        category: current_category.clone(),
        base_value: 30,
        margin_percentage: 52,
        volume_per_unit: 4,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    Ok(())
}

fn food_definitions<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    let mut current_category = ItemCategory::Resource(ResourceCategory::BiomatterProcessedFood);

    // Food Rations
    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_FOOD_RATIONS,
        name: "Food Rations".to_string(),
        description: Some(
            "A basic food ration. Contains all the nutrients needed to survive for a few days."
                .into(),
        ),
        category: current_category.clone(),
        base_value: 100,
        margin_percentage: 52,
        volume_per_unit: 1,
        units_per_stack: COMPACT_STACK_SIZE,
        metadata: vec![],
        gfx_key: None, //vec![FoodQuality(0)],
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_FOOD_AVERAGE,
        name: "Average Food".to_string(),
        description: Some(
            "Everyday foodstuffs. Contains all the nutrients needed to survive for a few days."
                .into(),
        ),
        category: current_category.clone(),
        base_value: 250,
        margin_percentage: 25,
        volume_per_unit: 1,
        units_per_stack: COMPACT_STACK_SIZE,
        metadata: vec![],
        gfx_key: None, //vec![FoodQuality(0)],
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_FOOD_LUXURY,
        name: "Luxury Food".to_string(),
        description: Some("Super deluxe foodstuffs. Not necessary for anyone to survive.".into()),
        category: current_category.clone(),
        base_value: 2000,
        margin_percentage: 75,
        volume_per_unit: 1,
        units_per_stack: COMPACT_STACK_SIZE,
        metadata: vec![],
        gfx_key: None, //vec![FoodQuality(0)],
    })?;

    ///////////////////////////////////////////////////////////////////////////////////////
    // BiomatterProcessedLuxury
    current_category = ItemCategory::Resource(ResourceCategory::BiomatterProcessedLuxury);

    // Space "Fuel"
    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_ALCOHOL,
        name: "Space 'Fuel'".to_string(),
        description: Some(
            "Actually a variant of vodka using common supplies found in spaceships to make. It's usually illegal due to its propensity to turn the users blind.".into()
        ),
        category: current_category.clone(),
        base_value: 50,
        margin_percentage: 33,
        volume_per_unit: 1,
        units_per_stack: COMPACT_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    Ok(())
}

fn ship_module_definitions<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    use ItemMetadata::*;

    dsl.create_item_definition(CreateItemDefinition {
        id: SMOD_BASIC_MINING_LASER,
        name: "Basic Mining Laser".to_string(),
        description: Some("Manufactured by many, functionally the same.".into()),
        category: ItemCategory::ShipModule(ShipModuleType::MiningLaserBasic),
        base_value: 500,
        margin_percentage: 52,
        volume_per_unit: 4,
        units_per_stack: LARGE_STACK_SIZE,
        metadata: vec![MiningSpeedMultiplier(1.0), EnergyConsumption(1.75)],
        gfx_key: None,
    })?;

    // Ship Weapons
    dsl.create_item_definition(CreateItemDefinition {
        id: SMOD_AUTOCANNON,
        name: "Autocannon".to_string(),
        description: Some("A reliable kinetic weapon that fires solid projectiles at high velocity. Effective against armor but less so against shields.".into()),
        category: ItemCategory::ShipModule(ShipModuleType::WeaponKinetic),
        base_value: 750,
        margin_percentage: 42,
        volume_per_unit: 4,
        units_per_stack: LARGE_STACK_SIZE,
        metadata: vec![Weapon(WeaponType::Hitscan), BaseDamage(1.0), ShieldDamageMod(0.4), KineticDamageMod(1.2), EnergyConsumption(0.5), LockOnAngleBoundRads(PI/48.0), CooldownMs(500)],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: SMOD_IONIC_BLASTER,
        name: "Ionic Blaster".to_string(),
        description: Some("An energy weapon that fires charged ionic particles. Highly effective against shields but reduced damage to armor.".into()),
        category: ItemCategory::ShipModule(ShipModuleType::WeaponEnergy),
        base_value: 900,
        margin_percentage: 35,
        volume_per_unit: 4,
        units_per_stack: LARGE_STACK_SIZE,
        metadata: vec![Weapon(WeaponType::Hitscan), BaseDamage(0.8), ShieldDamageMod(1.5), KineticDamageMod(0.5), EnergyConsumption(2.0), LockOnAngleBoundRads(PI/32.0), CooldownMs(250)],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: SMOD_PLASMA_CANNON,
        name: "Plasma Cannon".to_string(),
        description: Some("An advanced energy weapon that fires superheated plasma bolts. Balanced damage against both shields and armor but high energy consumption.".into()),
        category: ItemCategory::ShipModule(ShipModuleType::WeaponEnergy),
        base_value: 1900,
        margin_percentage: 28,
        volume_per_unit: 4,
        units_per_stack: LARGE_STACK_SIZE,
        metadata: vec![Weapon(WeaponType::Hitscan), BaseDamage(2.3), ShieldDamageMod(0.9), KineticDamageMod(1.0), EnergyConsumption(3.0), LockOnAngleBoundRads(PI/42.0), CooldownMs(750)],
        gfx_key: None,
    })?;

    // Research Items
    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_RESEARCH_FRAGMENT,
        name: "Research Fragment".to_string(),
        description: Some(
            "A piece of research data usually collected from a laboratory station module.".into(),
        ),
        category: ItemCategory::Resource(ResourceCategory::ResearchDataFragments),
        base_value: 150,
        margin_percentage: 52,
        volume_per_unit: 1,
        units_per_stack: COMPACT_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_RESEARCH_FRAGMENT_RARE,
        name: "Rare Research Fragment".to_string(),
        description: Some(
            "A piece of research data rarely collected - usually from a laboratory module near a space anomaly.".into()
        ),
        category: ItemCategory::Resource(ResourceCategory::ResearchDataFragments),
        base_value: 800,
        margin_percentage: 52,
        volume_per_unit: 1,
        units_per_stack: COMPACT_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_RESEARCH_FRAGMENT_EXOTIC,
        name: "Exotic Research Fragment".to_string(),
        description: Some(
            "A piece of exotic research data collected from an alien race or itself is an active anomaly.".into()
        ),
        category: ItemCategory::Resource(ResourceCategory::ExoticMatter),
        base_value: 5000,
        margin_percentage: 25,
        volume_per_unit: 1,
        units_per_stack: COMPACT_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_RESEARCH_DEVICE,
        name: "Research Device".to_string(),
        description: Some(
            "Needed to build lab station modules, it's built out of computers and viveium crystals.".into()
        ),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentAdvanced),
        base_value: 2500,
        margin_percentage: 13,
        volume_per_unit: 8,
        units_per_stack: MASSIVE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_RESEARCH_DEVICE_RARE,
        name: "Rare Research Device".to_string(),
        description: Some(
            "A specialized research device made to process exotic research fragments and produce rare fragments. Much more expensive/resource-intensive than a standard research device.".into()
        ),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentAdvanced),
        base_value: 10000,
        margin_percentage: 13,
        volume_per_unit: 8,
        units_per_stack: MASSIVE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    // Computer Components
    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_COMPUTER_WAFERS,
        name: "Computer Wafers".to_string(),
        description: Some("Processed silicon into wafers for computer manufacturing.".into()),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 200,
        margin_percentage: 52,
        volume_per_unit: 4,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_COMPUTER_CHIPS,
        name: "Computer Chips".to_string(),
        description: Some("Wafers processed into functional chips for electronics.".into()),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 400,
        margin_percentage: 24,
        volume_per_unit: 4,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_COMPUTER_CORE,
        name: "Computer Core".to_string(),
        description: Some("Wafer processed into CPU cores for advanced computers.".into()),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 800,
        margin_percentage: 33,
        volume_per_unit: 4,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_COMPUTER_BOARD,
        name: "Computer Board".to_string(),
        description: Some(
            "Chips and cores processed into a general purpose computer board.".into(),
        ),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 1200,
        margin_percentage: 15,
        volume_per_unit: 4,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    // Metal Components
    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_METAL_PLATES,
        name: "Metal Plates".to_string(),
        description: Some(
            "Metal plates for normal components. Used for floorboards, light armor, component casings, etc.".into()
        ),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 150,
        margin_percentage: 25,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_METAL_LINKAGES,
        name: "Metal Linkages".to_string(),
        description: Some(
            "Metal connecting devices. Only a few are needed, but almost every module needs at least one.".into()
        ),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 300,
        margin_percentage: 33,
        volume_per_unit: 4,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_METAL_ARMOR_PLATES,
        name: "Metal Armor Plates".to_string(),
        description: Some(
            "Metal plates made specifically for armoring ships and the outside of stations.".into(),
        ),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 500,
        margin_percentage: 52,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_METAL_HARDENED_ARMOR_PLATES,
        name: "Hardened Armor Plates".to_string(),
        description: Some(
            "Metal plates made for combat spacecraft with enhanced protection.".into(),
        ),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 1000,
        margin_percentage: 11,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_METAL_HULL_STRUCTURE,
        name: "Hull Structure".to_string(),
        description: Some(
            "Metal plates and beams welded together to make a section of structure for a capital-class spacecraft or station.".into()
        ),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 2000,
        margin_percentage: 33,
        volume_per_unit: 16,
        units_per_stack: MASSIVE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    // Module Components
    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_MODULE_COMPONENTS,
        name: "Module Components".to_string(),
        description: Some(
            "A generic component part. Needed by almost any ship or station module. Requires a few computer boards, hull structure, plates, etc.".into()
        ),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 800,
        margin_percentage: 33,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_MODULE_COMPONENTS_ADVANCED,
        name: "Advanced Module Components".to_string(),
        description: Some(
            "An advanced component required for more complex components. Requires a few viveium crystals plus more computer boards than normal components.".into()
        ),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentAdvanced),
        base_value: 2500,
        margin_percentage: 22,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_MODULE_COMPONENTS_SHIP,
        name: "Ship Module Components".to_string(),
        description: Some("A generic small spacecraft-sized component.".into()),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 1000,
        margin_percentage: 13,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_MODULE_COMPONENTS_CAPITAL,
        name: "Capital Ship Components".to_string(),
        description: Some("A generic large spacecraft-sized component.".into()),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentAdvanced),
        base_value: 5000,
        margin_percentage: 9,
        volume_per_unit: 16,
        units_per_stack: MASSIVE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_MODULE_COMPONENTS_STATION,
        name: "Station Module Components".to_string(),
        description: Some("A generic station-sized component.".into()),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentAdvanced),
        base_value: 8000,
        margin_percentage: 13,
        volume_per_unit: 16,
        units_per_stack: MASSIVE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    // Specialized Components
    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_MODULE_COMPONENTS_WEAPON,
        name: "Weapon Components".to_string(),
        description: Some("Specialized components for weapon systems.".into()),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 1500,
        margin_percentage: 21,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_MODULE_COMPONENTS_ENGINE,
        name: "Engine Components".to_string(),
        description: Some("Specialized components for propulsion systems.".into()),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 1200,
        margin_percentage: 21,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_MODULE_COMPONENTS_SHIELD,
        name: "Shield Components".to_string(),
        description: Some("Specialized components for shield generator systems.".into()),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 1800,
        margin_percentage: 32,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    dsl.create_item_definition(CreateItemDefinition {
        id: ITEM_MODULE_COMPONENTS_MISSILE,
        name: "Missile Components".to_string(),
        description: Some("Specialized components for missile systems and launchers.".into()),
        category: ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        base_value: 2000,
        margin_percentage: 32,
        volume_per_unit: 8,
        units_per_stack: LOOSE_STACK_SIZE,
        metadata: vec![],
        gfx_key: None,
    })?;

    Ok(())
}
