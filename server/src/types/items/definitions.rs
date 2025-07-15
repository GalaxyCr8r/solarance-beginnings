use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::{ dsl, DSL };

use super::*;

// # Ship modules
// ## Ship Weapons

// ## Ship Engines

// ## Ship Shields

// ## Mining Lasers
/// Basic cheap mining laser produced by everyone.
pub const SMOD_BASIC_MINING_LASER: u32 = 17_000;

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

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    commodity_definitions(&dsl)?;

    info!("Item Defs Loaded: {}", dsl.count_of_all_item_definitions());
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn commodity_definitions(dsl: &DSL) -> Result<(), String> {
    energy_definitions(dsl)?;
    ore_definitions(dsl)?;
    ingot_definitions(dsl)?;
    biomatter_definitions(dsl)?;
    food_definitions(dsl)?;
    ship_module_definitions(dsl)?;
    Ok(())
}

fn energy_definitions(dsl: &DSL) -> Result<(), String> {
    let current_category = ItemCategory::Resource(ResourceCategory::StoredEnergy);

    // Energy Cells
    dsl.create_item_definition(
        ITEM_ENERGY_CELL,
        "Energy Cell",
        Some(
            "Energy Cells are the unified energy storage used throughout the whole known universe. 
            The unification of the energy storage specifications allows for interstellar trading with this most basic of all products.".into()
        ),
        current_category.clone(),
        20,
        52,
        1,
        COMPACT_STACK_SIZE,
        vec![],
        None
    )?;

    // Compressed Hydrogen
    dsl.create_item_definition(
        ITEM_COMPRESSED_HYDROGEN,
        "Compressed Hydrogen",
        Some("Compressed hydrogen fuel used for basic ship propulsion systems.".into()),
        ItemCategory::Resource(ResourceCategory::ConsumableShipFuel),
        15,
        75,
        2,
        COMPACT_STACK_SIZE,
        vec![],
        None
    )?;

    // Jump Drive Fuel
    dsl.create_item_definition(
        ITEM_JUMPDRIVE_FUEL,
        "Jump Drive Fuel",
        Some("Exotic fuel required for faster-than-light travel between star systems.".into()),
        ItemCategory::Resource(ResourceCategory::ConsumableShipFuel),
        500,
        33,
        4,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    Ok(())
}

fn ore_definitions(dsl: &DSL) -> Result<(), String> {
    let current_category = ItemCategory::Resource(ResourceCategory::RawOre);

    dsl.create_item_definition(
        ITEM_ICE_ORE,
        "Ice Ore",
        Some("Raw ice mined from an asteroid. Needs to be melted and filtered.".into()),
        current_category.clone(),
        50,
        15,
        8,
        LARGE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_IRON_ORE,
        "Iron Ore",
        Some(
            "Raw ore from a variety of sources. It's a common mineral that all metal components need.".into()
        ),
        current_category.clone(),
        100,
        22,
        8,
        LARGE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_SILICON_ORE,
        "Silicon Ore",
        Some(
            "Silicon ore to be processed. Used to create microchips and other advanced goods.".into()
        ),
        current_category.clone(),
        100,
        33,
        8,
        LARGE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_CARBON_ORE,
        "Carbon Ore",
        Some(
            "Raw carbon ore from asteroid mining. Essential for many industrial processes.".into()
        ),
        current_category.clone(),
        80,
        25,
        8,
        LARGE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_URANIUM_ORE,
        "Uranium Ore",
        Some(
            "Radioactive uranium ore. Handle with care. Used for energy production and weapons.".into()
        ),
        current_category.clone(),
        350,
        16,
        8,
        LARGE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_VIVEIUM_ORE,
        "Viveium Ore",
        Some(
            "Rare viveium ore with unique properties. Essential for advanced research and technology.".into()
        ),
        current_category.clone(),
        1000,
        5,
        8,
        LARGE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_TITANIUM_ORE,
        "Titanium Ore",
        Some(
            "Strong and lightweight titanium ore. Preferred for high-performance ship components.".into()
        ),
        current_category.clone(),
        250,
        42,
        8,
        LARGE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_GOLD_ORE,
        "Gold Ore",
        Some("Precious gold ore. Valuable for trade and specialized electronics.".into()),
        current_category.clone(),
        400,
        25,
        8,
        LARGE_STACK_SIZE,
        vec![],
        None
    )?;

    Ok(())
}

fn ingot_definitions(dsl: &DSL) -> Result<(), String> {
    let current_category = ItemCategory::Resource(ResourceCategory::RefinedIngot);

    dsl.create_item_definition(
        ITEM_IRON_INGOT,
        "Iron Ingot",
        Some("Refined iron ingot. Used in many ship components.".into()),
        current_category.clone(),
        150,
        33,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_SILICON_RAW,
        "Silicon Ingot",
        Some("Refined silicon ingot. Used in many computer components.".into()),
        current_category.clone(),
        150,
        33,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_URANIUM_INGOT,
        "Uranium Ingot",
        Some("Refined uranium ingot. Used in many reactor and weapon components.".into()),
        current_category.clone(),
        200,
        22,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_VIVEIUM_INGOT,
        "Viveium Ingot",
        Some("Refined viveium ingot. Used in many research components.".into()),
        current_category.clone(),
        1250,
        33,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_TITANIUM_INGOT,
        "Titanium Ingot",
        Some("Refined titanium ingot. Used in many advanced ship components.".into()),
        current_category.clone(),
        300,
        15,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_GOLD_INGOT,
        "Gold Ingot",
        Some("Refined gold ingot. Used in luxury goods and specialized electronics.".into()),
        current_category.clone(),
        500,
        25,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_CARBON_RAW,
        "Carbon Ingot",
        Some("Refined carbon ingot. Essential for advanced manufacturing processes.".into()),
        current_category.clone(),
        120,
        12,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_URANIUM_ENRICHED,
        "Enriched Uranium",
        Some("Highly enriched uranium for advanced reactor cores and weapons systems.".into()),
        current_category.clone(),
        800,
        29,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_VIVEIUM_CRYSTAL,
        "Viveium Crystal",
        Some(
            "Rare crystal byproduct of viveium refinement. Used in advanced research devices.".into()
        ),
        ItemCategory::Resource(ResourceCategory::ExoticMatter),
        2500,
        52,
        4,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    Ok(())
}

fn biomatter_definitions(dsl: &DSL) -> Result<(), String> {
    let current_category = ItemCategory::Resource(ResourceCategory::BiomatterRaw);

    dsl.create_item_definition(
        ITEM_WATER,
        "Water Barrel",
        Some("A barrel of clean drinkable water.".into()),
        current_category.clone(),
        75,
        60,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_BIOMATTER_RAW_ANIMAL,
        "Biomatter Waste",
        Some("Decomposing plant/food waste, recycling organic materials, or manure.".into()),
        current_category.clone(),
        105,
        80,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_BIOMATTER_COMPOST,
        "Compost",
        Some(
            "Compost is a mixture of ingredients used as plant fertilizer and to improve soil's physical, chemical, and biological properties. It is commonly prepared by decomposing plant and food waste, recycling organic materials, and manure.".into()
        ),
        current_category.clone(),
        275,
        52,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_BIOMATTER_RAW_HAZARD,
        "Hazardous Biomatter",
        Some("Dangerous biological waste that requires special handling and disposal.".into()),
        current_category.clone(),
        25,
        90,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_BIOMATTER_RAW_PLANT,
        "Plant Biomatter",
        Some("Discarded plant matter suitable for processing into useful materials.".into()),
        current_category.clone(),
        50,
        52,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_BIOMATTER_RAW_MINERALS,
        "Mineral Biomatter",
        Some("Biologically-available minerals and vitamins extracted from organic sources.".into()),
        current_category.clone(),
        120,
        52,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_BIOMATTER_MALIGNANT,
        "Malignant Biomatter",
        Some(
            "Alien biological matter of unknown origin. Requires extreme caution when handling.".into()
        ),
        current_category.clone(),
        200,
        75,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    // Gas items
    dsl.create_item_definition(
        ITEM_HELIUM_GAS,
        "Helium Gas",
        Some("Compressed helium gas used in various industrial applications.".into()),
        current_category.clone(),
        40,
        75,
        4,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_HYDROGEN_GAS,
        "Hydrogen Gas",
        Some("Compressed hydrogen gas used for fuel and industrial processes.".into()),
        current_category.clone(),
        30,
        52,
        4,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    Ok(())
}

fn food_definitions(dsl: &DSL) -> Result<(), String> {
    let mut current_category = ItemCategory::Resource(ResourceCategory::BiomatterProcessedFood);

    // Food Rations
    dsl.create_item_definition(
        ITEM_FOOD_RATIONS,
        "Food Rations",
        Some(
            "A basic food ration. Contains all the nutrients needed to survive for a few days.".into()
        ),
        current_category.clone(),
        100,
        52,
        1,
        COMPACT_STACK_SIZE,
        vec![], //vec![FoodQuality(0)],
        None
    )?;

    dsl.create_item_definition(
        ITEM_FOOD_AVERAGE,
        "Average Food",
        Some(
            "Everyday foodstuffs. Contains all the nutrients needed to survive for a few days.".into()
        ),
        current_category.clone(),
        250,
        25,
        1,
        COMPACT_STACK_SIZE,
        vec![], //vec![FoodQuality(0)],
        None
    )?;

    dsl.create_item_definition(
        ITEM_FOOD_LUXURY,
        "Luxury Food",
        Some("Super deluxe foodstuffs. Not necessary for anyone to survive.".into()),
        current_category.clone(),
        2000,
        75,
        1,
        COMPACT_STACK_SIZE,
        vec![], //vec![FoodQuality(0)],
        None
    )?;

    ///////////////////////////////////////////////////////////////////////////////////////
    // BiomatterProcessedLuxury
    current_category = ItemCategory::Resource(ResourceCategory::BiomatterProcessedLuxury);

    // Space "Fuel"
    dsl.create_item_definition(
        ITEM_ALCOHOL,
        "Space 'Fuel'",
        Some(
            "Actually a variant of vodka using common supplies found in spaceships to make. It's usually illegal due to its propensity to turn the users blind.".into()
        ),
        current_category.clone(),
        50,
        33,
        1,
        COMPACT_STACK_SIZE,
        vec![],
        None
    )?;

    Ok(())
}

fn ship_module_definitions(dsl: &DSL) -> Result<(), String> {
    use ItemMetadata::*;

    dsl.create_item_definition(
        SMOD_BASIC_MINING_LASER,
        "Basic Mining Laser",
        Some("Manufactured by many, functionally the same.".into()),
        ItemCategory::ShipModule(ShipModuleType::MiningLaserBasic),
        500,
        52,
        4,
        LARGE_STACK_SIZE,
        vec![MiningSpeedMultiplier(1.0), EnergyConsumption(1.75)],
        None
    )?;

    // Research Items
    dsl.create_item_definition(
        ITEM_RESEARCH_FRAGMENT,
        "Research Fragment",
        Some("A piece of research data usually collected from a laboratory station module.".into()),
        ItemCategory::Resource(ResourceCategory::ResearchDataFragments),
        150,
        52,
        1,
        COMPACT_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_RESEARCH_FRAGMENT_RARE,
        "Rare Research Fragment",
        Some(
            "A piece of research data rarely collected - usually from a laboratory module near a space anomaly.".into()
        ),
        ItemCategory::Resource(ResourceCategory::ResearchDataFragments),
        800,
        52,
        1,
        COMPACT_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_RESEARCH_FRAGMENT_EXOTIC,
        "Exotic Research Fragment",
        Some(
            "A piece of exotic research data collected from an alien race or itself is an active anomaly.".into()
        ),
        ItemCategory::Resource(ResourceCategory::ExoticMatter),
        5000,
        25,
        1,
        COMPACT_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_RESEARCH_DEVICE,
        "Research Device",
        Some(
            "Needed to build lab station modules, it's built out of computers and viveium crystals.".into()
        ),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentAdvanced),
        2500,
        13,
        8,
        MASSIVE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_RESEARCH_DEVICE_RARE,
        "Rare Research Device",
        Some(
            "A specialized research device made to process exotic research fragments and produce rare fragments. Much more expensive/resource-intensive than a standard research device.".into()
        ),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentAdvanced),
        10000,
        13,
        8,
        MASSIVE_STACK_SIZE,
        vec![],
        None
    )?;

    // Computer Components
    dsl.create_item_definition(
        ITEM_COMPUTER_WAFERS,
        "Computer Wafers",
        Some("Processed silicon into wafers for computer manufacturing.".into()),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        200,
        52,
        4,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_COMPUTER_CHIPS,
        "Computer Chips",
        Some("Wafers processed into functional chips for electronics.".into()),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        400,
        24,
        4,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_COMPUTER_CORE,
        "Computer Core",
        Some("Wafer processed into CPU cores for advanced computers.".into()),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        800,
        33,
        4,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_COMPUTER_BOARD,
        "Computer Board",
        Some("Chips and cores processed into a general purpose computer board.".into()),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        1200,
        15,
        4,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    // Metal Components
    dsl.create_item_definition(
        ITEM_METAL_PLATES,
        "Metal Plates",
        Some(
            "Metal plates for normal components. Used for floorboards, light armor, component casings, etc.".into()
        ),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        150,
        25,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_METAL_LINKAGES,
        "Metal Linkages",
        Some(
            "Metal connecting devices. Only a few are needed, but almost every module needs at least one.".into()
        ),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        300,
        33,
        4,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_METAL_ARMOR_PLATES,
        "Metal Armor Plates",
        Some(
            "Metal plates made specifically for armoring ships and the outside of stations.".into()
        ),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        500,
        52,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_METAL_HARDENED_ARMOR_PLATES,
        "Hardened Armor Plates",
        Some("Metal plates made for combat spacecraft with enhanced protection.".into()),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        1000,
        11,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_METAL_HULL_STRUCTURE,
        "Hull Structure",
        Some(
            "Metal plates and beams welded together to make a section of structure for a capital-class spacecraft or station.".into()
        ),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        2000,
        33,
        16,
        MASSIVE_STACK_SIZE,
        vec![],
        None
    )?;

    // Module Components
    dsl.create_item_definition(
        ITEM_MODULE_COMPONENTS,
        "Module Components",
        Some(
            "A generic component part. Needed by almost any ship or station module. Requires a few computer boards, hull structure, plates, etc.".into()
        ),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        800,
        33,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_MODULE_COMPONENTS_ADVANCED,
        "Advanced Module Components",
        Some(
            "An advanced component required for more complex components. Requires a few viveium crystals plus more computer boards than normal components.".into()
        ),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentAdvanced),
        2500,
        22,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_MODULE_COMPONENTS_SHIP,
        "Ship Module Components",
        Some("A generic small spacecraft-sized component.".into()),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        1000,
        13,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_MODULE_COMPONENTS_CAPITAL,
        "Capital Ship Components",
        Some("A generic large spacecraft-sized component.".into()),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentAdvanced),
        5000,
        9,
        16,
        MASSIVE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_MODULE_COMPONENTS_STATION,
        "Station Module Components",
        Some("A generic station-sized component.".into()),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentAdvanced),
        8000,
        13,
        16,
        MASSIVE_STACK_SIZE,
        vec![],
        None
    )?;

    // Specialized Components
    dsl.create_item_definition(
        ITEM_MODULE_COMPONENTS_WEAPON,
        "Weapon Components",
        Some("Specialized components for weapon systems.".into()),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        1500,
        21,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_MODULE_COMPONENTS_ENGINE,
        "Engine Components",
        Some("Specialized components for propulsion systems.".into()),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        1200,
        21,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_MODULE_COMPONENTS_SHIELD,
        "Shield Components",
        Some("Specialized components for shield generator systems.".into()),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        1800,
        32,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_MODULE_COMPONENTS_MISSILE,
        "Missile Components",
        Some("Specialized components for missile systems and launchers.".into()),
        ItemCategory::Resource(ResourceCategory::ManufacturedComponentBasic),
        2000,
        32,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    Ok(())
}
