use spacetimedb::{ReducerContext, SpacetimeType};

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum ResourceType {
    IronOre,
    Silicon,
    Ice,
    Water,
    MetalPlate,
    ComputerChips,
    Fuel,
}

#[derive(SpacetimeType, Debug, Clone)]
pub struct ItemStack {
    pub resource: ResourceType,
    pub quantity: u32,
}

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum OrderType {
    Mine(ResourceType),
    HaulToStation(u64),       // station_id
    TradeAtStation(u64),      // station_id
    DefendSector(u64),        // sector_id
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {

    Ok(())
}