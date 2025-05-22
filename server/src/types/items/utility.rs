use super::{*};

pub fn get_item_definition(ctx: &ReducerContext, item_id: u32) -> Option<ItemDefinition> {
    if let Some(item) = ctx.db.item_definition().id().find(item_id) {
        Some(item)
    } else {
        None
    }
}