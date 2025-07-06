use spacetimedb::*;

use super::*;

pub fn get_item_definition(ctx: &ReducerContext, item_id: u32) -> Result<ItemDefinition, String> {
    // TODO DSL-ify this.
    if let Some(item) = ctx.db().item_definition().id().find(item_id) {
        //info!("item def id: {}", item_id);
        Ok(item)
    } else {
        // for item in ctx.db().item_definition().iter() {
        //     info!("item id: {} - Name: {}", item.id, item.name)
        // }

        Err(format!("item def id: {} NOT FOUND", item_id))
    }
}
