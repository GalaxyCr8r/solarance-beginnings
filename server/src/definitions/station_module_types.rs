use std::u8;

use log::info;
use spacetimedb::SpacetimeType;
use spacetimedsl::*;

use crate::{
    definitions::item_types::*,
    tables::{economy::ResourceAmount, stations::*},
};

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

pub fn init<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    info!("Initing station modules...");

    basic_station_module_blueprints(dsl)?;
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

fn basic_station_module_blueprints<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
) -> Result<(), String> {
    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_TRADING_BAZAAR,
        name: "Trading Bazaar".to_string(),
        description: "A basic trading port. Can fit a modest selection of goods.".to_string(),
        category: StationModuleCategory::LogisticsAndStorage,
        specific_type: StationModuleSpecificType::TradingPort,
        build_cost_resources: calculate_basic_module_cost(30_000, 0),
        build_time_seconds: 0,
        power_consumption_mw_operational: 5_000.0,
        power_consumption_mw_idle: 2_500.0,
        cpu_load_flops: 100.0,
        required_station_tech_level: 0,
        max_internal_storage_slots: 10,
        max_internal_storage_volume_per_slot_m3: Some(768),
        provides_station_morale_boost: Some(10),
        icon_asset_id: Some("station.icon.trading".to_string()),
        construction_hp: 3_000,
        operational_hp: 30_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_TRADING_MARKET,
        name: "Trading Market".to_string(),
        description: "A basic trading port. Can fit a reasonable amount of goods.".to_string(),
        category: StationModuleCategory::LogisticsAndStorage,
        specific_type: StationModuleSpecificType::TradingPort,
        build_cost_resources: calculate_basic_module_cost(50_000, 1),
        build_time_seconds: 0,
        power_consumption_mw_operational: 6_000.0,
        power_consumption_mw_idle: 2_750.0,
        cpu_load_flops: 100.0,
        required_station_tech_level: 0,
        max_internal_storage_slots: 30,
        max_internal_storage_volume_per_slot_m3: Some(1024),
        provides_station_morale_boost: Some(10),
        icon_asset_id: Some("station.icon.trading".to_string()),
        construction_hp: 5_000,
        operational_hp: 50_000,
    })?;

    /////////////////////////////////////////////////////////////////////////////////////

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_TRADING_PORT,
        name: "Trading Port".to_string(),
        description: "A large trading facility. Can handle significant cargo volumes.".to_string(),
        category: StationModuleCategory::LogisticsAndStorage,
        specific_type: StationModuleSpecificType::TradingPort,
        build_cost_resources: calculate_basic_module_cost(75_000, 2),
        build_time_seconds: 0,
        power_consumption_mw_operational: 8_000.0,
        power_consumption_mw_idle: 3_000.0,
        cpu_load_flops: 100.0,
        required_station_tech_level: 0,
        max_internal_storage_slots: 50,
        max_internal_storage_volume_per_slot_m3: Some(2048),
        provides_station_morale_boost: Some(15),
        icon_asset_id: Some("station.icon.trading".to_string()),
        construction_hp: 7_500,
        operational_hp: 75_000,
    })?;

    /////////////////////////////////////////////////////////////////////////////////////
    // Storage Depot Modules

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_MINOR_DEPOT,
        name: "Minor Storage Depot".to_string(),
        description: "A small storage facility for basic cargo needs.".to_string(),
        category: StationModuleCategory::LogisticsAndStorage,
        specific_type: StationModuleSpecificType::StorageDepot,
        build_cost_resources: calculate_basic_module_cost(20_000, 0),
        build_time_seconds: 0,
        power_consumption_mw_operational: 2_000.0,
        power_consumption_mw_idle: 1_500.0,
        cpu_load_flops: 50.0,
        required_station_tech_level: 0,
        max_internal_storage_slots: 5,
        max_internal_storage_volume_per_slot_m3: Some(1000),
        provides_station_morale_boost: Some(5),
        icon_asset_id: Some("station.icon.storage".to_string()),
        construction_hp: 1_500,
        operational_hp: 20_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_MAJOR_DEPOT,
        name: "Major Storage Depot".to_string(),
        description: "A large storage facility for significant cargo volumes.".to_string(),
        category: StationModuleCategory::LogisticsAndStorage,
        specific_type: StationModuleSpecificType::StorageDepot,
        build_cost_resources: calculate_basic_module_cost(40_000, 1),
        build_time_seconds: 0,
        power_consumption_mw_operational: 4_000.0,
        power_consumption_mw_idle: 2_000.0,
        cpu_load_flops: 75.0,
        required_station_tech_level: 0,
        max_internal_storage_slots: 15,
        max_internal_storage_volume_per_slot_m3: Some(2500),
        provides_station_morale_boost: Some(10),
        icon_asset_id: Some("station.icon.storage".to_string()),
        construction_hp: 3_000,
        operational_hp: 40_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_CAPITOL_DEPOT,
        name: "Capitol Storage Depot".to_string(),
        description: "A massive storage facility for industrial-scale operations.".to_string(),
        category: StationModuleCategory::LogisticsAndStorage,
        specific_type: StationModuleSpecificType::StorageDepot,
        build_cost_resources: calculate_basic_module_cost(80_000, 2),
        build_time_seconds: 0,
        power_consumption_mw_operational: 8_000.0,
        power_consumption_mw_idle: 3_000.0,
        cpu_load_flops: 100.0,
        required_station_tech_level: 0,
        max_internal_storage_slots: 30,
        max_internal_storage_volume_per_slot_m3: Some(5000),
        provides_station_morale_boost: Some(20),
        icon_asset_id: Some("station.icon.storage".to_string()),
        construction_hp: 6_000,
        operational_hp: 80_000,
    })?;

    /////////////////////////////////////////////////////////////////////////////////////
    // Refinery Modules

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_REFINERY_MINOR,
        name: "Minor Ore Refinery".to_string(),
        description: "A basic refinery, can process most common ores.".to_string(),
        category: StationModuleCategory::ResourceProductionAndRefining,
        specific_type: StationModuleSpecificType::RefineryBasicOre,
        build_cost_resources: calculate_basic_module_cost(25_000, 0),
        build_time_seconds: 0,
        power_consumption_mw_operational: 50_000.0,
        power_consumption_mw_idle: 2_500.0,
        cpu_load_flops: 500.0,
        required_station_tech_level: 1,
        max_internal_storage_slots: 3,
        max_internal_storage_volume_per_slot_m3: Some(2000),
        provides_station_morale_boost: Some(-5),
        icon_asset_id: Some("station.icon.refinery".to_string()),
        construction_hp: 2_000,
        operational_hp: 25_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_REFINERY_MAJOR,
        name: "Major Ore Refinery".to_string(),
        description: "An advanced refinery with higher throughput and efficiency.".to_string(),
        category: StationModuleCategory::ResourceProductionAndRefining,
        specific_type: StationModuleSpecificType::RefineryBasicOre,
        build_cost_resources: calculate_basic_module_cost(50_000, 1),
        build_time_seconds: 0,
        power_consumption_mw_operational: 75_000.0,
        power_consumption_mw_idle: 3_500.0,
        cpu_load_flops: 750.0,
        required_station_tech_level: 2,
        max_internal_storage_slots: 8,
        max_internal_storage_volume_per_slot_m3: Some(4000),
        provides_station_morale_boost: Some(-10),
        icon_asset_id: Some("station.icon.refinery".to_string()),
        construction_hp: 4_000,
        operational_hp: 50_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_REFINERY_EXOTIC,
        name: "Exotic Material Refinery".to_string(),
        description: "A specialized refinery for processing rare and exotic materials.".to_string(),
        category: StationModuleCategory::ResourceProductionAndRefining,
        specific_type: StationModuleSpecificType::RefineryExoticOre,
        build_cost_resources: calculate_basic_module_cost(100_000, 3),
        build_time_seconds: 0,
        power_consumption_mw_operational: 150_000.0,
        power_consumption_mw_idle: 5_000.0,
        cpu_load_flops: 1_500.0,
        required_station_tech_level: 3,
        max_internal_storage_slots: 15,
        max_internal_storage_volume_per_slot_m3: Some(8000),
        provides_station_morale_boost: Some(-20),
        icon_asset_id: Some("station.icon.refinery".to_string()),
        construction_hp: 8_000,
        operational_hp: 100_000,
    })?;

    /////////////////////////////////////////////////////////////////////////////////////
    // Farm Modules

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_FARM_BASIC,
        name: "Basic Farm".to_string(),
        description: "A simple agricultural facility producing basic food rations.".to_string(),
        category: StationModuleCategory::ResourceProductionAndRefining,
        specific_type: StationModuleSpecificType::FarmStandard,
        build_cost_resources: calculate_basic_module_cost(15_000, 0),
        build_time_seconds: 0,
        power_consumption_mw_operational: 10_000.0,
        power_consumption_mw_idle: 1_500.0,
        cpu_load_flops: 200.0,
        required_station_tech_level: 1,
        max_internal_storage_slots: 2,
        max_internal_storage_volume_per_slot_m3: Some(500),
        provides_station_morale_boost: Some(-2),
        icon_asset_id: Some("station.icon.farm".to_string()),
        construction_hp: 1_000,
        operational_hp: 15_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_FARM_STANDARD,
        name: "Standard Farm".to_string(),
        description: "An improved agricultural facility with better yields.".to_string(),
        category: StationModuleCategory::ResourceProductionAndRefining,
        specific_type: StationModuleSpecificType::FarmStandard,
        build_cost_resources: calculate_basic_module_cost(25_000, 1),
        build_time_seconds: 0,
        power_consumption_mw_operational: 15_000.0,
        power_consumption_mw_idle: 2_000.0,
        cpu_load_flops: 300.0,
        required_station_tech_level: 1,
        max_internal_storage_slots: 5,
        max_internal_storage_volume_per_slot_m3: Some(1000),
        provides_station_morale_boost: Some(-3),
        icon_asset_id: Some("station.icon.farm".to_string()),
        construction_hp: 2_000,
        operational_hp: 25_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_FARM_ADVANCED,
        name: "Advanced Farm".to_string(),
        description: "A high-tech agricultural facility with optimized growing conditions."
            .to_string(),
        category: StationModuleCategory::ResourceProductionAndRefining,
        specific_type: StationModuleSpecificType::FarmStandard,
        build_cost_resources: calculate_basic_module_cost(40_000, 2),
        build_time_seconds: 0,
        power_consumption_mw_operational: 25_000.0,
        power_consumption_mw_idle: 2_500.0,
        cpu_load_flops: 500.0,
        required_station_tech_level: 2,
        max_internal_storage_slots: 10,
        max_internal_storage_volume_per_slot_m3: Some(2000),
        provides_station_morale_boost: Some(-5),
        icon_asset_id: Some("station.icon.farm".to_string()),
        construction_hp: 3_500,
        operational_hp: 40_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_FARM_LUXURY,
        name: "Luxury Farm".to_string(),
        description: "A premium agricultural facility producing high-quality luxury foods."
            .to_string(),
        category: StationModuleCategory::ResourceProductionAndRefining,
        specific_type: StationModuleSpecificType::FarmLuxury,
        build_cost_resources: calculate_basic_module_cost(60_000, 3),
        build_time_seconds: 0,
        power_consumption_mw_operational: 40_000.0,
        power_consumption_mw_idle: 3_000.0,
        cpu_load_flops: 750.0,
        required_station_tech_level: 2,
        max_internal_storage_slots: 15,
        max_internal_storage_volume_per_slot_m3: Some(3000),
        provides_station_morale_boost: Some(-8),
        icon_asset_id: Some("station.icon.farm".to_string()),
        construction_hp: 5_000,
        operational_hp: 60_000,
    })?;

    /////////////////////////////////////////////////////////////////////////////////////
    // Laboratory Modules

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_LABORATORY_BASIC,
        name: "Basic Laboratory".to_string(),
        description: "A research facility for basic scientific studies and development."
            .to_string(),
        category: StationModuleCategory::ResearchAndDevelopment,
        specific_type: StationModuleSpecificType::Laboratory,
        build_cost_resources: calculate_basic_module_cost(35_000, 1),
        build_time_seconds: 0,
        power_consumption_mw_operational: 30_000.0,
        power_consumption_mw_idle: 2_000.0,
        cpu_load_flops: 400.0,
        required_station_tech_level: 1,
        max_internal_storage_slots: 5,
        max_internal_storage_volume_per_slot_m3: Some(1500),
        provides_station_morale_boost: Some(-5),
        icon_asset_id: Some("station.icon.laboratory".to_string()),
        construction_hp: 2_500,
        operational_hp: 35_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_LABORATORY_ADVANCED,
        name: "Advanced Laboratory".to_string(),
        description: "A sophisticated research facility with advanced equipment and capabilities."
            .to_string(),
        category: StationModuleCategory::ResearchAndDevelopment,
        specific_type: StationModuleSpecificType::Laboratory,
        build_cost_resources: calculate_basic_module_cost(70_000, 2),
        build_time_seconds: 0,
        power_consumption_mw_operational: 60_000.0,
        power_consumption_mw_idle: 3_000.0,
        cpu_load_flops: 800.0,
        required_station_tech_level: 2,
        max_internal_storage_slots: 12,
        max_internal_storage_volume_per_slot_m3: Some(3000),
        provides_station_morale_boost: Some(-10),
        icon_asset_id: Some("station.icon.laboratory".to_string()),
        construction_hp: 5_000,
        operational_hp: 70_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_LABORATORY_EXOTIC,
        name: "Exotic Research Laboratory".to_string(),
        description:
            "A cutting-edge research facility for studying exotic materials and phenomena."
                .to_string(),
        category: StationModuleCategory::ResearchAndDevelopment,
        specific_type: StationModuleSpecificType::Laboratory,
        build_cost_resources: calculate_basic_module_cost(120_000, 4),
        build_time_seconds: 0,
        power_consumption_mw_operational: 100_000.0,
        power_consumption_mw_idle: 4_000.0,
        cpu_load_flops: 1_500.0,
        required_station_tech_level: 3,
        max_internal_storage_slots: 20,
        max_internal_storage_volume_per_slot_m3: Some(5000),
        provides_station_morale_boost: Some(-20),
        icon_asset_id: Some("station.icon.laboratory".to_string()),
        construction_hp: 10_000,
        operational_hp: 120_000,
    })?;

    /////////////////////////////////////////////////////////////////////////////////////
    // Manufacturing Modules

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_MANUFACTURING_FACTORY,
        name: "Basic Factory".to_string(),
        description: "A manufacturing facility for producing basic components and goods."
            .to_string(),
        category: StationModuleCategory::ManufacturingAndAssembly,
        specific_type: StationModuleSpecificType::FactoryBasicComponents,
        build_cost_resources: calculate_basic_module_cost(45_000, 1),
        build_time_seconds: 0,
        power_consumption_mw_operational: 40_000.0,
        power_consumption_mw_idle: 2_500.0,
        cpu_load_flops: 600.0,
        required_station_tech_level: 2,
        max_internal_storage_slots: 8,
        max_internal_storage_volume_per_slot_m3: Some(2000),
        provides_station_morale_boost: Some(-8),
        icon_asset_id: Some("station.icon.factory".to_string()),
        construction_hp: 3_500,
        operational_hp: 45_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_MANUFACTURING_FACTORY_ADVANCED,
        name: "Advanced Factory".to_string(),
        description: "A high-tech manufacturing facility with automated production lines."
            .to_string(),
        category: StationModuleCategory::ManufacturingAndAssembly,
        specific_type: StationModuleSpecificType::FactoryAdvancedComponents,
        build_cost_resources: calculate_basic_module_cost(80_000, 2),
        build_time_seconds: 0,
        power_consumption_mw_operational: 70_000.0,
        power_consumption_mw_idle: 3_500.0,
        cpu_load_flops: 1_000.0,
        required_station_tech_level: 3,
        max_internal_storage_slots: 15,
        max_internal_storage_volume_per_slot_m3: Some(4000),
        provides_station_morale_boost: Some(-15),
        icon_asset_id: Some("station.icon.factory".to_string()),
        construction_hp: 6_500,
        operational_hp: 80_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_MANUFACTURING_ASSEMBLER,
        name: "Component Assembler".to_string(),
        description: "A specialized facility for assembling complex components from basic parts."
            .to_string(),
        category: StationModuleCategory::ManufacturingAndAssembly,
        specific_type: StationModuleSpecificType::ComponentAssembler,
        build_cost_resources: calculate_basic_module_cost(60_000, 2),
        build_time_seconds: 0,
        power_consumption_mw_operational: 50_000.0,
        power_consumption_mw_idle: 3_000.0,
        cpu_load_flops: 800.0,
        required_station_tech_level: 2,
        max_internal_storage_slots: 12,
        max_internal_storage_volume_per_slot_m3: Some(3000),
        provides_station_morale_boost: Some(-12),
        icon_asset_id: Some("station.icon.assembler".to_string()),
        construction_hp: 5_000,
        operational_hp: 60_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_MANUFACTURING_SHIPYARD,
        name: "Shipyard Fabrication".to_string(),
        description: "A massive facility capable of constructing and repairing spacecraft."
            .to_string(),
        category: StationModuleCategory::ManufacturingAndAssembly,
        specific_type: StationModuleSpecificType::ShipyardFabrication,
        build_cost_resources: calculate_basic_module_cost(150_000, 4),
        build_time_seconds: 0,
        power_consumption_mw_operational: 120_000.0,
        power_consumption_mw_idle: 5_000.0,
        cpu_load_flops: 2_000.0,
        required_station_tech_level: 4,
        max_internal_storage_slots: 25,
        max_internal_storage_volume_per_slot_m3: Some(8000),
        provides_station_morale_boost: Some(-25),
        icon_asset_id: Some("station.icon.shipyard".to_string()),
        construction_hp: 12_000,
        operational_hp: 150_000,
    })?;

    /////////////////////////////////////////////////////////////////////////////////////
    // Solar Array Modules

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_SOLAR_ARRAY_SMALL,
        name: "Small Solar Array".to_string(),
        description: "A compact solar power generation system for basic energy needs.".to_string(),
        category: StationModuleCategory::ResourceProductionAndRefining,
        specific_type: StationModuleSpecificType::SolarArray,
        build_cost_resources: calculate_basic_module_cost(20_000, 0),
        build_time_seconds: 0,
        power_consumption_mw_operational: 15_000.0,
        power_consumption_mw_idle: 1_500.0,
        cpu_load_flops: 300.0,
        required_station_tech_level: 1,
        max_internal_storage_slots: 3,
        max_internal_storage_volume_per_slot_m3: Some(1000),
        provides_station_morale_boost: Some(5),
        icon_asset_id: Some("station.icon.solar".to_string()),
        construction_hp: 1_500,
        operational_hp: 20_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_SOLAR_ARRAY_MEDIUM,
        name: "Medium Solar Array".to_string(),
        description: "A mid-sized solar power system with improved efficiency.".to_string(),
        category: StationModuleCategory::ResourceProductionAndRefining,
        specific_type: StationModuleSpecificType::SolarArray,
        build_cost_resources: calculate_basic_module_cost(35_000, 1),
        build_time_seconds: 0,
        power_consumption_mw_operational: 25_000.0,
        power_consumption_mw_idle: 2_000.0,
        cpu_load_flops: 500.0,
        required_station_tech_level: 1,
        max_internal_storage_slots: 6,
        max_internal_storage_volume_per_slot_m3: Some(2000),
        provides_station_morale_boost: Some(8),
        icon_asset_id: Some("station.icon.solar".to_string()),
        construction_hp: 2_500,
        operational_hp: 35_000,
    })?;

    dsl.create_station_module_blueprint(CreateStationModuleBlueprint {
        id: MODULE_SOLAR_ARRAY_LARGE,
        name: "Large Solar Array".to_string(),
        description: "A large-scale solar power installation for industrial energy requirements."
            .to_string(),
        category: StationModuleCategory::ResourceProductionAndRefining,
        specific_type: StationModuleSpecificType::SolarArray,
        build_cost_resources: calculate_basic_module_cost(60_000, 2),
        build_time_seconds: 0,
        power_consumption_mw_operational: 45_000.0,
        power_consumption_mw_idle: 3_000.0,
        cpu_load_flops: 800.0,
        required_station_tech_level: 2,
        max_internal_storage_slots: 12,
        max_internal_storage_volume_per_slot_m3: Some(4000),
        provides_station_morale_boost: Some(15),
        icon_asset_id: Some("station.icon.solar".to_string()),
        construction_hp: 4_500,
        operational_hp: 60_000,
    })?;

    Ok(())
}
