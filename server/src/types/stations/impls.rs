use super::*;


impl StationSize {
    /// How many modules can this szie support?
    pub fn modules(&self) -> u8 {
        match self {
            StationSize::Capital => 13,
            StationSize::Large => 9,
            StationSize::Medium => 7,
            StationSize::Small => 5,
            StationSize::Outpost => 3,
            StationSize::Satellite => 1,
        }
    }

    pub fn base_cost(&self) -> u32 {
        (self.modules().pow(2)) as u32 * 100_000 + 300_000
    }

    /// Retooling a space station to a larger size should be possible, but discouraged.
    pub fn upgrade_cost(&self, new_size: StationSize) -> u32 {
        new_size.base_cost() - self.base_cost() + (new_size.modules() - self.modules()) as u32
    }

    pub fn base_health(&self) -> u32 {
        (self.modules().pow(2)) as u32 * 25_000 + 100_000
    }
}