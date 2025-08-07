use super::*;

//////////////////////////////////////////////////////////////
// Impls
//////////////////////////////////////////////////////////////

// impl Ship {
//     //
// }

impl ShipStatus {
    pub fn get_remaining_cargo_space(&self) -> u16 {
        self.get_max_cargo_capacity() - self.get_used_cargo_capacity()
    }

    pub fn calculate_used_cargo_space(&self, ctx: &ReducerContext) -> u16 {
        let dsl = dsl(ctx);
        let mut used_cargo_space = 0;

        info!(
            "Calculating cargo space usage for ship #{}. (Max cargo {}v)",
            self.id, self.max_cargo_capacity
        );

        // Collect all the ship items and calculate their volume usage
        for item in dsl.get_ship_cargo_items_by_ship_id(self.get_id()) {
            if let Ok(item_def) = get_item_definition(ctx, item.get_item_id().value()) {
                let volume_usage = item.quantity * item_def.get_volume_per_unit();
                info!(
                    "     Stack of {}x {}: {} volume used",
                    item.quantity,
                    item_def.get_name(),
                    volume_usage
                );
                used_cargo_space += volume_usage;
            }
        }
        info!(
            "Total cargo space usage for ship #{}: {}",
            self.id, used_cargo_space
        );

        used_cargo_space
    }
}
