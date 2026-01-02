use spacetimedsl::*;

use crate::tables::{players::*, stellarobjects::*};

pub fn initialize_player_ship<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player: &PlayerId,
    sobj: &StellarObject,
) -> Result<(), String> {
    Ok(())
}

pub fn deinitialize_player_ship<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player: &PlayerId,
    sobj: &StellarObject,
) -> Result<(), String> {
    Ok(())
}
