use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::dsl;

use crate::types::{
    common::Vec2,
    stations::CreateStationRow,
    stellarobjects::{ utility::create_sobj_internal, StellarObjectTransformInternal },
};

use super::*;

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
        "Trading Port Mk1",
        "A basic trading port. Can fit a reasonable amount of goods.",
        StationModuleCategory::CivilianAndSupportServices,
        StationModuleSpecificType::TradingPort,
        vec![ResourceAmount { resource_item_id: todo!(), quantity: 1_000 }],
        0,
        power_consumption_mw_operational,
        power_consumption_mw_idle,
        cpu_load_flops,
        required_station_tech_level,
        max_internal_storage_slots,
        max_internal_storage_volume_per_slot_m3,
        provides_station_morale_boost,
        icon_asset_id,
        construction_hp,
        operational_hp
    )
}
