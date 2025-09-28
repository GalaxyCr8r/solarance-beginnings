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

impl ShipTypeDefinition {
    pub fn get_world_corners_at_position(
        &self,
        position: &crate::types::common::Vec2,
        angle: f32,
    ) -> [crate::types::common::Vec2; 4] {
        let half_width = self.sprite_width as f32 / 2.0;
        let half_height = self.sprite_height as f32 / 2.0;

        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // Corners relative to ship's center, assuming 0 orientation
        let corners_local = [
            (half_width, half_height),   // top_right
            (-half_width, half_height),  // top_left
            (-half_width, -half_height), // bottom_left
            (half_width, -half_height),  // bottom_right
        ];

        // Rotate and translate to world space
        corners_local.map(|(x, y)| {
            let rotated_x = x * cos_angle - y * sin_angle;
            let rotated_y = x * sin_angle + y * cos_angle;
            crate::types::common::Vec2::new(position.x + rotated_x, position.y + rotated_y)
        })
    }
}
