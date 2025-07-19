use log::info;

use crate::types::items::ItemDefinition;

use super::*;

impl StationSize {
    /// How many modules can this size support?
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
        (self.modules().pow(2) as u32) * 100_000 + 300_000
    }

    /// Retooling a space station to a larger size should be possible, but discouraged.
    pub fn upgrade_cost(&self, new_size: StationSize) -> u32 {
        new_size.base_cost() - self.base_cost() + ((new_size.modules() - self.modules()) as u32)
    }

    pub fn base_health(&self) -> u32 {
        (self.modules().pow(2) as u32) * 25_000 + 100_000
    }

    pub fn base_shields(&self) -> u32 {
        (self.modules().pow(2) as u32) * 50_000 + 200_000
    }
}

impl StationModuleInventoryItem {
    /// Calculates the current price of an item based on its quantity and item definition.
    pub fn calculate_current_price(&self, item_def: &ItemDefinition) -> u32 {
        // Convert to floats
        let max_quantity = self.max_quantity as f32;
        let current_quantity = self.quantity as f32;
        let value = item_def.base_value as f32;

        info!("Calculating price for {}", item_def.name);
        // Calc the percent and resultant multiplier
        let percent_full = current_quantity / max_quantity;
        let multiplier = percent_full * -2.0 + 1.0; // 1.0 .. -1.0
                                                    // info!("    Curr/Max : {}/{}", current_quantity, max_quantity);
                                                    // info!("    Multipler : {}", multiplier);

        // Find the value of the given margin
        let curr_margin_perc = (item_def.margin_percentage as f32) * 0.01;
        let margin_value = value * curr_margin_perc;
        // info!("    Curr Margin Perc : {}", curr_margin_perc);
        // info!("    Base Margin Value : {}c", margin_value);
        // info!("    Adjusted Value : {}c", margin_value * multiplier);
        info!(
            "    New Value : {}c",
            (value + margin_value * multiplier) as u32
        );

        // If current_quantity == max_quantity then the current price should be base_value + (base_value * default_margin * -1.0)
        // If current_quantity == 0 then the current price should be base_value + (base_value * default_margin * 1.0)
        (value + margin_value * multiplier) as u32
    }
}
