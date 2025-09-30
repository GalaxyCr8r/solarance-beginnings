use super::*;
use spacetimedsl::{dsl, Wrapper};

// Timers related to combat structs will go here

#[spacetimedb::reducer]
pub fn cleanup_visual_effect(ctx: &ReducerContext, timer: VisualEffectTimer) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Delete the visual effect from the database
    let effect_id = VisualEffectId::new(timer.get_effect_id().value());
    if let Ok(_) = dsl.delete_visual_effect_by_id(effect_id) {
        spacetimedb::log::info!("Cleaned up visual effect {}", timer.get_effect_id().value());
    } else {
        // Visual effect might have already been deleted, which is fine
        spacetimedb::log::info!(
            "Visual effect {} already cleaned up",
            timer.get_effect_id().value()
        );
    }

    // The timer itself is automatically deleted by SpacetimeDB after this reducer runs
    Ok(())
}

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {
    // Timer initialization will go here
    Ok(())
}
