use super::*;
use crate::types::items::{definitions::*, GetItemDefinitionRowOptionById, ItemDefinitionId};

#[dsl(plural_name = trading_port_modules)]
#[table(name = trading_port_module, public)]
pub struct TradingPort {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,
    // Configuration for item capacity is better in StationModuleBlueprint (max_internal_storage_slots/volume)

    // This table will mainly link to its active trade listings.
}

/// Represents items the Trading Port module is actively buying or selling.
/// This is distinct from general market orders placed by players at a station.
#[dsl(plural_name = trading_port_listings)]
#[table(name = trading_port_listing, public)]
pub struct TradingPortListing {
    #[primary_key]
    #[use_wrapper(path = StationModuleInventoryItemId)]
    /// FK to StationModuleInventoryItem
    id: u64,

    /// None if the port is not buying, Some percentage of how much margin the port want below base price.
    pub buying_margin: Option<f32>,
    /// None if the port is not selling, Some percentage of how much margin the port want above base price.
    pub selling_margin: Option<f32>,
}

/// Configuration for a trading port item listing
pub struct TradingPortItemConfig {
    pub item_id: u32,
    pub starting_quantity: u32,
    pub buying_margin: Option<f32>,
    pub selling_margin: Option<f32>,
}

/// Generic function to create a trading port with specified items and configurations
pub fn create_trading_port_with_items(
    ctx: &ReducerContext,
    station: &Station,
    module_name: &str,
    items: &[TradingPortItemConfig],
    under_construction: bool,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if under_construction {
        return Err("Stations under construction are not yet implemented".to_string());
    }

    let blueprint = dsl.get_station_module_blueprint_by_id(StationModuleBlueprintId::new(
        definitions::MODULE_TRADING_BAZAAR,
    ))?;

    let module = dsl.create_station_module(
        station.get_id(),
        blueprint.get_id(),
        module_name,
        true,
        None,
        ctx.timestamp,
    )?;

    let _trading = dsl.create_trading_port_module(module.get_id())?;

    // Create trading port listings for each configured item
    for item_config in items {
        // Calculate and set initial cached current price
        if let Ok(item_def) =
            dsl.get_item_definition_by_id(ItemDefinitionId::new(item_config.item_id))
        {
            let mut item = dsl.create_station_module_inventory_item(
                module.get_id(),
                ItemDefinitionId::new(item_config.item_id),
                item_config.starting_quantity,
                blueprint.max_internal_storage_volume_per_slot_m3.unwrap()
                    / *item_def.get_volume_per_unit() as u32,
                format!("{};{};trading", module.id, item_config.item_id).as_str(),
                0, // Initial cached price, will be updated immediately
            )?;

            let initial_price = item.calculate_current_price(&item_def);
            item.set_cached_price(initial_price);
            dsl.update_station_module_inventory_item_by_id(item.clone())?;

            dsl.create_trading_port_listing(
                item.get_id(),
                item_config.buying_margin,
                item_config.selling_margin,
            )?;
        }
    }

    Ok(())
}

pub fn create_basic_bazaar(
    ctx: &ReducerContext,
    station: &Station,
    under_construction: bool,
) -> Result<(), String> {
    let items = vec![
        TradingPortItemConfig {
            item_id: ITEM_CARBON_RAW,
            starting_quantity: 10,
            buying_margin: Some(0.8),
            selling_margin: None,
        },
        TradingPortItemConfig {
            item_id: ITEM_IRON_INGOT,
            starting_quantity: 20,
            buying_margin: Some(0.8),
            selling_margin: None,
        },
        TradingPortItemConfig {
            item_id: ITEM_HELIUM_GAS,
            starting_quantity: 40,
            buying_margin: Some(0.8),
            selling_margin: None,
        },
        TradingPortItemConfig {
            item_id: ITEM_HYDROGEN_GAS,
            starting_quantity: 40,
            buying_margin: Some(0.8),
            selling_margin: None,
        },
        TradingPortItemConfig {
            item_id: ITEM_ENERGY_CELL,
            starting_quantity: 512,
            buying_margin: Some(0.5),
            selling_margin: None,
        },
        TradingPortItemConfig {
            item_id: ITEM_FOOD_RATIONS,
            starting_quantity: 100,
            buying_margin: Some(0.5),
            selling_margin: None,
        },
        TradingPortItemConfig {
            item_id: ITEM_FOOD_AVERAGE,
            starting_quantity: 100,
            buying_margin: Some(0.5),
            selling_margin: None,
        },
        TradingPortItemConfig {
            item_id: ITEM_WATER,
            starting_quantity: 50,
            buying_margin: Some(0.5),
            selling_margin: None,
        },
    ];

    create_trading_port_with_items(ctx, station, "bazaar", &items, under_construction)
}

pub fn create_rich_speciality(
    ctx: &ReducerContext,
    station: &Station,
    under_construction: bool,
) -> Result<(), String> {
    let items = vec![
        TradingPortItemConfig {
            item_id: ITEM_URANIUM_ENRICHED,
            starting_quantity: 10,
            buying_margin: Some(0.8),
            selling_margin: None,
        },
        TradingPortItemConfig {
            item_id: ITEM_VIVEIUM_CRYSTAL,
            starting_quantity: 20,
            buying_margin: Some(0.8),
            selling_margin: None,
        },
        TradingPortItemConfig {
            item_id: ITEM_GOLD_INGOT,
            starting_quantity: 40,
            buying_margin: Some(0.8),
            selling_margin: None,
        },
        TradingPortItemConfig {
            item_id: ITEM_ALCOHOL,
            starting_quantity: 1000,
            buying_margin: Some(0.5),
            selling_margin: None,
        },
        TradingPortItemConfig {
            item_id: ITEM_FOOD_LUXURY,
            starting_quantity: 100,
            buying_margin: Some(0.5),
            selling_margin: None,
        },
        TradingPortItemConfig {
            item_id: ITEM_COMPUTER_BOARD,
            starting_quantity: 100,
            buying_margin: Some(0.5),
            selling_margin: None,
        },
    ];

    create_trading_port_with_items(ctx, station, "speciality", &items, under_construction)
}
