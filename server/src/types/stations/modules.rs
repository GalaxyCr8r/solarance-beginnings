use super::*;
use crate::types::economy::ResourceAmount;

// Child modules for different station module types
pub mod trading_port;
pub mod storage_depot;
pub mod embassy;
pub mod farm;
pub mod observatory;
pub mod refinery;
pub mod solar_array;
pub mod synthesizer;
pub mod manufacturing;
pub mod laboratory;
pub mod capital_dock;
pub mod anti_capital_turret;
pub mod residential;
pub mod hospital;

// Re-export the main types from child modules
pub use trading_port::{TradingPort, TradingPortListing};
pub use storage_depot::StorageDepot;
pub use embassy::{Embassy, EmbassyPresence};
pub use farm::{Farm, FarmOutputQuality};
pub use observatory::Observatory;
pub use refinery::Refinery;
pub use solar_array::SolarArray;
pub use synthesizer::Synthesizer;
pub use manufacturing::{Manufacturing, ProductionRecipeDefinition};
pub use laboratory::Laboratory;
pub use capital_dock::{CapitalDock, DockedCapitalShipAt};
pub use anti_capital_turret::AntiCapitalTurret;
pub use residential::Residential;
pub use hospital::Hospital;

// Re-export helper functions
pub use trading_port::create_basic_bazaar;
pub use refinery::definitions::create_basic_refinery_module;