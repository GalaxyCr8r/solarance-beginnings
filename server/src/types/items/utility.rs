use log::info;
use spacetimedb::Table;

use super::{*};

pub fn get_item_definition(ctx: &ReducerContext, item_id: u32) -> Option<ItemDefinition> {
    if let Some(item) = ctx.db.item_definition().id().find(item_id) {
        info!("item def id: {}", item_id);
        Some(item)
    } else {
        info!("item def id: {} NOT FOUND", item_id);
        for item in ctx.db.item_definition().iter() {
            info!("item id: {} - Name: {}", item.id, item.name)
        }

        None
    }
}