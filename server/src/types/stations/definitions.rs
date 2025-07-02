use std::u8;

use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::dsl;

use crate::types::{
    common::Vec2,
    items::definitions::{
        ITEM_COMPUTER_CORE,
        ITEM_ENERGY_CELL,
        ITEM_FOOD_RATIONS,
        ITEM_METAL_HULL,
        ITEM_METAL_PLATES,
    },
    stations::CreateStationRow,
    stellarobjects::{ utility::create_sobj_internal, StellarObjectTransformInternal },
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
    let dsl = dsl(ctx);

    basic_station_modules(ctx)?;

    info!("Station Modules Loaded: {}", dsl.get_all_station_module_blueprints().count());
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn calculateBasicModuleCost(max_hp: u32, relative_complexity: u8) -> Vec<ResourceAmount> {
    vec![
        ResourceAmount::new(ITEM_METAL_HULL, max_hp + ((relative_complexity as u32) * 500) / 10),
        ResourceAmount::new(ITEM_ENERGY_CELL, max_hp + ((relative_complexity as u32) * 1_000) / 5),
        ResourceAmount::new(ITEM_COMPUTER_CORE, max_hp / (575 / (relative_complexity as u32))),
        ResourceAmount::new(ITEM_METAL_PLATES, max_hp + ((relative_complexity as u32) * 100) / 125),
        ResourceAmount::new(ITEM_FOOD_RATIONS, max_hp + ((relative_complexity as u32) * 1_000) / 25)
    ]
}

fn basic_station_modules(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    /*
    pub category: StationModuleCategory,
    pub specific_type: StationModuleSpecificType,

    pub build_cost_resources: Vec<ResourceAmount>,
    pub build_time_seconds: u32,

    pub power_consumption_mw_operational: f32, // Power needed when active
    pub power_consumption_mw_idle: f32, // Power needed when idle
    pub cpu_load_flops: f32,

    pub required_station_tech_level: u8,

    pub max_internal_storage_slots: u16, // Number of distinct item types it can hold
    pub max_internal_storage_volume_per_slot_m3: Option<f32>, // Volume per slot

    pub provides_station_morale_boost: Option<i16>, // Base morale if applicable
    pub icon_asset_id: Option<String>,

    pub construction_hp: u32, // HP during construction phase
    pub operational_hp: u32, // Max HP when fully built */

    dsl.create_station_module_blueprint(
        MODULE_TRADING_BAZAAR,
        "Trading Bazaar",
        "A basic trading port. Can fit a reasonable amount of goods.",
        StationModuleCategory::CivilianAndSupportServices,
        StationModuleSpecificType::TradingPort,
        calculateBasicModuleCost(10_000, 0),
        0,
        5_000.0,
        2_500.0,
        100.0,
        0,
        50,
        Some(10),
        Some(10),
        None,
        1_000,
        10_000
    )?;

    dsl.create_station_module_blueprint(
        MODULE_TRADING_MARKET,
        "Trading Market",
        "A basic trading port. Can fit a reasonable amount of goods.",
        StationModuleCategory::CivilianAndSupportServices,
        StationModuleSpecificType::TradingPort,
        calculateBasicModuleCost(50_000, 0),
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
        50_000
    )?;

    ///////////////////////////////////////////////////////////////////////////////////////

    dsl.create_station_module_blueprint(
        MODULE_REFINERY_MINOR,
        "Ore Refinery",
        "A basic refinery, can process most ores.",
        StationModuleCategory::ResourceProductionAndRefining,
        StationModuleSpecificType::RefineryBasicOre,
        calculateBasicModuleCost(15_000, 0),
        0,
        50_000.0,
        2_500.0,
        500.0,
        1,
        6,
        Some(1000),
        Some(-5),
        None,
        2_000,
        15_000
    )?;

    Ok(())
}
