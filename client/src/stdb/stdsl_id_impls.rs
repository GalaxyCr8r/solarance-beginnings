// Regex to match Id struct names.
/*
(\w+Id) (\w+)

impl $1 {
    pub fn new(id: $2) -> Self {
        $1 { value: id }
    }
}

OR:
impl From<$2> for $1 {
    fn from(id: $2) -> Self {
        $1 { value: id }
    }
}

*/

use crate::module_bindings::*;

impl From<u32> for FactionId {
    fn from(id: u32) -> Self {
        FactionId { value: id }
    }
}

impl From<u32> for ItemDefinitionId {
    fn from(id: u32) -> Self {
        ItemDefinitionId { value: id }
    }
}

impl From<u64> for SectorId {
    fn from(id: u64) -> Self {
        SectorId { value: id }
    }
}

impl From<u64> for ShipId {
    fn from(id: u64) -> Self {
        ShipId { value: id }
    }
}

impl From<u64> for StationModuleId {
    fn from(id: u64) -> Self {
        StationModuleId { value: id }
    }
}

impl From<u64> for StellarObjectId {
    fn from(id: u64) -> Self {
        StellarObjectId { value: id }
    }
}
