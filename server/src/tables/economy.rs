use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Debug, Clone)]
pub struct ResourceAmount {
    //#[use_wrapper(crate::tables::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub resource_item_id: u32,

    pub quantity: u32,
}

impl ResourceAmount {
    pub fn new(resource_item_id: u32, quantity: u32) -> Self {
        ResourceAmount {
            resource_item_id,
            quantity,
        }
    }
}

impl PartialEq for ResourceAmount {
    fn eq(&self, other: &Self) -> bool {
        self.resource_item_id == other.resource_item_id && self.quantity == other.quantity
    }
}
