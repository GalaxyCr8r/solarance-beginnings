use std::u8;

use log::info;
use spacetimedb::*;
use spacetimedsl::dsl;

use crate::types::items::definitions::{
    ITEM_ENERGY_CELL, ITEM_FOOD_RATIONS, ITEM_METAL_HULL_STRUCTURE, ITEM_METAL_PLATES,
};

use super::*;

pub const MODULE_TRADING_BAZAAR: u32 = 1_000;
pub const MODULE_TRADING_MARKET: u32 = 1_010;
pub const MODULE_TRADING_PORT: u32 = 1_020;

pub const MODULE_MINOR_DEPOT: u32 = 2_000;
pub const MODULE_MAJOR_DEPOT: u32 = 2_010;
pub const MODULE_CAPITOL_DEPOT: u32 = 2_020;

pub const MODULE_REFINERY_MINOR: u32 = 3_110;
pub const MODULE_REFINERY_MAJOR: u32 = 3_111;
pub const MODULE_REFINERY_EXOTIC: u32 = 3_112;

pub const MODULE_REFINERY_ICE: u32 = 3_100;
pub const MODULE_REFINERY_CARBON: u32 = 3_101;
pub const MODULE_REFINERY_IRON: u32 = 3_102;
pub const MODULE_REFINERY_SILICON: u32 = 3_103;
pub const MODULE_REFINERY_URANIUM: u32 = 3_104;
pub const MODULE_REFINERY_VIVEIUM: u32 = 3_105;
pub const MODULE_REFINERY_TITANIUM: u32 = 3_106;

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
        "A basic trading port. Can fit a reasonable amount of goods.",
        StationModuleCategory::LogisticsAndStorage,
        StationModuleSpecificType::TradingPort,
        calculate_basic_module_cost(10_000, 0),
        0,
        5_000.0,
        2_500.0,
        100.0,
        0,
        50,
        Some(1000),
        Some(10),
        None,
        1_000,
        10_000,
    )?;

    dsl.create_station_module_blueprint(
        MODULE_TRADING_MARKET,
        "Trading Market",
        "A basic trading port. Can fit a reasonable amount of goods.",
        StationModuleCategory::LogisticsAndStorage,
        StationModuleSpecificType::TradingPort,
        calculate_basic_module_cost(50_000, 0),
        0,
        5_000.0,
        2_500.0,
        100.0,
        0,
        50,
        Some(256),
        Some(10),
        None,
        5_000,
        50_000,
    )?;

    /////////////////////////////////////////////////////////////////////////////////////

    dsl.create_station_module_blueprint(
        MODULE_REFINERY_MINOR,
        "Ore Refinery",
        "A basic refinery, can process most ores.",
        StationModuleCategory::ResourceProductionAndRefining,
        StationModuleSpecificType::RefineryBasicOre,
        calculate_basic_module_cost(15_000, 0),
        0,
        50_000.0,
        2_500.0,
        500.0,
        1,
        6,
        Some(5000),
        Some(-5),
        None,
        2_000,
        15_000,
    )?;

    Ok(())
}
