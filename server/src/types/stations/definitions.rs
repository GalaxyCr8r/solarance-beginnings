use std::u8;

use log::info;
use spacetimedb::*;
use spacetimedsl::dsl;

use crate::types::items::definitions::{
    ITEM_ENERGY_CELL, ITEM_FOOD_RATIONS, ITEM_METAL_HULL_STRUCTURE, ITEM_METAL_PLATES,
};

use super::*;

// Trading Modules
pub const MODULE_TRADING_BAZAAR: u32 = 1_000;
pub const MODULE_TRADING_MARKET: u32 = 1_010;
pub const MODULE_TRADING_PORT: u32 = 1_020;

// Storage depot modules
pub const MODULE_MINOR_DEPOT: u32 = 2_000;
pub const MODULE_MAJOR_DEPOT: u32 = 2_010;
pub const MODULE_CAPITOL_DEPOT: u32 = 2_020;

// Refinery modules
pub const MODULE_REFINERY_MINOR: u32 = 3_110;
pub const MODULE_REFINERY_MAJOR: u32 = 3_111;
pub const MODULE_REFINERY_EXOTIC: u32 = 3_112;

// Farm modules
pub const MODULE_FARM_BASIC: u32 = 4_000;
pub const MODULE_FARM_STANDARD: u32 = 4_001;
pub const MODULE_FARM_ADVANCED: u32 = 4_002;
pub const MODULE_FARM_LUXURY: u32 = 4_003;

// Laboratory modules
pub const MODULE_LABORATORY_BASIC: u32 = 5_000;
pub const MODULE_LABORATORY_ADVANCED: u32 = 5_001;
pub const MODULE_LABORATORY_EXOTIC: u32 = 5_002;

// Manufacturing modules
pub const MODULE_MANUFACTURING_FACTORY: u32 = 6_000;
pub const MODULE_MANUFACTURING_FACTORY_ADVANCED: u32 = 6_001;
pub const MODULE_MANUFACTURING_ASSEMBLER: u32 = 6_002;
pub const MODULE_MANUFACTURING_SHIPYARD: u32 = 6_003;

// Solar array modules
pub const MODULE_SOLAR_ARRAY_SMALL: u32 = 7_000;
pub const MODULE_SOLAR_ARRAY_MEDIUM: u32 = 7_001;
pub const MODULE_SOLAR_ARRAY_LARGE: u32 = 7_002;

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    info!("Initing station modules...");
    let dsl = dsl(ctx);

    basic_station_module_blueprints(ctx)?;

    info!(
        "Station Modules Loaded: {}",
        dsl.get_all_station_module_blueprints().count()
    );
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn calculate_basic_module_cost(max_hp: u32, relative_complexity: u8) -> Vec<ResourceAmount> {
    vec![
        ResourceAmount::new(
            ITEM_METAL_HULL_STRUCTURE,
            max_hp + ((relative_complexity as u32) * 500) / 10,
        ),
        ResourceAmount::new(
            ITEM_ENERGY_CELL,
            max_hp + ((relative_complexity as u32) * 1_000) / 5,
        ),
        // ResourceAmount::new(ITEM_COMPUTER_CORE, max_hp / (575 / (relative_complexity as u32))), // this is a problem child, its causing the wasm to fail.
        ResourceAmount::new(
            ITEM_METAL_PLATES,
            max_hp + ((relative_complexity as u32) * 100) / 125,
        ),
        ResourceAmount::new(
            ITEM_FOOD_RATIONS,
            max_hp + ((relative_complexity as u32) * 1_000) / 25,
        ),
    ]
}

fn basic_station_module_blueprints(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    dsl.create_station_module_blueprint(
        MODULE_TRADING_BAZAAR,
        "Trading Bazaar",
        "A basic trading port. Can fit a modest selection of goods.",
        StationModuleCategory::LogisticsAndStorage,
        StationModuleSpecificType::TradingPort,
        calculate_basic_module_cost(30_000, 0),
        0,
        5_000.0,
        2_500.0,
        100.0,
        0,
        10,
        Some(768),
        Some(10),
        Some("station.icon.trading".to_string()),
        3_000,
        30_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_TRADING_MARKET,
        "Trading Market",
        "A basic trading port. Can fit a reasonable amount of goods.",
        StationModuleCategory::LogisticsAndStorage,
        StationModuleSpecificType::TradingPort,
        calculate_basic_module_cost(50_000, 1),
        0,
        6_000.0,
        2_750.0,
        100.0,
        0,
        30,
        Some(1024),
        Some(10),
        Some("station.icon.trading".to_string()),
        5_000,
        50_000,
    )?;

    /////////////////////////////////////////////////////////////////////////////////////

    dsl.create_station_module_blueprint(
        MODULE_TRADING_PORT,
        "Trading Port",
        "A large trading facility. Can handle significant cargo volumes.",
        StationModuleCategory::LogisticsAndStorage,
        StationModuleSpecificType::TradingPort,
        calculate_basic_module_cost(75_000, 2),
        0,
        8_000.0,
        3_000.0,
        100.0,
        0,
        50,
        Some(2048),
        Some(15),
        Some("station.icon.trading".to_string()),
        7_500,
        75_000,
    )?;

    /////////////////////////////////////////////////////////////////////////////////////
    // Storage Depot Modules

    dsl.create_station_module_blueprint(
        MODULE_MINOR_DEPOT,
        "Minor Storage Depot",
        "A small storage facility for basic cargo needs.",
        StationModuleCategory::LogisticsAndStorage,
        StationModuleSpecificType::StorageDepot,
        calculate_basic_module_cost(20_000, 0),
        0,
        2_000.0,
        1_500.0,
        50.0,
        0,
        5,
        Some(1000),
        Some(5),
        Some("station.icon.storage".to_string()),
        1_500,
        20_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_MAJOR_DEPOT,
        "Major Storage Depot",
        "A large storage facility for significant cargo volumes.",
        StationModuleCategory::LogisticsAndStorage,
        StationModuleSpecificType::StorageDepot,
        calculate_basic_module_cost(40_000, 1),
        0,
        4_000.0,
        2_000.0,
        75.0,
        0,
        15,
        Some(2500),
        Some(10),
        Some("station.icon.storage".to_string()),
        3_000,
        40_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_CAPITOL_DEPOT,
        "Capitol Storage Depot",
        "A massive storage facility for industrial-scale operations.",
        StationModuleCategory::LogisticsAndStorage,
        StationModuleSpecificType::StorageDepot,
        calculate_basic_module_cost(80_000, 2),
        0,
        8_000.0,
        3_000.0,
        100.0,
        0,
        30,
        Some(5000),
        Some(20),
        Some("station.icon.storage".to_string()),
        6_000,
        80_000,
    )?;

    /////////////////////////////////////////////////////////////////////////////////////
    // Refinery Modules

    dsl.create_station_module_blueprint(
        MODULE_REFINERY_MINOR,
        "Minor Ore Refinery",
        "A basic refinery, can process most common ores.",
        StationModuleCategory::ResourceProductionAndRefining,
        StationModuleSpecificType::RefineryBasicOre,
        calculate_basic_module_cost(25_000, 0),
        0,
        50_000.0,
        2_500.0,
        500.0,
        1,
        3,
        Some(2000),
        Some(-5),
        Some("station.icon.refinery".to_string()),
        2_000,
        25_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_REFINERY_MAJOR,
        "Major Ore Refinery",
        "An advanced refinery with higher throughput and efficiency.",
        StationModuleCategory::ResourceProductionAndRefining,
        StationModuleSpecificType::RefineryBasicOre,
        calculate_basic_module_cost(50_000, 1),
        0,
        75_000.0,
        3_500.0,
        750.0,
        2,
        8,
        Some(4000),
        Some(-10),
        Some("station.icon.refinery".to_string()),
        4_000,
        50_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_REFINERY_EXOTIC,
        "Exotic Material Refinery",
        "A specialized refinery for processing rare and exotic materials.",
        StationModuleCategory::ResourceProductionAndRefining,
        StationModuleSpecificType::RefineryExoticOre,
        calculate_basic_module_cost(100_000, 3),
        0,
        150_000.0,
        5_000.0,
        1_500.0,
        3,
        15,
        Some(8000),
        Some(-20),
        Some("station.icon.refinery".to_string()),
        8_000,
        100_000,
    )?;

    /////////////////////////////////////////////////////////////////////////////////////
    // Farm Modules

    dsl.create_station_module_blueprint(
        MODULE_FARM_BASIC,
        "Basic Farm",
        "A simple agricultural facility producing basic food rations.",
        StationModuleCategory::ResourceProductionAndRefining,
        StationModuleSpecificType::FarmStandard,
        calculate_basic_module_cost(15_000, 0),
        0,
        10_000.0,
        1_500.0,
        200.0,
        1,
        2,
        Some(500),
        Some(-2),
        Some("station.icon.farm".to_string()),
        1_000,
        15_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_FARM_STANDARD,
        "Standard Farm",
        "An improved agricultural facility with better yields.",
        StationModuleCategory::ResourceProductionAndRefining,
        StationModuleSpecificType::FarmStandard,
        calculate_basic_module_cost(25_000, 1),
        0,
        15_000.0,
        2_000.0,
        300.0,
        1,
        5,
        Some(1000),
        Some(-3),
        Some("station.icon.farm".to_string()),
        2_000,
        25_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_FARM_ADVANCED,
        "Advanced Farm",
        "A high-tech agricultural facility with optimized growing conditions.",
        StationModuleCategory::ResourceProductionAndRefining,
        StationModuleSpecificType::FarmStandard,
        calculate_basic_module_cost(40_000, 2),
        0,
        25_000.0,
        2_500.0,
        500.0,
        2,
        10,
        Some(2000),
        Some(-5),
        Some("station.icon.farm".to_string()),
        3_500,
        40_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_FARM_LUXURY,
        "Luxury Farm",
        "A premium agricultural facility producing high-quality luxury foods.",
        StationModuleCategory::ResourceProductionAndRefining,
        StationModuleSpecificType::FarmLuxury,
        calculate_basic_module_cost(60_000, 3),
        0,
        40_000.0,
        3_000.0,
        750.0,
        2,
        15,
        Some(3000),
        Some(-8),
        Some("station.icon.farm".to_string()),
        5_000,
        60_000,
    )?;

    /////////////////////////////////////////////////////////////////////////////////////
    // Laboratory Modules

    dsl.create_station_module_blueprint(
        MODULE_LABORATORY_BASIC,
        "Basic Laboratory",
        "A research facility for basic scientific studies and development.",
        StationModuleCategory::ResearchAndDevelopment,
        StationModuleSpecificType::Laboratory,
        calculate_basic_module_cost(35_000, 1),
        0,
        30_000.0,
        2_000.0,
        400.0,
        1,
        5,
        Some(1500),
        Some(-5),
        Some("station.icon.laboratory".to_string()),
        2_500,
        35_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_LABORATORY_ADVANCED,
        "Advanced Laboratory",
        "A sophisticated research facility with advanced equipment and capabilities.",
        StationModuleCategory::ResearchAndDevelopment,
        StationModuleSpecificType::Laboratory,
        calculate_basic_module_cost(70_000, 2),
        0,
        60_000.0,
        3_000.0,
        800.0,
        2,
        12,
        Some(3000),
        Some(-10),
        Some("station.icon.laboratory".to_string()),
        5_000,
        70_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_LABORATORY_EXOTIC,
        "Exotic Research Laboratory",
        "A cutting-edge research facility for studying exotic materials and phenomena.",
        StationModuleCategory::ResearchAndDevelopment,
        StationModuleSpecificType::Laboratory,
        calculate_basic_module_cost(120_000, 4),
        0,
        100_000.0,
        4_000.0,
        1_500.0,
        3,
        20,
        Some(5000),
        Some(-20),
        Some("station.icon.laboratory".to_string()),
        10_000,
        120_000,
    )?;

    /////////////////////////////////////////////////////////////////////////////////////
    // Manufacturing Modules

    dsl.create_station_module_blueprint(
        MODULE_MANUFACTURING_FACTORY,
        "Basic Factory",
        "A manufacturing facility for producing basic components and goods.",
        StationModuleCategory::ManufacturingAndAssembly,
        StationModuleSpecificType::FactoryBasicComponents,
        calculate_basic_module_cost(45_000, 1),
        0,
        40_000.0,
        2_500.0,
        600.0,
        2,
        8,
        Some(2000),
        Some(-8),
        Some("station.icon.factory".to_string()),
        3_500,
        45_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_MANUFACTURING_FACTORY_ADVANCED,
        "Advanced Factory",
        "A high-tech manufacturing facility with automated production lines.",
        StationModuleCategory::ManufacturingAndAssembly,
        StationModuleSpecificType::FactoryAdvancedComponents,
        calculate_basic_module_cost(80_000, 2),
        0,
        70_000.0,
        3_500.0,
        1_000.0,
        3,
        15,
        Some(4000),
        Some(-15),
        Some("station.icon.factory".to_string()),
        6_500,
        80_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_MANUFACTURING_ASSEMBLER,
        "Component Assembler",
        "A specialized facility for assembling complex components from basic parts.",
        StationModuleCategory::ManufacturingAndAssembly,
        StationModuleSpecificType::ComponentAssembler,
        calculate_basic_module_cost(60_000, 2),
        0,
        50_000.0,
        3_000.0,
        800.0,
        2,
        12,
        Some(3000),
        Some(-12),
        Some("station.icon.assembler".to_string()),
        5_000,
        60_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_MANUFACTURING_SHIPYARD,
        "Shipyard Fabrication",
        "A massive facility capable of constructing and repairing spacecraft.",
        StationModuleCategory::ManufacturingAndAssembly,
        StationModuleSpecificType::ShipyardFabrication,
        calculate_basic_module_cost(150_000, 4),
        0,
        120_000.0,
        5_000.0,
        2_000.0,
        4,
        25,
        Some(8000),
        Some(-25),
        Some("station.icon.shipyard".to_string()),
        12_000,
        150_000,
    )?;

    /////////////////////////////////////////////////////////////////////////////////////
    // Solar Array Modules

    dsl.create_station_module_blueprint(
        MODULE_SOLAR_ARRAY_SMALL,
        "Small Solar Array",
        "A compact solar power generation system for basic energy needs.",
        StationModuleCategory::ResourceProductionAndRefining,
        StationModuleSpecificType::SolarArray,
        calculate_basic_module_cost(20_000, 0),
        0,
        15_000.0,
        1_500.0,
        300.0,
        1,
        3,
        Some(1000),
        Some(5),
        Some("station.icon.solar".to_string()),
        1_500,
        20_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_SOLAR_ARRAY_MEDIUM,
        "Medium Solar Array",
        "A mid-sized solar power system with improved efficiency.",
        StationModuleCategory::ResourceProductionAndRefining,
        StationModuleSpecificType::SolarArray,
        calculate_basic_module_cost(35_000, 1),
        0,
        25_000.0,
        2_000.0,
        500.0,
        1,
        6,
        Some(2000),
        Some(8),
        Some("station.icon.solar".to_string()),
        2_500,
        35_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_SOLAR_ARRAY_LARGE,
        "Large Solar Array",
        "A large-scale solar power installation for industrial energy requirements.",
        StationModuleCategory::ResourceProductionAndRefining,
        StationModuleSpecificType::SolarArray,
        calculate_basic_module_cost(60_000, 2),
        0,
        45_000.0,
        3_000.0,
        800.0,
        2,
        12,
        Some(4000),
        Some(15),
        Some("station.icon.solar".to_string()),
        4_500,
        60_000,
    )?;

    Ok(())
}
