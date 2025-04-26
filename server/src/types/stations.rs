use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum StationKind {
    TradeHub,
    Refinery,
    Factory,
    StorageDepot,
}