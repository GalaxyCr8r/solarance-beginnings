use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table};

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum StationKind {
    TradeHub,
    Refinery,
    Factory,
    StorageDepot,
}