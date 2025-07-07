use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::{ dsl, DSL };

use super::*;

pub const SMOD_BASIC_MINING_LASER: u32 = 17_000;

pub const ITEM_RESEARCH_FRAGMENT: u32 = 10_000;
pub const ITEM_RESEARCH_FRAGMENT_RARE: u32 = 10_001;
pub const ITEM_RESEARCH_FRAGMENT_EXOTIC: u32 = 10_002;

pub const ITEM_RESEARCH_DEVICE: u32 = 10_000;
pub const ITEM_RESEARCH_DEVICE_RARE: u32 = 10_001;

pub const ITEM_ENERGY_CELL: u32 = 0_000;
pub const ITEM_COMPRESSED_HYDROGEN: u32 = 0_010;
pub const ITEM_JUMPDRIVE_FUEL: u32 = 0_050;

pub const ITEM_ALCOHOL: u32 = 1_002;

pub const ITEM_ICE_ORE: u32 = 2_000;
pub const ITEM_CARBON_ORE: u32 = 2_001;
pub const ITEM_IRON_ORE: u32 = 2_002;
pub const ITEM_SILICON_ORE: u32 = 2_003;
pub const ITEM_URANIUM_ORE: u32 = 2_004;
pub const ITEM_VIVEIUM_ORE: u32 = 2_005;
pub const ITEM_TITANIUM_ORE: u32 = 2_006;

pub const ITEM_CARBON_RAW: u32 = 2_101;
pub const ITEM_IRON_INGOT: u32 = 2_102;
pub const ITEM_SILICON_RAW: u32 = 2_103;
pub const ITEM_URANIUM_INGOT: u32 = 2_104;
pub const ITEM_URANIUM_ENRICHED: u32 = 2_114;
pub const ITEM_VIVEIUM_INGOT: u32 = 2_105;
pub const ITEM_TITANIUM_INGOT: u32 = 2_106;

pub const ITEM_VIVEIUM_CRYSTAL: u32 = 2_206;

pub const ITEM_HELIUM_GAS: u32 = 2_503;
pub const ITEM_HYDROGEN_GAS: u32 = 2_504;

pub const ITEM_WATER: u32 = 2_200;

pub const ITEM_BIOMATTER_RAW: u32 = 3_001;
pub const ITEM_BIOMATTER_REFUSE: u32 = 3_002;
pub const ITEM_BIOMATTER_RARE: u32 = 3_003;

pub const ITEM_FOOD_RATIONS: u32 = 3_101;
pub const ITEM_FOOD_AVERAGE: u32 = 3_102;
pub const ITEM_FOOD_LUXURY: u32 = 3_103;

pub const ITEM_COMPUTER_CHIPS: u32 = 4_000;
pub const ITEM_COMPUTER_CORE: u32 = 4_001;

pub const ITEM_MODULE_COMPONENTS: u32 = 4_100;
pub const ITEM_MODULE_COMPONENTS_ADVANCED: u32 = 4_101;
pub const ITEM_MODULE_COMPONENTS_SHIP: u32 = 4_102;
pub const ITEM_MODULE_COMPONENTS_WEAPON: u32 = 4_103;

pub const ITEM_METAL_PLATES: u32 = 4_200;
pub const ITEM_METAL_HULL: u32 = 4_201;

// Stack Sizes
pub const COMPACT_STACK_SIZE: u8 = 64;
pub const LOOSE_STACK_SIZE: u8 = 32;
pub const LARGE_STACK_SIZE: u8 = 16;
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
    use ItemMetadata::*;

    ///////////////////////////////////////////////////////////////////////////////////////
    // ENERGY
    let mut current_category = ItemCategory::Resource(ResourceCategory::StoredEnergy);

    // Energy Cells
    dsl.create_item_definition(
        ITEM_ENERGY_CELL,
        "Energy Cell",
        Some(
            "Energy Cells are the unified energy storage used throughout the whole known universe. 
            The unification of the energy storage specifications allows for interstellar trading with this most basic of all products.".into()
        ),
        current_category,
        20,
        1,
        COMPACT_STACK_SIZE,
        vec![],
        None
    )?;

    ///////////////////////////////////////////////////////////////////////////////////////
    // ORES
    current_category = ItemCategory::Resource(ResourceCategory::RawOre);

    dsl.create_item_definition(
        ITEM_ICE_ORE,
        "Ice Ore",
        Some("Raw ice mined from an asteroid. Needs to be melted and filtered.".into()),
        current_category.clone(),
        50,
        8,
        LOOSE_STACK_SIZE,
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
        8,
        LOOSE_STACK_SIZE,
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
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    ///////////////////////////////////////////////////////////////////////////////////////
    // Ingots
    current_category = ItemCategory::Resource(ResourceCategory::RefinedIngot);

    // Iron, Silicon, Uranium, Viveium, Titanium Ingots
    {
        dsl.create_item_definition(
            ITEM_IRON_INGOT,
            "Iron Ingot",
            Some("Refined iron ingot. Used in many ship components.".into()),
            current_category.clone(),
            150,
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
            8,
            LOOSE_STACK_SIZE,
            vec![],
            None
        )?;
    }

    ///////////////////////////////////////////////////////////////////////////////////////
    // RAW BIOMATTER
    current_category = ItemCategory::Resource(ResourceCategory::BiomatterRaw);

    dsl.create_item_definition(
        ITEM_WATER,
        "Water Barrel",
        Some("A barrel of clean drinkable water.".into()),
        current_category.clone(),
        75,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_BIOMATTER_REFUSE,
        "Biomatter Waste",
        Some("Decomposing plant/food waste, recycling organic materials, or manure.".into()),
        current_category.clone(),
        105,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    dsl.create_item_definition(
        ITEM_BIOMATTER_RAW,
        "Compost",
        Some(
            "Compost is a mixture of ingredients used as plant fertilizer and to improve soil's physical, chemical, and biological properties. It is commonly prepared by decomposing plant and food waste, recycling organic materials, and manure.".into()
        ),
        current_category.clone(),
        275,
        8,
        LOOSE_STACK_SIZE,
        vec![],
        None
    )?;

    ///////////////////////////////////////////////////////////////////////////////////////
    // BiomatterProcessedFood
    current_category = ItemCategory::Resource(ResourceCategory::BiomatterProcessedFood);

    // Food Rations
    dsl.create_item_definition(
        ITEM_FOOD_RATIONS,
        "Food Rations",
        Some(
            "A basic food ration. Contains all the nutrients needed to survive for a few days.".into()
        ),
        current_category.clone(),
        100,
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
        "Space Fuel",
        Some(
            "Actually a variant of vodka using common supplies found in spaceships to make. It's usually illegal due to its propensity to turn the users blind.".into()
        ),
        current_category.clone(),
        50,
        1,
        COMPACT_STACK_SIZE,
        vec![],
        None
    )?;

    ///////////////////////////////////////////////////////////////////////////////////////
    // ?
    current_category = ItemCategory::Resource(ResourceCategory::RefinedIngot);

    ///////////////////////////////////////////////////////////////////////////////////////
    // ?
    current_category = ItemCategory::Resource(ResourceCategory::RawOre);

    ///////////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////////////////////////////

    dsl.create_item_definition(
        SMOD_BASIC_MINING_LASER,
        "Basic Mining Laser",
        Some("Manufactured by many, functionally the same.".into()),
        ItemCategory::ShipModule(ShipModuleType::MiningLaserBasic),
        500,
        4,
        LARGE_STACK_SIZE,
        vec![MiningSpeedMultiplier(1.0), EnergyConsumption(1.75)],
        None
    )?;

    Ok(())
}
