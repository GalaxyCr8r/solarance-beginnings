use std::time::Duration;

use glam::Vec2;
use log::info;
use spacetimedb::{rand::Rng, TimeDuration};

use crate::{
    logic::ships::player_controller::initialize_player_controller,
    tables::{
        factions::FactionId,
        players::PlayerId,
        server_messages::utility::send_info_message,
        ships::timers::*,
        stellarobjects::{reducers::create_sobj_player_window_for, utility::*, *},
    },
};

use super::*;
