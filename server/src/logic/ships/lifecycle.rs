use crate::spacetimedsl::prelude::*;

use crate::tables::{players::*, stellarobjects::*};

pub fn initialize_player_ship<T: spacetimedsl::WriteContext>(
    _dsl: &DSL<T>,
    _player: &PlayerId,
    _sobj: &StellarObject,
) -> Result<(), String> {
    Ok(())
}

pub fn deinitialize_player_ship<T: spacetimedsl::WriteContext>(
    _dsl: &DSL<T>,
    _player: &PlayerId,
    _sobj: &StellarObject,
) -> Result<(), String> {
    Ok(())
}
